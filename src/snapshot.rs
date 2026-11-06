use std::{
    collections::HashSet,
    fs::{self, File},
    io::{BufWriter, Write},
    path::{Path, PathBuf},
    process::{Command, Stdio},
};

use cargo_metadata::{CargoOpt, DependencyKind, Message, Metadata, MetadataCommand, PackageId};

use crate::{
    cli::args::Args,
    config::{settings::Config, tracing::init_tracing},
    error::SnapshotError,
    fs::walker::{CfgSet, collect_project_rs_files, collect_used_rs_files},
};

pub(crate) struct Snapshot {
    pub config: Config,
    pub metadata: Metadata,
    cfg: CfgSet,
}

impl Snapshot {
    pub(crate) fn new() -> Result<Self, SnapshotError> {
        let args = Args::get();
        init_tracing(&args);
        let config = Config::new(&args);
        let target = args
            .target
            .clone()
            .or_else(|| std::env::var("CARGO_BUILD_TARGET").ok());
        let mut metadata_command = MetadataCommand::new();
        if args.all_features {
            metadata_command.features(CargoOpt::AllFeatures);
        }
        if args.no_default_features {
            metadata_command.features(CargoOpt::NoDefaultFeatures);
        }
        if !args.features.is_empty() {
            metadata_command.features(CargoOpt::SomeFeatures(args.features.clone()));
        }
        if let Some(target) = &target {
            metadata_command.other_options(vec!["--filter-platform".into(), target.clone()]);
        }
        let metadata = metadata_command.exec()?;
        let features = active_root_features(&metadata);
        let build_script_cfgs = active_root_build_script_cfgs(&metadata, &args, target.as_deref())?;
        let cfg = rustc_cfg(target.as_deref(), features, build_script_cfgs)?;
        Ok(Self {
            config,
            metadata,
            cfg,
        })
    }

    fn find_entry_points(&self) -> Vec<PathBuf> {
        let Some(root_pkg) = self.metadata.root_package() else {
            return Vec::new();
        };

        root_pkg
            .targets
            .iter()
            .filter(|target| {
                target
                    .kind
                    .iter()
                    .any(|kind| matches!(kind.to_string().as_str(), "bin" | "lib" | "proc-macro"))
            })
            .map(|target| PathBuf::from(&target.src_path))
            .collect()
    }

    fn src_files(&self) -> Vec<PathBuf> {
        let mut all_used = HashSet::new();

        for entry in self.find_entry_points() {
            all_used.extend(collect_used_rs_files(&entry, &self.cfg));
        }

        all_used.into_iter().collect()
    }

    fn sorted_sources(&self) -> Vec<PathBuf> {
        let mut sources = self.src_files();
        sources.sort_by(|a, b| {
            a.parent()
                .cmp(&b.parent())
                .then_with(|| a.file_name().cmp(&b.file_name()))
        });
        sources
    }

    fn write_dependencies<W: Write>(&self, writer: &mut W) -> Result<(), SnapshotError> {
        let Some(package) = self.metadata.root_package() else {
            return Err(SnapshotError::NoRootPackage);
        };
        let mut dependencies = package.dependencies.iter().collect::<Vec<_>>();
        dependencies.sort_by(|left, right| left.name.cmp(&right.name));

        writeln!(writer, "## Dependencies")?;
        writeln!(writer)?;
        if dependencies.is_empty() {
            writeln!(writer, "No dependencies.")?;
        } else {
            for dependency in dependencies {
                let kind = match dependency.kind {
                    DependencyKind::Normal => "normal",
                    DependencyKind::Development => "dev",
                    DependencyKind::Build => "build",
                    DependencyKind::Unknown => "unknown",
                };
                let optional = if dependency.optional {
                    ", optional"
                } else {
                    ""
                };
                writeln!(
                    writer,
                    "- `{}` `{}` ({kind}{optional})",
                    dependency.name, dependency.req
                )?;
            }
        }
        writeln!(writer)?;
        Ok(())
    }

