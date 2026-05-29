use anyhow::{Context, Result};
use std::env::current_dir;
use std::fs::{self, File};
use std::io::Write;
use std::path::{Component, Path, PathBuf};
use toml::Value;
use tracing::debug;

#[derive(Debug)]
pub struct Project {
    pub is_crate: bool,
    pub is_workspace_member: bool,
    pub is_workspace_root: bool,
    pub workspace_members: Vec<WorkspaceMember>,
    pub current_dir: PathBuf,
    pub manifest_path: Option<PathBuf>,
}

#[derive(Debug)]
pub struct WorkspaceMember {
    pub name: String,
    pub relative_path: PathBuf,
    pub absolute_path: PathBuf,
}

impl Project {
    pub fn default() -> Result<Self> {
        Self::new(&current_dir()?)
    }

    pub fn new(current_dir: &Path) -> Result<Self> {
        let manifest_path = current_dir.join("Cargo.toml");
        let is_crate = manifest_path.exists();

        let (is_part_of_workspace, workspace_root) = if is_crate {
            find_workspace_root(&manifest_path)?
        } else {
            (false, None)
        };

        let is_workspace_root = workspace_root.as_deref() == Some(current_dir);

        let workspace_members = match (is_workspace_root, &workspace_root) {
            (true, _) => get_workspace_members(&manifest_path)
                .context(format!("Не удалось прочитать {}", manifest_path.display()))?,
            (false, Some(root)) => get_workspace_members(&root.join("Cargo.toml"))?,
            (false, None) => vec![],
        };

        Ok(Self {
            is_crate,
            is_workspace_member: is_part_of_workspace && is_crate,
            is_workspace_root,
            workspace_members,
            current_dir: current_dir.to_path_buf(),
            manifest_path: is_crate.then_some(manifest_path),
        })
    }

    pub fn collect_sources(&self, output_path: &str) -> Result<()> {
        let mut output =
            File::create(output_path).context(format!("Не удалось создать {}", output_path))?;

        if self.is_workspace_root {
            for member in &self.workspace_members {
                writeln!(output, "// ===== CRATE: {} =====\n", member.name)?;
                collect_rs_files(
                    &self.current_dir,
                    &member.absolute_path.join("src"),
                    &mut output,
                )?;
                writeln!(output)?;
            }
        } else if self.is_crate {
            collect_rs_files(
                &self.current_dir,
                &self.current_dir.join("src"),
                &mut output,
            )?;
        } else {
            anyhow::bail!("Текущий каталог не является крейтом или воркспейсом");
        }

        Ok(())
    }
}

fn find_workspace_root(manifest_path: &Path) -> Result<(bool, Option<PathBuf>)> {
    debug!("manifest path {:?}", manifest_path);

    let parsed = read_toml(manifest_path)?;

    // Если есть секция [workspace] — это корень воркспейса
    if parsed.get("workspace").is_some() {
        return Ok((true, Some(manifest_path.parent().unwrap().to_path_buf())));
    }

    // Поднимаемся вверх по дереву каталогов в поисках корня воркспейса
    let crate_dir = manifest_path.parent().unwrap();
    let mut current = crate_dir;

    while let Some(parent) = current.parent() {
        let parent_manifest = parent.join("Cargo.toml");
        if parent_manifest.exists() {
            let parent_parsed = read_toml(&parent_manifest)?;
            if let Some(members) = get_members_from_toml(&parent_parsed) {
                let is_member = members
                    .iter()
                    .any(|m| normalize_path(&parent.join(m)) == normalize_path(crate_dir));
                if is_member {
                    return Ok((true, Some(parent.to_path_buf())));
                }
            }
        }
        current = parent;
    }

    Ok((false, None))
}

fn get_workspace_members(manifest_path: &Path) -> Result<Vec<WorkspaceMember>> {
    let parsed = read_toml(manifest_path)?;
    let workspace_dir = manifest_path.parent().unwrap();

    let mut members = get_members_from_toml(&parsed)
        .unwrap_or_default()
        .into_iter()
        .filter_map(|member_path| {
            let absolute_path = workspace_dir.join(&member_path);
            let cargo_toml = absolute_path.join("Cargo.toml");
            cargo_toml.exists().then(|| {
                let name = get_crate_name(&cargo_toml)
                    .or_else(|| {
                        Path::new(&member_path)
                            .file_name()?
                            .to_str()
                            .map(String::from)
                    })
                    .unwrap_or_else(|| member_path.clone());
                WorkspaceMember {
                    name,
                    relative_path: PathBuf::from(&member_path),
                    absolute_path,
                }
            })
        })
        .collect::<Vec<_>>();

    members.sort_by(|a, b| a.name.cmp(&b.name));
    Ok(members)
}

fn get_members_from_toml(parsed: &Value) -> Option<Vec<String>> {
    let members = parsed
        .get("workspace")?
        .get("members")?
        .as_array()?
        .iter()
        .filter_map(|v| v.as_str().map(String::from))
        .collect::<Vec<_>>();

    (!members.is_empty()).then_some(members)
}

fn get_crate_name(manifest_path: &Path) -> Option<String> {
    let parsed = read_toml(manifest_path).ok()?;
    parsed
        .get("package")?
        .get("name")?
        .as_str()
        .map(String::from)
}

/// Читает и парсит TOML файл
fn read_toml(path: &Path) -> Result<Value> {
    let content =
        fs::read_to_string(path).context(format!("Не удалось прочитать {}", path.display()))?;
    toml::from_str(&content).context(format!("Не удалось распарсить {}", path.display()))
}

/// Нормализует путь без обращения к файловой системе (безопасно на Windows)
fn normalize_path(path: &Path) -> PathBuf {
    let mut components = Vec::new();
    for component in path.components() {
        match component {
            Component::CurDir => {}
            Component::ParentDir => {
                components.pop();
            }
            other => components.push(other),
        }
    }
    components.iter().collect()
}

/// Рекурсивно обходит директорию и записывает содержимое .rs файлов.
/// Путь к файлу пишется относительно base_dir.
fn collect_rs_files(base_dir: &Path, dir: &Path, output: &mut impl Write) -> Result<()> {
    if !dir.exists() {
        debug!("src/ не найден в {:?}", dir);
        return Ok(());
    }

    let mut entries: Vec<_> = fs::read_dir(dir)
        .context(format!("Не удалось прочитать директорию {}", dir.display()))?
        .filter_map(|e| e.ok())
        .collect();

    entries.sort_by_key(|e| e.path());

    for entry in entries {
        let path = entry.path();
        if path.is_dir() {
            collect_rs_files(base_dir, &path, output)?;
        } else if path.extension().and_then(|e| e.to_str()) == Some("rs") {
            let relative = path.strip_prefix(base_dir).unwrap_or(&path);
            writeln!(output, "// ----- {} -----\n", relative.display())?;

            let content = fs::read_to_string(&path)
                .context(format!("Не удалось прочитать {}", path.display()))?;
            output.write_all(content.as_bytes())?;

            if !content.ends_with('\n') {
                writeln!(output)?;
            }
            writeln!(output)?;
        }
    }

    Ok(())
}
