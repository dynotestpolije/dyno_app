mod dyno;
mod user;

use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};

use dyno_core::{
    chrono::NaiveDateTime,
    crossbeam_channel::Sender,
    crypto::TokenDetails,
    dynotests::DynoTestDataInfo,
    ignore_err,
    reqwest::{multipart, Client, Response},
    tokio,
    users::{UserLogin, UserRegistration},
    BufferData, DynoConfig, DynoErr, DynoResult,
};
use eframe::epaint::mutex::Mutex;

use crate::AsyncMsg;

static APP_USER_AGENT: &str = concat!(env!("CARGO_PKG_NAME"), "/", env!("CARGO_PKG_VERSION"),);

#[derive(Clone)]
pub struct ApiService {
    client: Client,
    logined: Arc<AtomicBool>,
    token_session: Arc<Mutex<Option<TokenDetails>>>,
}

impl ApiService {
    pub fn new() -> DynoResult<Self> {
        let client = Client::builder()
            .user_agent(APP_USER_AGENT)
            .build()
            .map_err(DynoErr::service_error)?;
        Ok(Self {
            client,
            logined: Arc::new(AtomicBool::new(false)),
            token_session: Default::default(),
        })
    }

    #[inline]
    pub fn is_logined(&self) -> bool {
        self.logined.load(Ordering::Relaxed)
    }

    #[inline]
    fn get_token(&self) -> Option<String> {
        let lock = self.token_session.lock();
        lock.as_ref().and_then(|tok| tok.token.clone())
    }
}

impl ApiService {
    pub fn check_health(&self, tx: Sender<AsyncMsg>) {
        let client = self.client.clone();
        let async_spawn = async move {
            match client
                .get(api_url!("/health"))
                .send()
                .await
                .and_then(Response::error_for_status)
                .map_err(DynoErr::service_error)
            {
                Ok(resp) => tx.send(AsyncMsg::check_health(resp.status())),
                Err(err) => tx.send(AsyncMsg::error(err)),
            }
        };
        tokio::spawn(async_spawn);
    }

    pub fn login(&self, login: UserLogin, tx: Sender<AsyncMsg>) {
        let client = self.client.clone();
        let logined = self.logined.clone();
        let token_session = self.token_session.clone();

        tokio::spawn(async move {
            match user::user_login(client, login).await {
                Ok(resp) => {
                    logined.store(true, Ordering::Relaxed);
                    let mut token_lock = token_session.lock();
                    *token_lock = Some(resp.payload);
                }
                Err(err) => ignore_err!(tx.send(err)),
            }
        });
    }

    pub fn register(&self, register: UserRegistration, tx: Sender<AsyncMsg>) {
        let client = self.client.clone();

        tokio::spawn(async move {
            match user::user_register(client, register).await {
                Ok(ok) => ignore_err!(tx.send(AsyncMsg::OnMessage(format!(
                    "Success Register user in Api Endpoint, with response:{} !",
                    ok.payload
                )))),
                Err(err) => ignore_err!(tx.send(err)),
            }
        });
    }

    pub fn logout(&self, tx: Sender<AsyncMsg>) {
        if !self.is_logined() {
            return;
        }
        let token_session = self.token_session.clone();
        let client = self.client.clone();
        tokio::spawn(async move {
            match user::user_logout(client).await {
                Ok(ok) => {
                    {
                        let mut token_lock = token_session.lock();
                        *token_lock = None;
                    }
                    ignore_err!(tx.send(ok))
                }
                Err(err) => ignore_err!(tx.send(err)),
            }
        });
    }
}

impl ApiService {
    pub fn save_dyno(
        &self,
        data: BufferData,
        config: DynoConfig,
        start: NaiveDateTime,
        stop: NaiveDateTime,
        tx: Sender<AsyncMsg>,
    ) {
        let token = match self.get_token() {
            Some(tok) => tok,
            None => {
                ignore_err!(tx.send(AsyncMsg::error("You are not Login, please Login first.")));
                return;
            }
        };

        let client = self.client.clone();

        tokio::spawn(async move {
            let (file_part, checksum_hex) = match dyno::get_data_part(data).await {
                Ok(ok) => ok,
                Err(err) => {
                    ignore_err!(tx.send(AsyncMsg::error(err)));
                    return;
                }
            };
            let config_data = DynoTestDataInfo {
                checksum_hex,
                config,
                start,
                stop,
            };
            let info_part = match dyno::get_info_part(config_data).await {
                Ok(ok) => ok,
                Err(err) => {
                    ignore_err!(tx.send(AsyncMsg::error(err)));
                    return;
                }
            };
            let multiparts = multipart::Form::new()
                .part("data", file_part)
                .part("info", info_part);

            match dyno::save(client, token, multiparts).await {
                Ok(ok) => ignore_err!(tx.send(ok)),
                Err(err) => ignore_err!(tx.send(err)),
            }
        });
    }

    pub fn get_dyno(&self, tx: Sender<AsyncMsg>) {
        let token = match self.get_token() {
            Some(tok) => tok,
            None => {
                ignore_err!(tx.send(AsyncMsg::error("You are not Login, please Login first.")));
                return;
            }
        };
        let client = self.client.clone();

        tokio::spawn(async move {
            let result = dyno::get(client, token).await;
            ignore_err!(tx.send(result));
        });
    }

    pub fn load_dyno_file(&self, url: String, checksum: String, tx: Sender<AsyncMsg>) {
        let token = match self.get_token() {
            Some(tok) => tok,
            None => {
                ignore_err!(tx.send(AsyncMsg::error("You are not Login, please Login first.")));
                return;
            }
        };
        let client = self.client.clone();

        tokio::spawn(async move {
            let result = dyno::load_file(client, token, url, checksum).await;
            ignore_err!(tx.send(result));
        });
    }
}

macro_rules! api_url {
    ($paths:literal) => {
        concat!(env!("API_SERVER_URL"), "/api", $paths)
    };
}

macro_rules! data_url {
    ($paths:literal) => {
        concat!(env!("API_SERVER_URL"), "/data", $paths)
    };
    ($paths:expr) => {
        format!("{}/{}", env!("API_SERVER_URL"), $paths)
    };
}
use {api_url, data_url};
