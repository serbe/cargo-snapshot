use std::env::current_dir;

mod config;
mod project;

fn main() {
    let current_dir = current_dir().expect("Не удалось получить текущий каталог");

    // match analyze_directory(&current_dir) {
    //     Ok(result) => {
    //         println!("Анализ каталога: {}", current_dir.display());
    //         println!("=================================");
    //         println!("Является крейтом: {}", result.is_crate);
    //         println!("Часть воркспейса: {}", result.is_part_of_workspace);
    //         println!("Корень воркспейса: {}", result.is_workspace_root);

    //         if !result.workspace_members.is_empty() {
    //             println!("\nЧлены воркспейса ({})", result.workspace_members.len());
    //             for member in result.workspace_members {
    //                 println!("  • {} -> {}", member.name, member.path.display());
    //             }
    //         }

    //         if let Some(path) = result.manifest_path {
    //             println!("\nМанифест: {}", path.display());
    //         }
    //     }
    //     Err(e) => eprintln!("Ошибка: {}", e),
    // }
}
