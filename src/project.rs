use anyhow::{Context, Result};
use std::fs;
use std::path::{Path, PathBuf};

/// Результат анализа текущего каталога
#[derive(Debug)]
struct CargoSnapshot {
    /// Является ли текущий каталог крейтом
    is_crate: bool,
    /// Является ли частью workspace
    is_workspace_member: bool,
    /// Является ли корнем workspace
    is_workspace_root: bool,
    /// Члены workspace (если текущий каталог - корень workspace)
    workspace_members: Vec<WorkspaceMember>,
    /// Путь к Cargo.toml текущего проекта
    manifest_path: Option<PathBuf>,
    // /// Тип проекта
    // project_type: ProjectType,
}

#[derive(Debug)]
struct WorkspaceMember {
    /// Имя крейта
    name: String,
    /// Путь к крейту относительно корня workspace
    relative_path: PathBuf,
    /// Абсолютный путь к крейту
    absolute_path: PathBuf,
}

/// Тип проекта для более ясной семантики
#[derive(Debug)]
enum ProjectType {
    /// Обычный крейт (не в workspace)
    StandaloneCrate,
    /// Крейт, входящий в workspace
    WorkspaceMember { root_path: PathBuf },
    /// Корень workspace
    WorkspaceRoot,
    /// Не является крейтом
    NotACrate,
}

/// Анализирует указанный каталог
fn analyze_directory(path: &Path) -> Result<CargoSnapshot> {
    let manifest_path = path.join("Cargo.toml");
    let is_crate = manifest_path.exists();

    let (is_part_of_workspace, workspace_root) = if is_crate {
        find_workspace_root(&manifest_path)?
    } else {
        (false, None)
    };

    let is_workspace_root = if let Some(root) = &workspace_root {
        root == path
    } else {
        false
    };

    let workspace_members = if is_workspace_root {
        get_workspace_members(&manifest_path)
            .context(format!("Не удалось прочитать {}", manifest_path.display()))?
    } else if let Some(root) = workspace_root {
        // Если это не корень, но часть воркспейса
        let root_manifest = root.join("Cargo.toml");
        get_workspace_members(&root_manifest)?
    } else {
        vec![]
    };

    Ok(CargoSnapshot {
        is_crate,
        is_workspace_member: is_part_of_workspace && is_crate,
        is_workspace_root,
        workspace_members,
        manifest_path: if is_crate { Some(manifest_path) } else { None },
    })
}

/// Находит корень воркспейса, если текущий крейт является его частью
fn find_workspace_root(manifest_path: &Path) -> Result<(bool, Option<PathBuf>)> {
    let content = fs::read_to_string(manifest_path)
        .context(format!("Не удалось прочитать {}", manifest_path.display()))?;

    // Парсим как TOML (упрощенный вариант, не требует внешних крейтов)
    if let Some(workspace) = find_workspace_section(&content) {
        // Если есть секция [workspace], то это корень воркспейса
        if workspace.contains("[workspace]") {
            return Ok((true, Some(manifest_path.parent().unwrap().to_path_buf())));
        }
    }

    // Ищем виртуальный воркспейс
    if let Some(virtual_workspace) = find_virtual_workspace(&content) {
        if virtual_workspace {
            return Ok((true, Some(manifest_path.parent().unwrap().to_path_buf())));
        }
    }

    // Ищем членство в воркспейсе
    if let Some(members) = find_workspace_members(&content) {
        if !members.is_empty() {
            // Это корень воркспейса
            return Ok((true, Some(manifest_path.parent().unwrap().to_path_buf())));
        }
    }

    // Проверяем, не является ли этот крейт членом воркспейса
    // Поднимаемся вверх по дереву каталогов
    let mut current = manifest_path.parent().unwrap();
    while let Some(parent) = current.parent() {
        let parent_manifest = parent.join("Cargo.toml");
        if parent_manifest.exists() {
            let parent_content = fs::read_to_string(&parent_manifest).context(format!(
                "Не удалось прочитать {}",
                parent_manifest.display()
            ))?;

            if let Some(members) = find_workspace_members(&parent_content) {
                for member in members {
                    let member_path = parent.join(member);
                    if current.starts_with(&member_path) || current == member_path {
                        return Ok((true, Some(parent.to_path_buf())));
                    }
                }
            }
        }
        current = parent;
    }

    Ok((false, None))
}

