/// Конфигурация snapshot'а
pub struct SnapshotConfig {
    /// Игнорировать ли папку target
    exclude_target: bool,
    /// Дополнительные паттерны для исключения
    exclude_patterns: Vec<String>,
    /// Максимальный размер файла в байтах
    max_file_size: u64,
    /// Включать ли .gitignore
    respect_gitignore: bool,
}

impl Default for SnapshotConfig {
    fn default() -> Self {
        Self {
            exclude_target: true,
            exclude_patterns: vec!["*.rs.bk".to_string(), "Cargo.lock".to_string()],
            max_file_size: 1024 * 1024, // 1MB
            respect_gitignore: true,
        }
    }
}
