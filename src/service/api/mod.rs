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
    reqwest::{
        header::{HeaderMap, HeaderValue},
        multipart, Client, Response,
    },
    tokio,
    users::{UserLogin, UserRegistration},
    BufferData, DynoConfig, DynoErr,
};
use eframe::epaint::mutex::Mutex;

use crate::AsyncMsg;

static APP_USER_AGENT: &str = concat!("Dyno/Desktop-", env!("CARGO_PKG_VERSION"),);

#[derive(Clone)]
pub struct ApiService {
    pub url: String,
    pub client: Client,
    logined: Arc<AtomicBool>,
    token_session: Arc<Mutex<Option<TokenDetails>>>,
}

impl ApiService {
    pub fn new() -> Option<Self> {
        let url = std::env::var("DYNO_SERVER_URL").unwrap_or_else(|err| {
            dyno_core::log::error!(
                "Failed to Get DYNO_SERVER_URL from EnvVar, defaulting to [localhost:3000] - {err}"
            );
            "http://127.0.0.1:3000".to_owned()
        });
        let client = match Client::builder()
            .default_headers({
                let mut headers = HeaderMap::new();
                headers.insert("AppDyno", HeaderValue::from_static("Desktop"));
                headers
            })
            .user_agent(APP_USER_AGENT)
            .build()
            .map_err(DynoErr::service_error)
        {
            Ok(ok) => ok,
            Err(err) => {
                dyno_core::log::error!("Failed to create Api Client - {err}");
                return None;
            }
        };
        Some(Self {
            url,
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

    fn api_url(&self, url: impl AsRef<str>) -> String {
        format!("{}/api{}", self.url, url.as_ref())
    }

    fn data_url(&self, url: impl AsRef<str>) -> String {
        format!("{}{}", self.url, url.as_ref())
    }
}

impl ApiService {
    pub fn check_health(&self, tx: Sender<AsyncMsg>) {
        let client = self.client.clone();
        let url = self.api_url("/health");
        let async_spawn = async move {
            match client
                .get(url)
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

    pub fn set_active(&self, cfgs: DynoConfig, tx: Sender<AsyncMsg>) {
        let client = self.client.clone();
        let url = self.api_url("/active");
        let async_spawn = async move {
            match client
                .post(url)
                .json(&cfgs)
                .send()
                .await
                .and_then(Response::error_for_status)
                .map_err(DynoErr::service_error)
            {
                Ok(_resp) => tx.send(AsyncMsg::message("Success Connecting to API Server!")),
                Err(err) => tx.send(AsyncMsg::error(err)),
            }
        };
        tokio::spawn(async_spawn);
    }

    pub fn set_non_active(&self, tx: Sender<AsyncMsg>) {
        let client = self.client.clone();
        let url = self.api_url("/non_active");
        let async_spawn = async move {
            match client
                .post(url)
                .send()
                .await
                .and_then(Response::error_for_status)
                .map_err(DynoErr::service_error)
            {
                Ok(_resp) => tx.send(AsyncMsg::message("Success Disconnecting to API Server!")),
                Err(err) => tx.send(AsyncMsg::error(err)),
            }
        };
        tokio::spawn(async_spawn);
    }

    pub fn login(&self, login: UserLogin, tx: Sender<AsyncMsg>) {
        let client = self.client.clone();
        let logined = self.logined.clone();
        let token_session = self.token_session.clone();
        let url = self.api_url("/auth/login");
        tokio::spawn(async move {
            match user::user_login(url, client, login).await {
                Ok(resp) => {
                    logined.store(true, Ordering::Relaxed);
                    let mut token_lock = token_session.lock();
                    *token_lock = Some(resp.payload);
                    ignore_err!(tx.send(AsyncMsg::OnApiLogin));
                }
                Err(err) => ignore_err!(tx.send(err)),
            }
        });
    }

    pub fn register(&self, register: UserRegistration, tx: Sender<AsyncMsg>) {
        let client = self.client.clone();
        let url = self.api_url("/auth/register");

        tokio::spawn(async move {
            match user::user_register(url, client, register).await {
                Ok(_ok) => ignore_err!(tx.send(AsyncMsg::OnApiRegister)),
                Err(err) => ignore_err!(tx.send(err)),
            }
        });
    }

    pub fn logout(&self, tx: Sender<AsyncMsg>) {
        let token = match self.get_token() {
            Some(tok) => tok,
            None => {
                ignore_err!(tx.send(AsyncMsg::error(DynoErr::api_error(
                    "You are not Login, please Login first."
                ))));
                return;
            }
        };

        let logined = self.logined.clone();
        let token_session = self.token_session.clone();
        let client = self.client.clone();
        let url = self.api_url("/auth/logout");
        tokio::spawn(async move {
            match user::user_logout(url, client, token).await {
                Ok(ok) => {
                    logined.store(false, Ordering::Relaxed);
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
                ignore_err!(tx.send(AsyncMsg::error(DynoErr::api_error(
                    "You are not Login, please Login first."
                ))));
                return;
            }
        };

        let client = self.client.clone();
        let url = self.api_url("/dyno");

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

            ignore_err!(tx.send(dyno::save(url, client, token, multiparts).await))
        });
    }

    pub fn get_dyno(&self, tx: Sender<AsyncMsg>) {
        let token = match self.get_token() {
            Some(tok) => tok,
            None => {
                ignore_err!(tx.send(AsyncMsg::error(DynoErr::api_error(
                    "You are not Login, please Login first."
                ))));
                return;
            }
        };
        let client = self.client.clone();
        let url = self.api_url("/dyno?all=true");
        tokio::spawn(async move {
            ignore_err!(tx.send(dyno::get(url, client, token).await));
        });
    }

    pub fn load_dyno_file(&self, url: String, checksum: String, tx: Sender<AsyncMsg>) {
        let token = match self.get_token() {
            Some(tok) => tok,
            None => {
                ignore_err!(tx.send(AsyncMsg::error(DynoErr::api_error(
                    "You are not Login, please Login first."
                ))));
                return;
            }
        };
        let client = self.client.clone();
        let url = self.data_url(url);
        tokio::spawn(async move {
            ignore_err!(tx.send(dyno::load_file(url, client, token, checksum).await));
        });
    }
}