    fn write_readme<W: Write>(
        &self,
        writer: &mut W,
        workspace_root: &Path,
        root_name: &str,
    ) -> Result<(), SnapshotError> {
        let Some(package) = self.metadata.root_package() else {
            return Err(SnapshotError::NoRootPackage);
        };
        let manifest = PathBuf::from(package.manifest_path.as_str());
        let package_root = match manifest.parent() {
            Some(parent) => parent,
            None => workspace_root,
        };
        let readme_path = match &package.readme {
            Some(readme) => PathBuf::from(readme.as_str()),
            None => PathBuf::from("README.md"),
        };
        let readme_path = if readme_path.is_absolute() {
            readme_path
        } else {
            package_root.join(readme_path)
        };
        if !readme_path.is_file() {
            return Ok(());
        }

        let content = fs::read_to_string(&readme_path)?;
        let relative_path = match readme_path.strip_prefix(workspace_root) {
            Ok(relative) => relative,
            Err(_) => match readme_path.strip_prefix(package_root) {
                Ok(relative) => relative,
                Err(_) => &readme_path,
            },
        };
        let relative = relative_path.to_string_lossy().replace('\\', "/");
        let display_path = format!("{root_name}/{relative}");
        let fence = markdown_fence(&content);

        writeln!(writer, "## README: `{display_path}`")?;
        writeln!(writer)?;
        writeln!(writer, "{fence}markdown")?;
        write!(writer, "{content}")?;
        if !content.ends_with('\n') {
            writeln!(writer)?;
        }
        writeln!(writer, "{fence}")?;
        writeln!(writer)?;
        Ok(())
    }

    pub(crate) fn write_sources_md<P: AsRef<Path>>(&self, output: P) -> Result<(), SnapshotError> {
        let output = output.as_ref();
        let sources = self.sorted_sources();
        let root = Path::new(self.metadata.workspace_root.as_str());
        let project_sources = collect_project_rs_files(root)?;
        ensure_output_is_not_source(output, &project_sources)?;

        let file = File::create(output)?;
        let output_path = output.canonicalize().ok();
        let mut writer = BufWriter::new(file);
        let root = Path::new(self.metadata.workspace_root.as_str()).canonicalize()?;
        let root_name = root
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or("project");

        writeln!(writer, "# Project snapshot: {root_name}")?;
        writeln!(writer)?;
        self.write_dependencies(&mut writer)?;
        if self.config.readme {
            self.write_readme(&mut writer, &root, root_name)?;
        }
        writeln!(writer, "## Project tree")?;
        writeln!(writer)?;
        writeln!(writer, "```text")?;
        writeln!(writer, "{root_name}/")?;
        write_tree(&mut writer, &root, "", output_path.as_deref())?;
        writeln!(writer, "```")?;
        writeln!(writer)?;
        writeln!(writer, "## Used Rust files")?;
        writeln!(writer)?;

        for source in sources {
            let source_content = fs::read_to_string(&source)?;
            let content = if self.config.no_comments {
                strip_rust_comments(&source_content)
            } else {
                source_content
            };
            let relative = match source.strip_prefix(&root) {
                Ok(relative) => relative,
                Err(_) => &source,
            };
            let relative = format!(
                "{}/{}",
                root_name,
                relative.to_string_lossy().replace('\\', "/")
            );
            let fence = markdown_fence(&content);

            writeln!(writer, "### `{relative}`")?;
            writeln!(writer)?;
            writeln!(writer, "{fence}rust")?;
            write!(writer, "{content}")?;
            if !content.ends_with('\n') {
                writeln!(writer)?;
            }
            writeln!(writer, "{fence}")?;
            writeln!(writer)?;
        }

        writer.flush()?;
        Ok(())
    }
}

fn ensure_output_is_not_source(
    output: &Path,
    source_files: &HashSet<PathBuf>,
) -> Result<(), SnapshotError> {
    let output = canonical_output_path(output)?;

    for source in source_files {
        if source.canonicalize()? == output {
            return Err(SnapshotError::OutputIsSource(source.clone()));
        }
    }

    Ok(())
}

fn canonical_output_path(output: &Path) -> Result<PathBuf, std::io::Error> {
    if let Ok(path) = output.canonicalize() {
        return Ok(path);
    }

    let file_name = output.file_name().ok_or_else(|| {
        std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            "output path must name a file",
        )
    })?;
    let parent = output
        .parent()
        .filter(|parent| !parent.as_os_str().is_empty())
        .unwrap_or_else(|| Path::new("."));

    Ok(parent.canonicalize()?.join(file_name))
}

fn active_root_features(metadata: &Metadata) -> Vec<String> {
    let Some(package) = metadata.root_package() else {
        return Vec::new();
    };
    let Some(resolve) = &metadata.resolve else {
        return Vec::new();
    };

    resolve
        .nodes
        .iter()
        .find(|node| node.id == package.id)
        .map(|node| node.features.iter().map(ToString::to_string).collect())
        .unwrap_or_default()
}

