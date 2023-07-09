use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};

use crate::{paths::DynoPaths, AsyncMsg};
use dyno_core::{crossbeam_channel::Sender, ignore_err, serde, DynoErr, DynoResult};

use futures::{
    channel::mpsc::{unbounded, UnboundedReceiver, UnboundedSender},
    SinkExt, StreamExt,
};
use tokio_tungstenite::{connect_async, tungstenite::protocol::Message};

#[derive(Clone, serde::Deserialize, serde::Serialize)]
#[serde(crate = "serde")]
pub struct WebSocketServiceConf {
    pub url: String,
}

impl Default for WebSocketServiceConf {
    fn default() -> Self {
        Self {
            url: "ws://localhost:3000/ws".to_owned(),
        }
    }
}

#[derive(Clone)]
pub struct WebSocketService {
    conf: WebSocketServiceConf,

    tx_msg: UnboundedSender<Message>,
    tx: Sender<AsyncMsg>,
    start: Arc<AtomicBool>,
}

impl WebSocketService {
    // 1 second delay polling on sending
    const DELAY_POLL: dyno_core::tokio::time::Duration =
        dyno_core::tokio::time::Duration::from_secs(1);

    pub fn new(tx: Sender<AsyncMsg>, paths: &DynoPaths) -> Self {
        let conf = paths
            .get_config::<WebSocketServiceConf>("websocket.toml")
            .unwrap_or_default();
        let (tx_msg, _) = unbounded();

        Self {
            conf,
            tx,
            tx_msg,
            start: Default::default(),
        }
    }

    pub fn start(&mut self) {
        if self.start.load(Ordering::Relaxed) {
            dyno_core::log::error!("ERROR: `WebSocketService` already running in background, stop the service first, and start again");
            return;
        }
        dyno_core::log::info!("running `WebSocketService`");
        let (tx_msg, rx_msg) = unbounded();
        self.tx_msg = tx_msg;
        let tx = self.tx.clone();

        dyno_core::tokio::spawn(Self::start_impl(
            self.conf.url.clone(),
            self.start.clone(),
            rx_msg,
            tx,
        ));
    }

    pub fn stop(&self) {
        self.start.store(false, Ordering::Relaxed);
    }

    pub fn send_data<D: serde::Serialize>(&self, data: &D) -> DynoResult<()> {
        let data = dyno_core::serde_json::to_vec(data)?;
        self.tx_msg
            .unbounded_send(Message::Binary(data))
            .map_err(DynoErr::api_error)
    }

    async fn start_impl(
        url: String,
        start: Arc<AtomicBool>,
        mut rx_msg: UnboundedReceiver<Message>,
        tx: Sender<AsyncMsg>,
    ) {
        let (mut ws, _) = match connect_async(url).await {
            Ok(ok) => {
                start.store(true, Ordering::Relaxed);
                ok
            }
            Err(err) => {
                ignore_err!(tx.send(AsyncMsg::error(DynoErr::api_error(err))));
                start.store(false, Ordering::Relaxed);
                return;
            }
        };

        loop {
            if !start.load(Ordering::Relaxed) {
                break;
            }
            if let Some(ok) = rx_msg.next().await {
                ignore_err!(ws.send(ok).await);
            }
            dyno_core::tokio::time::sleep(Self::DELAY_POLL).await;
        }
        if let Err(err) = ws.close(None).await {
            dyno_core::log::error!("{err}");
        }
    }
}
