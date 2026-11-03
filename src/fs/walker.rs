use std::{
    collections::HashSet,
    fs::read_to_string,
    path::{Path, PathBuf},
};

use syn::{Meta, punctuated::Punctuated, token::Comma};

#[derive(Clone, Debug, Default)]
pub(crate) struct CfgSet {
    values: HashSet<String>,
}

impl CfgSet {
    pub(crate) fn from_rustc_output(
        output: &str,
        features: impl IntoIterator<Item = String>,
    ) -> Self {
        let mut values = output.lines().map(str::to_owned).collect::<HashSet<_>>();
        values.extend(
            features
                .into_iter()
                .map(|feature| format!("feature={feature:?}")),
        );
        Self { values }
    }

    fn is_enabled(&self, meta: &Meta) -> bool {
        match meta {
            Meta::Path(path) => self.values.contains(&path_name(path)),
            Meta::NameValue(name_value) => {
                let syn::Expr::Lit(expression) = &name_value.value else {
                    return false;
                };
                let value = match &expression.lit {
                    syn::Lit::Str(value) => format!("{:?}", value.value()),
                    syn::Lit::Bool(value) => value.value.to_string(),
                    syn::Lit::Int(value) => value.base10_digits().to_owned(),
                    _ => return false,
                };
                self.values
                    .contains(&format!("{}={value}", path_name(&name_value.path)))
            }
            Meta::List(list) => {
                let Ok(items) = list.parse_args_with(Punctuated::<Meta, Comma>::parse_terminated)
                else {
                    return false;
                };
                match list.path.get_ident().map(ToString::to_string).as_deref() {
                    Some("all") => items.iter().all(|item| self.is_enabled(item)),
                    Some("any") => items.iter().any(|item| self.is_enabled(item)),
                    Some("not") => items.len() == 1 && !self.is_enabled(&items[0]),
                    _ => false,
                }
            }
        }
    }
}

fn path_name(path: &syn::Path) -> String {
    path.segments
        .iter()
        .map(|segment| segment.ident.to_string())
        .collect::<Vec<_>>()
        .join("::")
}

pub(crate) fn collect_used_rs_files(entry_point: &Path, cfg: &CfgSet) -> HashSet<PathBuf> {
    let mut visited = HashSet::new();
    let mut stack = vec![entry_point.to_path_buf()];

    while let Some(file_path) = stack.pop() {
        let file_path = match file_path.canonicalize() {
            Ok(canonical_path) => canonical_path,
            Err(_) => file_path,
        };
        if visited.contains(&file_path) {
            continue;
        }

        let content = match read_to_string(&file_path) {
            Ok(content) => content,
            Err(error) => {
                tracing::warn!(
                    path = %file_path.display(),
                    %error,
                    "could not read Rust source file; skipping it"
                );
                continue;
            }
        };

        let syntax = match syn::parse_file(&content) {
            Ok(syntax) => syntax,
            Err(error) => {
                tracing::warn!(
                    path = %file_path.display(),
                    %error,
                    "could not parse Rust source file; child modules will not be traversed"
                );
                visited.insert(file_path);
                continue;
            }
        };

        visited.insert(file_path.clone());

        // Resolve child modules relative to this module's directory.
        let Some(base_dir) = module_dir_for(&file_path) else {
            continue;
        };

        for item in &syntax.items {
            collect_mods_from_item(item, &base_dir, cfg, &mut stack);
        }
    }

    visited
}

fn module_dir_for(file_path: &Path) -> Option<PathBuf> {
    let (parent, file_stem) = file_path.parent().zip(file_path.file_stem())?;
    let file_stem = file_stem.to_string_lossy();

    Some(
        if file_stem == "mod" || file_stem == "main" || file_stem == "lib" {
            parent.to_path_buf()
        } else {
            parent.join(file_stem.as_ref())
        },
    )
}