fn active_root_build_script_cfgs(
    metadata: &Metadata,
    args: &Args,
    target: Option<&str>,
) -> Result<Vec<String>, SnapshotError> {
    let Some(package) = metadata.root_package() else {
        return Err(SnapshotError::NoRootPackage);
    };

    run_build_script_cfgs(
        std::env::var_os("CARGO").unwrap_or_else(|| "cargo".into()),
        package.manifest_path.as_std_path(),
        &package.id,
        args,
        target,
    )
}

fn run_build_script_cfgs(
    cargo: impl AsRef<std::ffi::OsStr>,
    manifest_path: &Path,
    package_id: &PackageId,
    args: &Args,
    target: Option<&str>,
) -> Result<Vec<String>, SnapshotError> {
    let mut command = Command::new(cargo);
    command
        .arg("check")
        .arg("--message-format=json")
        .arg("--manifest-path")
        .arg(manifest_path);
    if args.all_features {
        command.arg("--all-features");
    }
    if args.no_default_features {
        command.arg("--no-default-features");
    }
    if !args.features.is_empty() {
        command.arg("--features").arg(args.features.join(","));
    }
    if let Some(target) = target {
        command.arg("--target").arg(target);
    }

    let output = command
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()?;
    let mut cfgs = Vec::new();
    for message in Message::parse_stream(output.stdout.as_slice()) {
        if let Message::BuildScriptExecuted(build_script) = message?
            && &build_script.package_id == package_id
        {
            cfgs.extend(build_script.cfgs);
        }
    }

    if !output.status.success() {
        return Err(SnapshotError::CargoCheck(
            String::from_utf8_lossy(&output.stderr).trim().to_owned(),
        ));
    }

    Ok(cfgs)
}

fn rustc_cfg(
    target: Option<&str>,
    features: Vec<String>,
    build_script_cfgs: Vec<String>,
) -> Result<CfgSet, SnapshotError> {
    let mut command = Command::new("rustc");
    command.arg("--print").arg("cfg");
    if let Some(target) = target {
        command.arg("--target").arg(target);
    }

    let output = command.output()?;
    if !output.status.success() {
        return Err(SnapshotError::RustcCfg(
            String::from_utf8_lossy(&output.stderr).trim().to_owned(),
        ));
    }

    Ok(CfgSet::from_rustc_output(
        &String::from_utf8_lossy(&output.stdout),
        features,
        build_script_cfgs,
    ))
}

fn write_tree<W: Write>(
    writer: &mut W,
    directory: &Path,
    prefix: &str,
    output_path: Option<&Path>,
) -> Result<(), SnapshotError> {
    let mut entries = fs::read_dir(directory)?.collect::<Result<Vec<_>, _>>()?;
    entries.sort_by_key(fs::DirEntry::file_name);
    entries.retain(|entry| {
        let name = entry.file_name();
        if name == ".git" || name == "target" {
            return false;
        }

        output_path.is_none_or(|output| entry.path().canonicalize().ok().as_deref() != Some(output))
    });

    for (index, entry) in entries.iter().enumerate() {
        let is_last = index + 1 == entries.len();
        let connector = if is_last { "└── " } else { "├── " };
        let path = entry.path();
        let file_type = entry.file_type()?;
        let is_directory = file_type.is_dir();
        let name = entry.file_name().to_string_lossy().into_owned();

        writeln!(
            writer,
            "{prefix}{connector}{name}{}",
            if is_directory { "/" } else { "" }
        )?;

        if is_directory {
            let next_prefix = format!("{prefix}{}", if is_last { "    " } else { "│   " });
            write_tree(writer, &path, &next_prefix, output_path)?;
        }
    }

    Ok(())
}

fn strip_rust_comments(source: &str) -> String {
    let mut bytes = source.as_bytes().to_vec();
    let mut index = 0;

    while index < bytes.len() {
        if let Some(raw_string_end) = raw_string_end(&bytes, index) {
            index = raw_string_end;
            continue;
        }

        match bytes[index] {
            b'"' => index = quoted_string_end(&bytes, index, b'"'),
            b'\'' => {
                if let Some(end) = char_literal_end(&bytes, index) {
                    index = end;
                } else {
                    index += 1;
                }
            }
            b'/' if bytes.get(index + 1) == Some(&b'/') => {
                let mut end = index + 2;
                while end < bytes.len() && bytes[end] != b'\n' && bytes[end] != b'\r' {
                    end += 1;
                }
                blank_comment(&mut bytes[index..end]);
                index = end;
            }
            b'/' if bytes.get(index + 1) == Some(&b'*') => {
                let mut end = index + 2;
                let mut depth = 1usize;
                while end < bytes.len() && depth > 0 {
                    if bytes.get(end..end + 2) == Some(b"/*") {
                        depth += 1;
                        end += 2;
                    } else if bytes.get(end..end + 2) == Some(b"*/") {
                        depth -= 1;
                        end += 2;
                    } else {
                        end += 1;
                    }
                }
                blank_comment(&mut bytes[index..end]);
                index = end;
            }
            _ => index += 1,
        }
    }

    let stripped = String::from_utf8_lossy(&bytes);
    compact_blank_lines(&stripped)
}

