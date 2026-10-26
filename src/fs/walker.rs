use std::{
    collections::HashSet,
    fs::read_to_string,
    path::{Path, PathBuf},
};

pub(crate) fn collect_used_rs_files(entry_point: &Path) -> HashSet<PathBuf> {
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
            Ok(c) => c,
            Err(_) => continue, // Skip missing or unreadable module files.
        };

        let syntax = match syn::parse_file(&content) {
            Ok(s) => s,
            Err(_) => {
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
            collect_mods_from_item(item, &base_dir, &mut stack);
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

fn collect_mods_from_item(item: &syn::Item, base_dir: &Path, stack: &mut Vec<PathBuf>) {
    if let syn::Item::Mod(item_mod) = item {
        // Inline modules have no source file, but their child modules still
        // need to be resolved relative to `base_dir/module_name/`.
        if let Some((_, items)) = &item_mod.content {
            let inline_dir = base_dir.join(item_mod.ident.to_string());
            for inner in items {
                collect_mods_from_item(inner, &inline_dir, stack);
            }
            return;
        }

        // A `#[path = "..."]` attribute overrides the module file path.
        let explicit_path = item_mod.attrs.iter().find_map(|attr| {
            if attr.path().is_ident("path") {
                if let syn::Meta::NameValue(nv) = &attr.meta {
                    if let syn::Expr::Lit(syn::ExprLit {
                        lit: syn::Lit::Str(s),
                        ..
                    }) = &nv.value
                    {
                        return Some(s.value());
                    }
                }
            }
            None
        });

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