fn collect_mods_from_item(
    item: &syn::Item,
    base_dir: &Path,
    cfg: &CfgSet,
    stack: &mut Vec<PathBuf>,
) {
    if let syn::Item::Mod(item_mod) = item {
        if !attributes_enabled(&item_mod.attrs, cfg) {
            return;
        }
        // Inline modules have no source file, but their child modules still
        // need to be resolved relative to `base_dir/module_name/`.
        if let Some((_, items)) = &item_mod.content {
            let inline_dir = base_dir.join(item_mod.ident.to_string());
            for inner in items {
                collect_mods_from_item(inner, &inline_dir, cfg, stack);
            }
            return;
        }

        // A `#[path = "..."]` attribute overrides the module file path.
        let explicit_path = item_mod
            .attrs
            .iter()
            .filter(|attribute| attributes_enabled(std::slice::from_ref(*attribute), cfg))
            .find_map(|attribute| path_from_attribute(&attribute.meta, cfg));

        let mod_name = item_mod.ident.to_string();

        let candidate = if let Some(p) = explicit_path {
            base_dir.join(p)
        } else {
            // Prefer `foo.rs`, then fall back to `foo/mod.rs`.
            let as_file = base_dir.join(format!("{mod_name}.rs"));
            if as_file.exists() {
                as_file
            } else {
                base_dir.join(&mod_name).join("mod.rs")
            }
        };

        stack.push(candidate);
    }
}

fn attributes_enabled(attrs: &[syn::Attribute], cfg: &CfgSet) -> bool {
    attrs.iter().all(|attribute| match &attribute.meta {
        Meta::List(list) if list.path.is_ident("cfg") => list
            .parse_args::<Meta>()
            .is_ok_and(|condition| cfg.is_enabled(&condition)),
        Meta::List(list) if list.path.is_ident("cfg_attr") => list
            .parse_args_with(Punctuated::<Meta, Comma>::parse_terminated)
            .is_ok_and(|items| cfg_attr_enabled(&items, cfg)),
        _ => true,
    })
}

fn cfg_attr_enabled(items: &Punctuated<Meta, Comma>, cfg: &CfgSet) -> bool {
    let mut items = items.iter();
    let Some(condition) = items.next() else {
        return true;
    };
    !cfg.is_enabled(condition) || items.all(|attribute| meta_attribute_enabled(attribute, cfg))
}

fn meta_attribute_enabled(attribute: &Meta, cfg: &CfgSet) -> bool {
    match attribute {
        Meta::List(list) if list.path.is_ident("cfg") => list
            .parse_args::<Meta>()
            .is_ok_and(|condition| cfg.is_enabled(&condition)),
        Meta::List(list) if list.path.is_ident("cfg_attr") => list
            .parse_args_with(Punctuated::<Meta, Comma>::parse_terminated)
            .is_ok_and(|items| cfg_attr_enabled(&items, cfg)),
        _ => true,
    }
}