fn compact_blank_lines(source: &str) -> String {
    let mut compacted = String::with_capacity(source.len());
    let mut pending_blank_line = None;

    for line in source.split_inclusive('\n') {
        if line.trim().is_empty() {
            if !compacted.is_empty() && pending_blank_line.is_none() {
                pending_blank_line = Some(if line.ends_with("\r\n") { "\r\n" } else { "\n" });
            }
            continue;
        }

        if let Some(line_ending) = pending_blank_line.take() {
            compacted.push_str(line_ending);
        }
        compacted.push_str(line);
    }

    compacted
}

fn raw_string_end(bytes: &[u8], start: usize) -> Option<usize> {
    let mut index = start;
    if bytes.get(index) == Some(&b'b') {
        index += 1;
    }
    if bytes.get(index) != Some(&b'r') {
        return None;
    }
    index += 1;

    let mut hashes = 0;
    while bytes.get(index) == Some(&b'#') {
        hashes += 1;
        index += 1;
    }
    if bytes.get(index) != Some(&b'"') {
        return None;
    }
    index += 1;

    while index < bytes.len() {
        if bytes[index] == b'"'
            && (0..hashes).all(|offset| bytes.get(index + 1 + offset) == Some(&b'#'))
        {
            return Some(index + hashes + 1);
        }
        index += 1;
    }
    Some(bytes.len())
}

fn quoted_string_end(bytes: &[u8], start: usize, quote: u8) -> usize {
    let mut index = start + 1;
    while index < bytes.len() {
        match bytes[index] {
            b'\\' => index = (index + 2).min(bytes.len()),
            character if character == quote => return index + 1,
            _ => index += 1,
        }
    }
    bytes.len()
}

fn char_literal_end(bytes: &[u8], start: usize) -> Option<usize> {
    let content_start = start + 1;
    let content = *bytes.get(content_start)?;
    if content == b'\\' {
        let end = quoted_string_end(bytes, start, b'\'');
        return (end > content_start + 1 && bytes.get(end - 1) == Some(&b'\'')).then_some(end);
    }

    let character = source_char_len(bytes, content_start)?;
    let end = content_start + character;
    (bytes.get(end) == Some(&b'\'')).then_some(end + 1)
}

fn source_char_len(bytes: &[u8], index: usize) -> Option<usize> {
    let tail = std::str::from_utf8(bytes.get(index..)?).ok()?;
    tail.chars().next().map(char::len_utf8)
}

fn blank_comment(comment: &mut [u8]) {
    for byte in comment {
        if *byte != b'\n' && *byte != b'\r' {
            *byte = b' ';
        }
    }
}

fn markdown_fence(content: &str) -> String {
    let longest_run = content
        .split('\n')
        .map(|line| {
            let mut longest = 0;
            let mut current = 0;
            for character in line.chars() {
                if character == '`' {
                    current += 1;
                    longest = longest.max(current);
                } else {
                    current = 0;
                }
            }
            longest
        })
        .max()
        .unwrap_or(0);

    "`".repeat(3.max(longest_run + 1))
}

#[cfg(test)]
mod tests {
    use std::{
        fs,
        path::PathBuf,
        sync::atomic::{AtomicUsize, Ordering},
    };

    use super::{ensure_output_is_not_source, run_build_script_cfgs, strip_rust_comments};
    use crate::{cli::args::Args, fs::walker::collect_used_rs_files};
    use cargo_metadata::{MetadataCommand, PackageId};
    use clap::Parser;
    use std::collections::HashSet;

    static NEXT_TEST_FILE: AtomicUsize = AtomicUsize::new(0);