/// Получает список всех крейтов в воркспейсе
fn get_workspace_members(manifest_path: &Path) -> Result<Vec<WorkspaceMember>> {
    let content = fs::read_to_string(manifest_path)?;

    let workspace_dir = manifest_path.parent().unwrap();
    let mut members = Vec::new();

    // Получаем список членов из секции [workspace]
    if let Some(member_paths) = find_workspace_members(&content) {
        for member_path in member_paths {
            let relative_path = workspace_dir.join(&member_path);
            let cargo_toml = relative_path.join("Cargo.toml");
            let absolute_path = fs::canonicalize(&relative_path)?;

            if cargo_toml.exists() {
                if let Some(name) = get_crate_name(&cargo_toml) {
                    members.push(WorkspaceMember {
                        name,
                        relative_path,
                        absolute_path,
                    });
                } else if let Some(name) = get_package_name_from_path(&member_path) {
                    members.push(WorkspaceMember {
                        name,
                        relative_path,
                        absolute_path,
                    });
                }
            }
        }
    }

    // Сортируем для консистентности
    members.sort_by(|a, b| a.name.cmp(&b.name));
    Ok(members)
}

/// Поиск секции workspace в TOML
fn find_workspace_section(content: &str) -> Option<&str> {
    let lines: Vec<&str> = content.lines().collect();
    for (i, line) in lines.iter().enumerate() {
        if line.trim() == "[workspace]" {
            let mut end = i + 1;
            while end < lines.len() && !lines[end].trim().starts_with('[') {
                end += 1;
            }
            return Some(
                &content[lines[i].as_ptr() as usize - content.as_ptr() as usize
                    ..lines[end - 1].as_ptr() as usize + lines[end - 1].len()
                        - content.as_ptr() as usize],
            );
        }
    }
    None
}

/// Проверка на виртуальный воркспейс
fn find_virtual_workspace(content: &str) -> Option<bool> {
    let has_workspace = content.contains("[workspace]");
    let has_package = content.contains("[package]");
    Some(has_workspace && !has_package)
}

/// Поиск членов воркспейса
fn find_workspace_members(content: &str) -> Option<Vec<String>> {
    if let Some(workspace_section) = find_workspace_section(content) {
        let mut members = Vec::new();
        for line in workspace_section.lines() {
            let line = line.trim();
            if line.starts_with("members = [") || line.starts_with("members=[") {
                // Парсим массив
                let array_start = line.find('[').unwrap();
                let array_end = line.rfind(']');

                if let Some(end) = array_end {
                    let array_content = &line[array_start + 1..end];
                    for member in array_content.split(',') {
                        let member = member.trim().trim_matches('"').trim_matches('\'');
                        if !member.is_empty() {
                            members.push(member.to_string());
                        }
                    }
                }
            } else if line.starts_with('"') || line.starts_with('\'') {
                // Многострочный формат
                let member = line.trim_matches('"').trim_matches('\'').trim();
                if !member.is_empty() && !member.starts_with('[') && !member.starts_with(']') {
                    members.push(member.to_string());
                }
            }
        }

        if !members.is_empty() {
            return Some(members);
        }
    }
    None
}

/// Получение имени крейта из Cargo.toml
fn get_crate_name(manifest_path: &Path) -> Option<String> {
    let content = fs::read_to_string(manifest_path).ok()?;

    for line in content.lines() {
        let line = line.trim();
        if line.starts_with("name = ") {
            let name = line
                .trim_start_matches("name = ")
                .trim_matches('"')
                .trim_matches('\'')
                .to_string();
            return Some(name);
        }
    }
    None
}

/// Получение имени пакета из пути
fn get_package_name_from_path(path: &str) -> Option<String> {
    let path = Path::new(path);
    let name = path.file_name()?.to_str()?;
    Some(name.to_string())
}
