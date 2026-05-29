// ----- src\client.rs -----

use std::time::Duration;

use reqwest::Client;

use crate::{config::WorkerConfig, error::DaemonResult};

#[derive(Debug, Clone)]
pub struct ClientConfig {
    pub connect_timeout_secs: u64,
    pub timeout_secs: u64,
    pub proxy_port: Option<u16>,
}

impl ClientConfig {
    pub fn with_proxy(config: &WorkerConfig, port: u16) -> Self {
        Self {
            connect_timeout_secs: config.connect_timeout_secs,
            timeout_secs: config.request_timeout_secs,
            proxy_port: Some(port),
        }
    }

    pub fn direct(config: &WorkerConfig) -> Self {
        Self {
            connect_timeout_secs: config.connect_timeout_secs,
            timeout_secs: config.request_timeout_secs,
            proxy_port: None,
        }
    }

    pub fn build_client(&self) -> DaemonResult<Client> {
        let mut builder = Client::builder();

        if let Some(port) = self.proxy_port {
            let proxy_url = format!("socks5://127.0.0.1:{}", port);
            builder = builder.proxy(reqwest::Proxy::all(proxy_url)?);
        }

        Ok(builder
            .connect_timeout(Duration::from_secs(self.connect_timeout_secs))
            .timeout(Duration::from_secs(self.timeout_secs))
            .build()?)
    }
}

// pub fn build_client(
//     port: Option<u16>,
//     connect_timeout_secs: u64,
//     timeout_secs: u64,
// ) -> DaemonResult<Client> {
//     let mut c_builder = Client::builder();
//     if let Some(port) = port {
//         let proxy_url = format!("socks5://127.0.0.1:{}", port);

//         c_builder = c_builder.proxy(reqwest::Proxy::all(proxy_url)?);
//     }

//     Ok(c_builder
//         .connect_timeout(Duration::from_secs(connect_timeout_secs))
//         .timeout(Duration::from_secs(timeout_secs))
//         .build()?)
// }

// ----- src\config.rs -----

use std::path::Path;

use serde::Deserialize;
use tokio::fs;

use crate::error::{DaemonError, DaemonResult};

#[derive(Clone, Debug, Deserialize)]
pub struct WorkerConfig {
    pub task_timeout_secs: u64,
    pub connect_timeout_secs: u64,
    pub max_concurrent_requests: usize,
    pub request_timeout_secs: u64,
    pub xray_path: String,
    pub urls: Vec<String>,
}

#[derive(Clone, Debug, Deserialize)]
pub struct ManagerConfig {
    pub max_queue: usize,
    pub max_concurrent_tasks: usize,
    pub port_range_start: u16,
    pub port_count: usize,

    #[serde(flatten)]
    pub worker: WorkerConfig,
}

#[derive(Clone, Debug, Deserialize)]
pub struct AppConfig {
    pub subscription: String,
    pub db_path: String,
    #[serde(flatten)]
    pub manager: ManagerConfig,
}

impl AppConfig {
    pub async fn from_file<P: AsRef<Path>>(path: P) -> DaemonResult<Self> {
        let content = fs::read_to_string(path.as_ref()).await?;
        let settings: Self = toml::from_str(&content)?;

        settings.validate()?;

        Ok(settings)
    }

    pub async fn load() -> DaemonResult<Self> {
        // Пробуем стандартные пути
        let config_paths = [
            "config.toml",
            "settings.toml",
            "./config/config.toml",
            "./config/settings.toml",
        ];

        for path in config_paths {
            if Path::new(path).exists() {
                return Self::from_file(path).await;
            }
        }

        Err(DaemonError::NoConfig(format!("{:?}", config_paths)))
    }