fn path_from_attribute(attribute: &Meta, cfg: &CfgSet) -> Option<String> {
    match attribute {
        Meta::NameValue(name_value) if name_value.path.is_ident("path") => {
            let syn::Expr::Lit(expression) = &name_value.value else {
                return None;
            };
            let syn::Lit::Str(path) = &expression.lit else {
                return None;
            };
            Some(path.value())
        }
        Meta::List(list) if list.path.is_ident("cfg_attr") => {
            let items = list
                .parse_args_with(Punctuated::<Meta, Comma>::parse_terminated)
                .ok()?;
            let mut items = items.iter();
            let condition = items.next()?;
            if !cfg.is_enabled(condition) {
                return None;
            }
            items.find_map(|attribute| path_from_attribute(attribute, cfg))
        }
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use std::{
        fs,
        path::PathBuf,
        sync::atomic::{AtomicUsize, Ordering},
    };

    use super::{CfgSet, Meta, collect_used_rs_files};

    static NEXT_TEST_FILE: AtomicUsize = AtomicUsize::new(0);

    fn test_directory() -> PathBuf {
        let directory = std::env::temp_dir().join(format!(
            "cargo-snapshot-walker-{}-{}",
            std::process::id(),
            NEXT_TEST_FILE.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir_all(&directory).expect("create test directory");
        directory
    }

    fn test_source_path() -> PathBuf {
        std::env::temp_dir().join(format!(
            "cargo-snapshot-walker-{}-{}.rs",
            std::process::id(),
            NEXT_TEST_FILE.fetch_add(1, Ordering::Relaxed)
        ))
    }

    #[test]
    fn skips_modules_with_inactive_feature_cfg() {
        let directory = test_directory();
        let entry = directory.join("main.rs");
        let conditional_module = directory.join("conditional.rs");
        fs::write(
            &entry,
            "#[cfg(feature = \"disabled_feature\")] mod conditional;\n",
        )
        .expect("write test entry point");
        fs::write(&conditional_module, "pub fn unavailable() {}\n")
            .expect("write conditional module");

        let cfg = CfgSet::from_rustc_output("", Vec::new());
        let files = collect_used_rs_files(&entry, &cfg);

        assert!(
            !files.contains(
                &conditional_module
                    .canonicalize()
                    .expect("canonicalize inactive conditional module")
            )
        );
        fs::remove_dir_all(directory).expect("remove test directory");
    }

    #[test]
    fn includes_modules_with_active_feature_cfg() {
        let directory = test_directory();
        let entry = directory.join("main.rs");
        let conditional_module = directory.join("conditional.rs");
        fs::write(
            &entry,
            "#[cfg(feature = \"enabled_feature\")] mod conditional;\n",
        )
        .expect("write test entry point");
        fs::write(&conditional_module, "pub fn available() {}\n")
            .expect("write conditional module");
        let cfg = CfgSet::from_rustc_output("", ["enabled_feature".to_owned()]);

        let files = collect_used_rs_files(&entry, &cfg);

        assert!(
            files.contains(
                &conditional_module
                    .canonicalize()
                    .expect("canonicalize active conditional module")
            )
        );
        fs::remove_dir_all(directory).expect("remove test directory");
    }

    #[test]
    fn evaluates_composite_cfg_conditions() {
        let cfg = CfgSet::from_rustc_output("unix\ntarget_arch=\"x86_64\"", []);
        let condition: Meta = syn::parse_quote!(all(unix, not(target_os = "windows")));

        assert!(cfg.is_enabled(&condition));
    }

    #[test]
    fn resolves_path_attribute_from_active_cfg_attr() {
        let directory = test_directory();
        let entry = directory.join("main.rs");
        let unix_module = directory.join("unix_impl.rs");
        let other_module = directory.join("fallback.rs");
        fs::write(
            &entry,
            "#[cfg_attr(unix, path = \"unix_impl.rs\")] mod platform;\n",
        )
        .expect("write test entry point");
        fs::write(&unix_module, "pub fn unix() {}\n").expect("write unix module");
        fs::write(&other_module, "pub fn fallback() {}\n").expect("write fallback module");
        let cfg = CfgSet::from_rustc_output("unix", []);

        let files = collect_used_rs_files(&entry, &cfg);

        assert!(
            files.contains(
                &unix_module
                    .canonicalize()
                    .expect("canonicalize unix module")
            )
        );
        assert!(
            !files.contains(
                &other_module
                    .canonicalize()
                    .expect("canonicalize fallback module")
            )
        );
        fs::remove_dir_all(directory).expect("remove test directory");
    }

    #[test]
    fn skips_unreadable_source_file() {
        let file = test_source_path();
        let files = collect_used_rs_files(&file, &CfgSet::default());

        assert!(files.is_empty());
    }

    #[test]
    fn keeps_unparseable_file_but_does_not_traverse_it() {
        let file = test_source_path();
        fs::write(&file, "mod broken {\n").expect("create invalid Rust source");

        let files = collect_used_rs_files(&file, &CfgSet::default());
        let canonical_file = file.canonicalize().expect("canonicalize test source");

        assert_eq!(files.len(), 1);
        assert!(files.contains(&canonical_file));
        fs::remove_file(file).expect("remove test source");
    }
}
