use std::{fmt::Display, sync::Arc};

use iced::task::{sipper, Never, Sipper};
use tokio::{process::Child, sync::{RwLock, mpsc}};
use tracing::{debug, info};
use tokio::time::{sleep, Duration};

#[derive(Clone,Debug)]
pub enum LCommand {
    Started{ host: String, port: u16, command: Arc<RwLock<Option<Child>>> },
    Stopped,
}

#[derive(Clone,Debug)]
pub enum LlamaEvent {
    SipperStarted(mpsc::Sender<LCommand>),
    NoLocalFound,
    LocalFoundNotResponding,
    LocalFoundNotRunning,
    Running,
    Error(String),
}

impl Display for LlamaEvent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let label = match self {
            LlamaEvent::NoLocalFound => t!("st_nlocfnd").to_string(),
            LlamaEvent::LocalFoundNotResponding => t!("st_busy").to_string(),
            LlamaEvent::LocalFoundNotRunning => t!("st_stopped").to_string(),
            LlamaEvent::Running => t!("st_running").to_string(),
            LlamaEvent::Error(msg) => format!("{}: {}", t!("error: ").to_string(), msg),
            LlamaEvent::SipperStarted(_) => String::from("⏩"),
        };
        write!(f, "{}", label)
    }
}

/// Checks if llama-server is actually responding on the /health endpoint
async fn is_llama_server_healthy(client: &reqwest::Client, url: &str) -> bool {
    if url.is_empty() {
        debug!("Empty addr");
        return false;
    }

    let url = format!("{}/health",url);
    debug!("url={}", url);

    let res = client.get(&url)
        .timeout(Duration::from_millis(100))
        .send()
        .await;
    if let Err(e) = &res {
        debug!("Error llama: {}", e);
    }
    
    res.ok().map(|resp| resp.status().is_success())
        .unwrap_or(false)
}

pub fn connect() -> impl Sipper<Never, LlamaEvent> {
    sipper(async |mut output| {
        let (sender, mut receiver) = mpsc::channel::<LCommand>(10);
        let client = reqwest::Client::new();
        let mut started = false;
        let mut laddr = String::new();
        let mut comm = None;
        output.send(LlamaEvent::SipperStarted(sender)).await;

        loop {
            match receiver.try_recv() {
                Ok(LCommand::Started { host, port, command }) => {
                    laddr = format!("http://{}:{}", host, port);
                    info!("Started: {}", laddr);
                    started = true;
                    comm = Some(command);
                }
                Ok(LCommand::Stopped) => {
                    laddr = String::new();
                    if let Some(c) = comm.clone() {
                        started = still_running(c).await;
                    }
                }
                Err(_) => {

                    let is_healthy = is_llama_server_healthy(&client, &laddr).await;
                    if let Some(c) = comm.clone() {
                        started = still_running(c).await;
                    }
                    match (started, is_healthy) {
                        (true,true) => output.send(LlamaEvent::Running).await,
                        (true, false) => output.send(LlamaEvent::LocalFoundNotResponding).await,
                        _ => output.send(LlamaEvent::LocalFoundNotRunning).await,
                    }
                    sleep(Duration::from_secs(2)).await;
                }
            }
        }
    })
}

async fn still_running(c: Arc<RwLock<Option<Child>>>) -> bool {
    let mut c = c.write().await;
    if c.is_none() {
        return false;
    }
    if let Some(r) = c.as_mut().map(async |d| d.try_wait() ) {
        let r = r.await;
        if let Ok(Some(_)) = r {
            return false;
        }
    }
    true
}