    pub fn validate(&self) -> DaemonResult<()> {
        if self.db_path.is_empty() {
            return Err(DaemonError::EmptyDbPath);
        }
        if self.subscription.is_empty() {
            return Err(DaemonError::EmptySubscription);
        }
        if self.manager.max_concurrent_tasks == 0 {
            return Err(DaemonError::WrongMaxTasks);
        }
        if self.manager.worker.max_concurrent_requests == 0 {
            return Err(DaemonError::WrongMaxRequests);
        }
        if self.manager.port_count == 0 {
            return Err(DaemonError::WrongPortCount);
        }
        if self.manager.worker.urls.is_empty() {
            return Err(DaemonError::EmptyUrls);
        }
        if !Path::new(&self.manager.worker.xray_path).exists() {
            return Err(DaemonError::XrayNotFound(
                self.manager.worker.xray_path.clone(),
            ));
        }
        Ok(())
    }
}

// ----- src\error.rs -----

use thiserror::Error;

#[derive(Debug, Error)]
pub enum DaemonError {
    #[error("No config file found. Tried paths: {0}")]
    NoConfig(String),

    #[error("subscription cannot be empty")]
    EmptySubscription,

    #[error("db_path cannot be empty")]
    EmptyDbPath,

    #[error("max_concurrent_tasks must be > 0")]
    WrongMaxTasks,

    #[error("max_concurrent_requests must be > 0")]
    WrongMaxRequests,

    #[error("port_count must be > 0")]
    WrongPortCount,

    #[error("urls cannot be empty")]
    EmptyUrls,

    #[error("XRay binary not found on path: {0}")]
    XrayNotFound(String),

    #[error("Proxy process died unexpectedly")]
    ProxyDied,

