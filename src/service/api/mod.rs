mod dyno;
mod user;

use dyno_core::{
    chrono::NaiveDateTime,
    crossbeam_channel::Sender,
    crypto::TokenDetails,
    ignore_err,
    serde::de::DeserializeOwned,
    tokio,
    users::{UserLogin, UserRegistration},
    ApiResponse, BufferData, DynoConfig, DynoErr, DynoResult,
};
use eframe::epaint::mutex::Mutex;
use reqwest::{header, Client, Response};
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};

use crate::AsyncMsg;

const APP_USER_AGENT: &str = concat!("Dyno/Desktop-", env!("CARGO_PKG_VERSION"),);
#[allow(clippy::declare_interior_mutable_const)]
const HEADER_NAME_APP_USER_AGENT: header::HeaderName = header::HeaderName::from_static("app");
#[allow(clippy::declare_interior_mutable_const)]
const HEADER_VALUE_APP_USER_AGENT: header::HeaderValue =
    header::HeaderValue::from_static("Desktop");

#[derive(Clone)]
pub struct ApiService {
    pub url: String,
    pub client: Client,
    tx: Sender<AsyncMsg>,
    logined: Arc<AtomicBool>,
    token_session: Arc<Mutex<Option<TokenDetails>>>,
}

impl ApiService {
    pub fn new(tx: Sender<AsyncMsg>) -> Option<Self> {
        let url = std::env::var("DYNO_SERVER_URL").unwrap_or_else(|err| {
            dyno_core::log::error!(
                "Failed to Get DYNO_SERVER_URL from EnvVar, defaulting to [localhost:3000] - {err}"
            );
            "http://127.0.0.1:3000".to_owned()
        });
        let client = match Client::builder()
            .default_headers(
                [(HEADER_NAME_APP_USER_AGENT, HEADER_VALUE_APP_USER_AGENT)]
                    .into_iter()
                    .collect(),
            )
            .user_agent(header::HeaderValue::from_static(APP_USER_AGENT))
            .build()
        {
            Ok(ok) => ok,
            Err(err) => {
                dyno_core::log::error!("Failed to build client: {err}");
                return None;
            }
        };

        Some(Self {
            tx,
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
    pub fn check_health(&self) {
        let tx = self.tx.clone();
        let client = self.client.clone();
        let url = self.api_url("/health");
        let async_spawn = async move {
            match client
                .get(url)
                .send()
                .await
                .map(Resp::from_resp)
                .map_err(DynoErr::service_error)
            {
                Ok(Resp::Success(_)) => tx.send(AsyncMsg::check_health()),
                Ok(Resp::Error(_)) => tx.send(AsyncMsg::error("Failed to check health api")),
                Err(err) => tx.send(AsyncMsg::error(err)),
            }
        };
        tokio::spawn(async_spawn);
    }

    pub fn set_active(&self, cfgs: DynoConfig) {
        let tx = self.tx.clone();
        let client = self.client.clone();
        let url = self.api_url("/active");
        let async_spawn = async move {
            match client
                .post(url)
                .json(&cfgs)
                .send()
                .await
                .map(Resp::from_resp)
                .map_err(DynoErr::service_error)
            {
                Ok(Resp::Success(_)) => {
                    tx.send(AsyncMsg::message("Success Connecting to API Server!"))
                }
                Ok(Resp::Error(_)) => tx.send(AsyncMsg::error("Failed set active in api server!")),
                Err(err) => tx.send(AsyncMsg::error(err)),
            }
        };
        tokio::spawn(async_spawn);
    }

    pub fn set_non_active(&self) {
        let tx = self.tx.clone();
        let client = self.client.clone();
        let url = self.api_url("/non_active");
        let async_spawn = async move {
            match client
                .post(url)
                .send()
                .await
                .map(Resp::from_resp)
                .map_err(DynoErr::service_error)
            {
                Ok(Resp::Success(_)) => {
                    tx.send(AsyncMsg::message("Success Disconnecting to API Server!"))
                }
                Ok(Resp::Error(_)) => {
                    tx.send(AsyncMsg::message("Failed Disconnecting to API Server!"))
                }
                Err(err) => tx.send(AsyncMsg::error(err)),
            }
        };
        tokio::spawn(async_spawn);
    }

    pub fn login(&self, login: UserLogin) {
        let tx = self.tx.clone();
        let client = self.client.clone();
        let logined = self.logined.clone();
        let token_session = self.token_session.clone();
        let url = self.api_url("/auth/login");
        tokio::spawn(async move {
            match user::user_login(url, client, login).await {
                Ok(resp) => {
                    logined.store(true, Ordering::Relaxed);
                    let mut token_lock = token_session.lock();
                    *token_lock = Some(resp);
                    ignore_err!(tx.send(AsyncMsg::OnApiLogin));
                }
                Err(err) => ignore_err!(tx.send(err)),
            }
        });
    }

    pub fn register(&self, register: UserRegistration) {
        let tx = self.tx.clone();
        let client = self.client.clone();
        let url = self.api_url("/auth/register");
        tokio::spawn(async move {
            ignore_err!(tx.send(user::user_register(url, client, register).await))
        });
    }

    pub fn logout(&self) {
        let tx = self.tx.clone();
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
    ) {
        let tx = self.tx.clone();
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
            ignore_err!(tx.send(dyno::save(url, client, token, data, config, start, stop).await))
        });
    }

    pub fn get_dyno(&self) {
        let tx = self.tx.clone();
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

    pub fn load_dyno_file(&self, url: String, checksum: String) {
        let tx = self.tx.clone();
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

pub enum Resp {
    Success(Response),
    Error(Response),
}

impl Resp {
    pub fn from_resp(resp: Response) -> Self {
        if resp.status().is_success() {
            Self::Success(resp)
        } else {
            Self::Error(resp)
        }
    }
    pub async fn get_error(self) -> String {
        let Self::Error(resp) = self else { return String::new(); };
        match resp.text().await {
            Ok(k) => k,
            Err(err) => err.to_string(),
        }
    }

    pub async fn get_json<T>(self) -> DynoResult<T>
    where
        T: DeserializeOwned,
    {
        match self {
            Resp::Success(resp) => match resp.json::<ApiResponse<T>>().await {
                Ok(ok) => Ok(ok.payload),
                Err(err) => Err(DynoErr::api_error(err)),
            },
            Resp::Error(_) => Err(DynoErr::api_error(self.get_error().await)),
        }
    }
}

#[inline]
pub fn map_resp_error(error: reqwest::Error) -> AsyncMsg {
    AsyncMsg::OnError(DynoErr::api_error(error))
}