    #[test]
    fn includes_custom_cfg_emitted_by_root_build_script() {
        let directory = std::env::temp_dir().join(format!(
            "cargo-snapshot-build-cfg-{}-{}",
            std::process::id(),
            NEXT_TEST_FILE.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir_all(directory.join("src")).expect("create package source directory");
        fs::write(
            directory.join("Cargo.toml"),
            "[package]\nname = \"build-cfg-fixture\"\nversion = \"0.1.0\"\nedition = \"2024\"\nbuild = \"build.rs\"\n",
        )
        .expect("write fixture manifest");
        fs::write(
            directory.join("build.rs"),
            "fn main() { println!(\"cargo:rustc-check-cfg=cfg(custom_mode)\"); println!(\"cargo:rustc-cfg=custom_mode\"); }\n",
        )
        .expect("write fixture build script");
        fs::write(
            directory.join("src/main.rs"),
            "#[cfg(custom_mode)] mod custom;\nfn main() {}\n",
        )
        .expect("write fixture entry point");
        fs::write(directory.join("src/custom.rs"), "pub fn enabled() {}\n")
            .expect("write custom module");

        let metadata = MetadataCommand::new()
            .manifest_path(directory.join("Cargo.toml"))
            .exec()
            .expect("read fixture metadata");
        let package_id: PackageId = metadata
            .root_package()
            .expect("fixture root package")
            .id
            .clone();
        let args = Args::parse_from(["cargo-snapshot"]);
        let cfgs = run_build_script_cfgs(
            std::env::var_os("CARGO").unwrap_or_else(|| "cargo".into()),
            &directory.join("Cargo.toml"),
            &package_id,
            &args,
            None,
        )
        .expect("collect custom build-script cfg");
        let cfg = super::CfgSet::from_rustc_output("", [], cfgs);
        let files = collect_used_rs_files(&directory.join("src/main.rs"), &cfg);

        assert!(
            files.contains(
                &directory
                    .join("src/custom.rs")
                    .canonicalize()
                    .expect("canonicalize custom module")
            )
        );
        fs::remove_dir_all(directory).expect("remove fixture package");
    }

    #[test]
    fn rejects_output_that_is_a_source_file_without_modifying_it() {
        let file = std::env::temp_dir().join(format!(
            "cargo-snapshot-source-{}-{}.rs",
            std::process::id(),
            NEXT_TEST_FILE.fetch_add(1, Ordering::Relaxed)
        ));
        fs::write(&file, "fn original() {}\n").expect("create test source");

        let result = ensure_output_is_not_source(&file, &HashSet::from([file.clone()]));

        assert!(matches!(
            result,
            Err(crate::error::SnapshotError::OutputIsSource(path)) if path == file
        ));
        assert_eq!(
            fs::read_to_string(&file).expect("read test source"),
            "fn original() {}\n"
        );
        fs::remove_file(file).expect("remove test source");
    }

    #[test]
    fn allows_output_that_is_not_a_source_file() {
        let source = PathBuf::from("src/main.rs");
        let output = PathBuf::from("snapshot.md");

        assert!(ensure_output_is_not_source(&output, &HashSet::from([source])).is_ok());
    }

    #[test]
    fn rejects_unrelated_rust_file_as_output() {
        let directory = std::env::temp_dir().join(format!(
            "cargo-snapshot-output-{}-{}",
            std::process::id(),
            NEXT_TEST_FILE.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir_all(&directory).expect("create test directory");
        let integration_test = directory.join("tests").join("integration.rs");
        fs::create_dir_all(integration_test.parent().expect("test parent"))
            .expect("create tests directory");
        fs::write(&integration_test, "fn integration_test() {}\n")
            .expect("create unrelated Rust source");

        let sources = HashSet::from([integration_test.clone()]);
        let result = ensure_output_is_not_source(&integration_test, &sources);

        assert!(matches!(
            result,
            Err(crate::error::SnapshotError::OutputIsSource(path)) if path == integration_test
        ));
        assert_eq!(
            fs::read_to_string(&integration_test).expect("read integration source"),
            "fn integration_test() {}\n"
        );
        fs::remove_dir_all(directory).expect("remove test directory");
    }

    #[test]
    fn removes_comments_but_preserves_string_literals() {
        let source = r####"let text = "// not a comment /* either */";
let raw = r###"// still text /* still text */"###; // remove this
"####;
        let stripped = strip_rust_comments(source);

        assert!(stripped.contains(r#""// not a comment /* either */""#));
        assert!(stripped.contains("r###\"// still text /* still text */\"###"));
        assert!(!stripped.contains("remove this"));
    }

    #[test]
    fn collapses_blank_lines_after_removing_comments() {
        let stripped =
            strip_rust_comments("fn first() {}\n// comment\n// another\n\nfn second() {}\n");

        assert_eq!(stripped, "fn first() {}\n\nfn second() {}\n");
    }

    #[test]
    fn removes_nested_block_comments_and_keeps_newlines() {
        let stripped = strip_rust_comments("before /* outer /* inner */ end */ after\nnext");

        assert!(stripped.contains("before"));
        assert!(stripped.contains("after\nnext"));
        assert!(!stripped.contains("outer"));
        assert_eq!(stripped.lines().count(), 2);
    }
}