    #[error("Failed to convert u16 to usize: {0}")]
    U16ToUsizeConversion(#[from] std::num::TryFromIntError),

    #[error("Timeout error: {0}")]
    Timeout(#[from] tokio::time::error::Elapsed),

    #[error("Error conver to json: {0}")]
    ToJson(#[from] xray_parser::ParserError),

    #[error("Failed to parse TOML config: {source}")]
    ConfigParse {
        #[from]
        source: toml::de::Error,
    },

    #[error("Network error: {0}")]
    Network(#[from] reqwest::Error),

    #[error("std io error: {0}")]
    StdIO(#[from] std::io::Error),

    #[error("db error: {0}")]
    DbError(#[from] xray_db::DbError),

    #[error("channel closed")]
    ChannelClosed,

    #[error("task join error: {0}")]
    Join(#[from] tokio::task::JoinError),

    #[error("decode base64 error: {0}")]
    Base64Decode(#[from] base64::DecodeError),

    #[error("decode utf8 error: {0}")]
    Utf8Decode(#[from] std::string::FromUtf8Error),

    #[error("No available ports in pool")]
    NoAvailablePorts,

    #[error("Invalid subscription: {0}")]
    InvalidSubscriptionSource(String),
}

pub type DaemonResult<T> = Result<T, DaemonError>;

// ----- src\main.rs -----

use tracing::{error, info};
use xray_db::Database;

use crate::{config::AppConfig, error::DaemonResult, manager::Manager, subscription::Subscription};

mod client;
mod config;
mod error;
mod manager;
mod port_pool;
mod proxy;
// mod retry;
mod result;
mod subscription;
mod task;
mod utils;
mod worker;

#[tokio::main]
async fn main() -> DaemonResult<()> {
    tracing_subscriber::fmt()
        .with_env_filter(std::env::var("RUST_LOG").unwrap_or_else(|_| "info".to_string()))
        .init();

    // Загружаем конфиг
    let config = AppConfig::load().await?;
    info!("Starting with config: {:?}", config);

    let db = Database::new(&config.db_path).await?;

    let worker_config = config.manager.worker.clone();

    let manager = Manager::start(config.manager).await?;
    info!("Manager started");

    let nodes = Subscription::new(&config.subscription, worker_config)
        .get_nodes()
        .await?;

    for node in nodes {
        if let Err(e) = manager.submit_task(&node).await {
            error!("Failed to submit task for node {}: {}", node.uuid, e);
        }
    }

    // Ждём сигнал завершения
    tokio::signal::ctrl_c().await?;
    info!("Shutting down...");

    manager.shutdown();

    Ok(())
}

// ----- src\manager.rs -----

use std::sync::atomic::{AtomicU64, Ordering};

use tokio::sync::mpsc::{Sender, channel};
use tokio_util::sync::CancellationToken;
use xray_parser::XRayNode;

use crate::{
    config::ManagerConfig,
    error::{DaemonError, DaemonResult},
    task::NodeTask,
    worker::Worker,
};

pub struct Manager {
    tx: Sender<NodeTask>,
    shutdown: CancellationToken,
    task_counter: AtomicU64,
}

impl Manager {
    pub async fn start(config: ManagerConfig) -> DaemonResult<Self> {
        let (tx, rx) = channel::<NodeTask>(config.max_queue);
        let shutdown = CancellationToken::new();
        let task_counter = AtomicU64::new(0);

        let worker = Worker::new(rx, config, shutdown.clone())?;

        // Запускаем воркер в отдельной задаче
        tokio::spawn(worker.run());

        Ok(Self {
            tx,
            shutdown,
            task_counter,
        })
    }

    pub async fn submit_task(&self, node: &XRayNode) -> DaemonResult<()> {
        let task = NodeTask {
            id: self.task_counter.fetch_add(1, Ordering::Relaxed),
            node: node.clone(),
        };

        self.tx
            .send(task)
            .await
            .map_err(|_| DaemonError::ChannelClosed)?;

        Ok(())
    }

    pub fn shutdown(&self) {
        self.shutdown.cancel();
    }
}

// ----- src\port_pool.rs -----

use std::collections::VecDeque;
use std::ops::Range;
use std::sync::Arc;
use tokio::sync::Mutex;
use tracing::debug;

use crate::error::{DaemonError, DaemonResult};

#[derive(Debug)]
pub struct PortPool {
    available: Mutex<VecDeque<u16>>,
    range: Range<u16>, // Для проверки при возврате
}

/// Guard, который автоматически возвращает порт при Drop
pub struct PortLease {
    port: u16,
    pool: Arc<PortPool>,
    returned: bool,
}

impl PortLease {
    pub fn port(&self) -> u16 {
        self.port
    }

    /// Ручной возврат порта (можно вызвать явно)
    pub async fn release(mut self) {
        if !self.returned {
            self.returned = true;
            self.pool.release(self.port).await;
        }
    }
}

impl Drop for PortLease {
    fn drop(&mut self) {
        if !self.returned {
            let pool = self.pool.clone();
            let port = self.port;
            // Асинхронный возврат без блокировки
            tokio::spawn(async move {
                pool.release(port).await;
            });
        }
    }
}

impl PortPool {
    pub fn new(start: u16, count: usize) -> DaemonResult<Self> {
        let end = start
            .checked_add(u16::try_from(count)?)
            .ok_or(DaemonError::WrongPortCount)?;

        let available: VecDeque<u16> = (start..end).collect();

        Ok(Self {
            available: Mutex::new(available),
            range: start..end,
        })
    }

    /// Получить порт (без Guard - может быть забыт)
    pub async fn acquire(&self) -> Option<u16> {
        self.available.lock().await.pop_front()
    }

    /// Получить порт с Guard (рекомендуемый способ)
    pub async fn acquire_guarded(self: Arc<Self>) -> DaemonResult<PortLease> {
        let port = self.acquire().await.ok_or(DaemonError::NoAvailablePorts)?;

        debug!("Port {} acquired", port);

        Ok(PortLease {
            port,
            pool: self,
            returned: false,
        })
    }

    /// Вернуть порт в пул
    pub async fn release(&self, port: u16) {
        // Проверяем, что порт из нашего диапазона
        if !self.range.contains(&port) {
            debug!("Ignoring release of port {} (out of range)", port);
            return;
        }

        let mut available = self.available.lock().await;

        // Предотвращаем дублирование
        if available.contains(&port) {
            debug!("Port {} already in pool, skipping", port);
            return;
        }

        available.push_back(port);
        debug!("Port {} released ({} available)", port, available.len());
    }

    // /// Количество доступных портов
    // pub async fn available_count(&self) -> usize {
    //     self.available.lock().await.len()
    // }

    // pub async fn with_lease<F, T>(self: &Arc<Self>, f: F) -> DaemonResult<T>
    // where
    //     F: FnOnce(u16) -> impl std::future::Future<Output = DaemonResult<T>>,
    // {
    //     let lease = self.clone().acquire_guarded().await?;
    //     let result = f(lease.port()).await?;
    //     lease.release().await;
    //     Ok(result)
    // }
}

// ----- src\proxy.rs -----

use std::process::Stdio;
use tempfile::TempDir;
use tokio::{
    fs,
    io::{AsyncBufReadExt, BufReader},
    net::TcpStream,
    process::{Child, Command},
    time::{self, Duration},
};
use tracing::{debug, error, info, warn};
use xray_parser::{ConfigParser, ToJson, XRayNode};

use crate::error::{DaemonError, DaemonResult};

pub struct ProxyProcess {
    child: Child,
    port: u16,
    _temp_dir: TempDir,
}

impl ProxyProcess {
    pub async fn start(xray_path: &str, node: &XRayNode, port: u16) -> DaemonResult<Self> {
        info!(
            "Starting XRay proxy for node {} on 127.0.0.1:{}",
            node.uuid, port
        );

        // Создаём временную директорию
        let temp_dir = TempDir::new()?;
        let config_path = temp_dir.path().join(format!("config_{}.json", port));

        let proxy_cfg = ConfigParser::generate_for_node(node, port, Some("127.0.0.1"))?;
        let config_json = proxy_cfg.to_json()?;

        // Записываем основной файл
        fs::write(&config_path, config_json).await?;

        // Используем tokio::process::Command для асинхронности
        let mut child = Command::new(xray_path)
            .args(["run", "-c"])
            .arg(&config_path)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .kill_on_drop(true) // Автоматически убиваем при drop
            .spawn()?;

        // Забираем stdout/stderr для логирования
        if let Some(stdout) = child.stdout.take() {
            tokio::spawn(async move {
                let reader = BufReader::new(stdout);
                let mut lines = AsyncBufReadExt::lines(reader);
                while let Ok(Some(line)) = lines.next_line().await {
                    debug!("[XRay:{:?} stdout] {}", port, line);
                }
            });
        }

        if let Some(stderr) = child.stderr.take() {
            tokio::spawn(async move {
                let reader = BufReader::new(stderr);
                let mut lines = AsyncBufReadExt::lines(reader);
                while let Ok(Some(line)) = lines.next_line().await {
                    if line.contains("error") || line.contains("failed") || line.contains("ERROR") {
                        error!("[XRay:{:?} stderr] {}", port, line);
                    } else {
                        debug!("[XRay:{:?} stderr] {}", port, line);
                    }
                }
            });
        }

        Ok(Self {
            child,
            _temp_dir: temp_dir,
            port,
        })
    }

    pub fn is_alive(&mut self) -> bool {
        match self.child.try_wait() {
            Ok(None) => true,

            Ok(Some(status)) => {
                warn!("Proxy exited: {:?}", status);
                false
            }

            Err(err) => {
                error!("try_wait failed: {}", err);
                false
            }
        }
    }

    /// Проверяет, готов ли прокси
    pub async fn wait_until_ready(&mut self, timeout_secs: u64) -> DaemonResult<()> {
        let proxy_addr = format!("127.0.0.1:{}", self.port);
        let timeout_duration = Duration::from_secs(timeout_secs);

        time::timeout(timeout_duration, async {
            loop {
                if !self.is_alive() {
                    return Err(DaemonError::ProxyDied);
                }

                match TcpStream::connect(&proxy_addr).await {
                    Ok(_) => {
                        info!("Proxy on port {} is ready", self.port);
                        return Ok(());
                    }
                    Err(_) => {
                        time::sleep(Duration::from_millis(100)).await;
                    }
                }
            }
        })
        .await??;

        Ok(())
    }

    /// Останавливает прокси процесс
    pub async fn stop(mut self) -> DaemonResult<()> {
        info!("Stopping proxy {}", self.port);

        let _ = self.child.start_kill();

        let _ = time::timeout(Duration::from_secs(5), self.child.wait()).await;

        Ok(())
    }
}

// ----- src\result.rs -----

use std::time::Instant;

use reqwest::{Error, Response};
use time::OffsetDateTime;
use xray_parser::XRayNode;

#[derive(Debug, Clone)]
pub struct UrlTestResult {
    pub url: String,
    pub status: Option<u16>,
    pub latency_ms: u64,
    pub error: Option<String>,
}

impl UrlTestResult {
    pub async fn from_reqwest(
        started: Instant,
        url: String,
        result: Result<Response, Error>,
    ) -> Self {
        let (status, error) = match result {
            Ok(response) => (Some(response.status().as_u16()), None),
            Err(e) => (None, Some(e.to_string())),
        };
        let latency_ms = started.elapsed().as_millis() as u64;

        Self {
            url,
            status,
            latency_ms,
            error,
        }
    }
}

#[derive(Debug, Clone)]
pub struct NodeTestResult {
    pub task_id: u64,
    pub node: XRayNode,
    pub results: Vec<UrlTestResult>,
    pub error: Option<String>,
    pub started_at: OffsetDateTime,
    pub finished_at: OffsetDateTime,
}

impl NodeTestResult {
    pub fn success(
        task_id: u64,
        node: XRayNode,
        results: Vec<UrlTestResult>,
        started_at: OffsetDateTime,
    ) -> Self {
        Self {
            task_id,
            node,
            results,
            error: None,
            started_at,
            finished_at: OffsetDateTime::now_utc(),
        }
    }

    pub fn failure(
        task_id: u64,
        node: XRayNode,
        started_at: OffsetDateTime,
        error: impl Into<String>,
    ) -> Self {
        Self {
            task_id,
            node,
            results: vec![],
            error: Some(error.into()),
            started_at,
            finished_at: OffsetDateTime::now_utc(),
        }
    }
}

// ----- src\retry.rs -----

use std::future::Future;
use std::time::Duration;
use tokio::time;

pub async fn retry<F, Fut, T, E>(max_attempts: usize, delay: Duration, f: F) -> Result<T, E>
where
    F: Fn() -> Fut,
    Fut: Future<Output = Result<T, E>>,
{
    let mut attempts = 0;
    loop {
        attempts += 1;
        match f().await {
            Ok(value) => return Ok(value),
            Err(e) if attempts < max_attempts => {
                tracing::debug!("Attempt {} failed, retrying in {:?}", attempts, delay);
                time::sleep(delay).await;
            }
            Err(e) => return Err(e),
        }
    }
}

// ----- src\subscription.rs -----

use tokio::fs;
use tracing::{info, warn};
use xray_parser::{XRayNode, parser::uri::NodeUriParser};

use crate::{
    client::ClientConfig,
    config::WorkerConfig,
    error::{DaemonError, DaemonResult},
    utils::maybe_decode_base64,
};

#[derive(Debug, Clone)]
pub struct Subscription {
    source: String,
    config: WorkerConfig,
}

impl Subscription {
    pub fn new(source: &str, config: WorkerConfig) -> Self {
        Self {
            source: source.into(),
            config,
        }
    }

    async fn download_from_url(&self, url: &str) -> DaemonResult<String> {
        info!("Downloading subscription from URL...");
        let client = ClientConfig::direct(&self.config).build_client()?;
        let content = client.get(url).send().await?.text().await?;
        Ok(maybe_decode_base64(content))
    }

    async fn read_from_file(&self, path: &str) -> DaemonResult<String> {
        info!("Reading subscription from file: {}", path);
        let content = fs::read_to_string(path).await?;
        Ok(maybe_decode_base64(content))
    }

    pub async fn get_nodes(&self) -> DaemonResult<Vec<XRayNode>> {
        let content = if self.source.starts_with("http://") || self.source.starts_with("https://") {
            self.download_from_url(&self.source).await?
        } else if tokio::fs::try_exists(&self.source).await? {
            self.read_from_file(&self.source).await?
        } else {
            if self.source.contains("://") {
                self.source.clone()
            } else {
                return Err(DaemonError::InvalidSubscriptionSource(self.source.clone()));
            }
        };

        Ok(Self::parse_nodes_from_text(&content))
    }

    fn parse_nodes_from_text(content: &str) -> Vec<XRayNode> {
        let lines: Vec<&str> = content
            .lines()
            .map(str::trim)
            .filter(|l| !l.is_empty())
            .collect();

        info!("Parsing {} lines from subscription", lines.len());

        let nodes: Vec<XRayNode> = lines
            .iter()
            .filter_map(|line| NodeUriParser::parse(line).ok())
            .collect();

        let failed = lines.len() - nodes.len();
        if failed > 0 {
            warn!("Failed to parse {} / {} lines", failed, lines.len());
        }

        info!("Successfully parsed {} nodes", nodes.len());

        nodes
    }
}

// ----- src\task.rs -----

use std::{
    sync::Arc,
    time::{Duration, Instant},
};

use futures::{StreamExt, stream};
use time::OffsetDateTime;
use tokio::time::sleep;
use xray_parser::XRayNode;

use crate::{
    client::ClientConfig,
    config::WorkerConfig,
    error::DaemonResult,
    port_pool::PortPool,
    proxy::ProxyProcess,
    result::{NodeTestResult, UrlTestResult},
};

#[derive(Clone, Debug)]
pub struct NodeTask {
    pub id: u64,
    pub node: XRayNode,
}

const RELEASE_WAIT_MS: u64 = 100;

pub async fn run_task(
    port_pool: Arc<PortPool>,
    config: WorkerConfig,
    task: NodeTask,
) -> DaemonResult<NodeTestResult> {
    let started_at = OffsetDateTime::now_utc();

    tokio::time::timeout(
        Duration::from_secs(config.task_timeout_secs + config.request_timeout_secs + 10),
        async {
            let lease = port_pool.acquire_guarded().await?;
            run_task_with_port(lease.port(), &config, task, started_at).await
        },
    )
    .await?
}

async fn run_task_with_port(
    port: u16,
    config: &WorkerConfig,
    task: NodeTask,
    started_at: OffsetDateTime,
) -> DaemonResult<NodeTestResult> {
    let mut proxy = ProxyProcess::start(&config.xray_path, &task.node, port).await?;
    proxy.wait_until_ready(config.task_timeout_secs).await?;

    let results = test_urls_with_limit(port, config).await?;

    proxy.stop().await?;
    sleep(Duration::from_millis(RELEASE_WAIT_MS)).await;

    Ok(NodeTestResult::success(
        task.id, task.node, results, started_at,
    ))
}

async fn test_urls_with_limit(
    port: u16,
    config: &WorkerConfig,
) -> DaemonResult<Vec<UrlTestResult>> {
    let client = ClientConfig::with_proxy(config, port).build_client()?;

    let results: Vec<UrlTestResult> = stream::iter(config.urls.iter().cloned())
        .map(|url| {
            let client = client.clone();
            async move {
                let started = Instant::now();
                let result = client.get(&url).send().await;
                UrlTestResult::from_reqwest(started, url, result).await
            }
        })
        .buffer_unordered(config.max_concurrent_requests)
        .collect()
        .await;

    Ok(results)
}

// ----- src\utils.rs -----

use base64::{Engine as _, engine::general_purpose::STANDARD};
use tracing::info;

use crate::error::DaemonResult;

pub fn maybe_decode_base64(content: String) -> String {
    // Если это не похоже на base64, возвращаем как есть
    if !looks_like_base64(&content) {
        return content;
    }

    // Пробуем декодировать
    if let Ok(decoded) = try_decode_base64(&content)
        && decoded.contains("://")
    {
        info!("Decoded base64 subscription ({} bytes)", decoded.len());
        return decoded;
    }

    content
}

fn looks_like_base64(s: &str) -> bool {
    let trimmed = s.trim();

    // Многострочный или содержит URI - точно не base64
    if trimmed.contains("://") || trimmed.lines().count() > 1 {
        return false;
    }

    let cleaned: String = trimmed.chars().filter(|c| !c.is_whitespace()).collect();

    // Базовые проверки длины
    if cleaned.len() < 16 || !cleaned.len().is_multiple_of(4) {
        return false;
    }

    // Проверка допустимых символов
    cleaned
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || matches!(c, '+' | '/' | '='))
}

fn try_decode_base64(s: &str) -> DaemonResult<String> {
    let cleaned: String = s.chars().filter(|c| !c.is_whitespace()).collect();
    let bytes = STANDARD.decode(&cleaned)?;
    Ok(String::from_utf8(bytes)?)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn base64_decoded_when_contains_uris() {
        let uris = "vless://uuid@host:443?encryption=none\nvless://uuid2@host2:443?encryption=none";
        let encoded = STANDARD.encode(uris);
        let result = maybe_decode_base64(encoded);
        assert_eq!(result, uris);
    }

    #[test]
    fn base64_not_decoded_when_already_uri_list() {
        let uris = "vless://uuid@host:443?encryption=none\nvless://uuid2@host:443?encryption=none";
        let result = maybe_decode_base64(uris.to_string());
        assert_eq!(result, uris);
    }

    #[test]
    fn base64_not_decoded_when_random_base64_garbage() {
        // Корректный base64, но не содержит URI после декодирования
        let garbage = STANDARD.encode("this is not a proxy list at all");
        let result = maybe_decode_base64(garbage.clone());
        // Не должно декодироваться, так как результат не содержит "://"
        assert_eq!(result, garbage);
    }
}

// ----- src\worker.rs -----

use std::sync::Arc;

use tokio::{
    sync::{Semaphore, mpsc::Receiver},
    task::JoinSet,
};
use tokio_util::sync::CancellationToken;
use tracing::{error, info, warn};

use crate::{
    config::{ManagerConfig, WorkerConfig},
    error::DaemonResult,
    port_pool::PortPool,
    task::{NodeTask, run_task},
};

#[derive(Debug)]
pub struct Worker {
    pub rx: Receiver<NodeTask>,
    pub semaphore: Arc<Semaphore>,
    pub port_pool: Arc<PortPool>,
    pub worker_config: WorkerConfig,
    pub shutdown: CancellationToken,
}

impl Worker {
    pub fn new(
        rx: Receiver<NodeTask>,
        config: ManagerConfig,
        shutdown: CancellationToken,
    ) -> DaemonResult<Self> {
        let semaphore = Arc::new(Semaphore::new(config.max_concurrent_tasks));
        let port_pool = Arc::new(PortPool::new(config.port_range_start, config.port_count)?);
        let worker_config = config.worker;

        Ok(Self {
            rx,
            semaphore,
            port_pool,
            worker_config,
            shutdown,
        })
    }

    pub async fn run(mut self) {
        let mut tasks = JoinSet::new();

        // Фаза 1: Принимаем задачи
        loop {
            tokio::select! {
                _ = self.shutdown.cancelled() => {
                    info!("Shutdown signal received");
                    break;
                }
                Some(task) = self.rx.recv() => {
                    self.spawn_task(&mut tasks, task).await;
                }
                else => break,
            }
        }

        // Фаза 2: Дожидаемся завершения всех запущенных задач
        self.wait_for_all(tasks).await;
        info!("worker stopped");
    }

    async fn spawn_task(&self, tasks: &mut JoinSet<()>, task: NodeTask) {
        let permit = match self.semaphore.clone().acquire_owned().await {
            Ok(p) => p,
            Err(_) => {
                error!("Semaphore closed");
                return;
            }
        };

        let port_pool = self.port_pool.clone();
        let config = self.worker_config.clone();

        tasks.spawn(async move {
            let _permit = permit;
            Self::execute_task(port_pool, config, task).await;
        });
    }

    async fn execute_task(port_pool: Arc<PortPool>, config: WorkerConfig, task: NodeTask) {
        match run_task(port_pool, config, task).await {
            Ok(result) => info!(task_id = result.task_id, error = ?result.error, "task completed"),
            Err(err) => error!(%err, "task failed"),
        }
    }

    async fn wait_for_all(&mut self, mut tasks: JoinSet<()>) {
        let remaining = tasks.len();
        if remaining > 0 {
            info!("Waiting for {} remaining tasks", remaining);
        }

        while let Some(res) = tasks.join_next().await {
            if let Err(err) = res {
                warn!(%err, "Task join error");
            }
        }
    }
}

