use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};

use dyno_core::{
    asyncify,
    chrono::NaiveDateTime,
    crossbeam_channel::Sender,
    crypto::{checksum_from_bytes, compare_checksums, TokenDetails},
    dynotests::{DynoTest, DynoTestDataInfo},
    ignore_err,
    reqwest::{multipart, Client, Response},
    tokio,
    users::{UserLogin, UserRegistration},
    ApiResponse, BufferData, CompresedSaver as _, DynoConfig, DynoErr, DynoResult,
};
use eframe::epaint::mutex::Mutex;

use crate::AsyncMsg;

static APP_USER_AGENT: &str = concat!(env!("CARGO_PKG_NAME"), "/", env!("CARGO_PKG_VERSION"),);

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

        let async_spawn = async move {
            match client
                .post(api_url!("/auth/login"))
                .json(&login)
                .send()
                .await
                .and_then(Response::error_for_status)
                .map_err(AsyncMsg::error)
            {
                Ok(resp) => {
                    match resp
                        .json::<ApiResponse<TokenDetails>>()
                        .await
                        .map_err(DynoErr::service_error)
                    {
                        Ok(user_session_resp) => {
                            {
                                logined.store(true, Ordering::Relaxed);
                                let mut token_lock = token_session.lock();
                                *token_lock = Some(user_session_resp.payload);
                            }
                            ignore_err!(tx.send(AsyncMsg::message("Login Success!")))
                        }
                        Err(err) => ignore_err!(tx.send(AsyncMsg::error(err))),
                    }
                }
                Err(err) => ignore_err!(tx.send(err)),
            }
        };

        tokio::spawn(async_spawn);
    }

    pub fn register(&self, register: UserRegistration, tx: Sender<AsyncMsg>) {
        let client = self.client.clone();
        let async_spawn = async move {
            match client
                .post(api_url!("/auth/register"))
                .json(&register)
                .send()
                .await
                .and_then(Response::error_for_status)
                .map_err(AsyncMsg::error)
            {
                Ok(resp) => match resp.json::<ApiResponse<i32>>().await {
                    Ok(_resp) => {
                        ignore_err!(tx.send(AsyncMsg::message("Registration is Success!")))
                    }
                    Err(err) => ignore_err!(tx.send(AsyncMsg::error(err))),
                },
                Err(err) => ignore_err!(tx.send(err)),
            }
        };

        tokio::spawn(async_spawn);
    }

    pub fn logout(&self, tx: Sender<AsyncMsg>) {
        if !self.is_logined() {
            return;
        }
        let token_session = self.token_session.clone();
        let client = self.client.clone();
        let async_spawn = async move {
            match client
                .get(api_url!("/auth/logout"))
                .send()
                .await
                .and_then(Response::error_for_status)
                .map_err(AsyncMsg::error)
            {
                Ok(resp) => {
                    let status = resp.status();
                    if status.is_success() {
                        {
                            // reset the token and session
                            let mut token_lock = token_session.lock();
                            *token_lock = None;
                        }
                        ignore_err!(tx.send(AsyncMsg::message("Logout is Success!")))
                    } else {
                        ignore_err!(tx.send(AsyncMsg::error(format!(
                            "Logout is Error with status: {status}"
                        ))))
                    }
                }
                Err(err) => ignore_err!(tx.send(err)),
            }
        };
        tokio::spawn(async_spawn);
    }
}

impl ApiService {
    #[inline]
    pub async fn get_info_part(config: DynoTestDataInfo) -> DynoResult<multipart::Part> {
        asyncify!(move || config.compress().and_then(|info| {
            let len = info.len() as _;
            multipart::Part::stream_with_length(info, len)
                .mime_str("application/json")
                .map_err(DynoErr::service_error)
        }))
    }

    #[inline]
    pub async fn get_data_part(data: BufferData) -> DynoResult<(multipart::Part, String)> {
        asyncify!(move || data.compress().and_then(|compressed| {
            let checksum = dyno_core::crypto::checksum_from_bytes(&compressed);
            let compressed_len = compressed.len() as _;
            multipart::Part::stream_with_length(compressed, compressed_len)
                .file_name(dyno_core::uuid::Uuid::new_v4().simple().to_string())
                .mime_str("application/octet-stream")
                .map_err(DynoErr::service_error)
                .map(|part| (part, checksum))
        }))
    }

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

        let async_spawn = async move {
            let (file_part, checksum_hex) = match Self::get_data_part(data).await {
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
            let info_part = match Self::get_info_part(config_data).await {
                Ok(ok) => ok,
                Err(err) => {
                    ignore_err!(tx.send(AsyncMsg::error(err)));
                    return;
                }
            };
            let multiparts = multipart::Form::new()
                .part("data", file_part)
                .part("info", info_part);

            match client
                .post(api_url!("/dyno"))
                .multipart(multiparts)
                .bearer_auth(token)
                .send()
                .await
                .and_then(Response::error_for_status)
                .map_err(AsyncMsg::error)
            {
                Ok(resp) => match resp.json::<ApiResponse<i32>>().await {
                    Ok(id) => ignore_err!(tx.send(AsyncMsg::message(format!(
                        "Save data is Success with id - {}",
                        id.payload
                    )))),
                    Err(err) => ignore_err!(tx.send(AsyncMsg::error(err))),
                },
                Err(err) => ignore_err!(tx.send(err)),
            }
        };

        tokio::spawn(async_spawn);
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

        let async_spawn = async move {
            match client
                .get(api_url!("dyno"))
                .bearer_auth(token)
                .send()
                .await
                .and_then(Response::error_for_status)
                .map_err(AsyncMsg::error)
            {
                Ok(resp) => match resp
                    .json::<Vec<DynoTest>>()
                    .await
                    .map(AsyncMsg::on_get_dyno)
                    .map_err(AsyncMsg::error)
                {
                    Ok(ok) => ignore_err!(tx.send(ok)),
                    Err(err) => ignore_err!(tx.send(err)),
                },
                Err(err) => ignore_err!(tx.send(err)),
            }
        };

        tokio::spawn(async_spawn);
    }

    pub fn get_dyno_file(&self, dyno_url: String, checksum: String, tx: Sender<AsyncMsg>) {
        let token = match self.get_token() {
            Some(tok) => tok,
            None => {
                ignore_err!(tx.send(AsyncMsg::error("You are not Login, please Login first.")));
                return;
            }
        };
        let client = self.client.clone();

        let async_spawn = async move {
            match client
                .get(data_url!(dyno_url))
                .bearer_auth(token)
                .send()
                .await
                .and_then(Response::error_for_status)
                .map_err(AsyncMsg::error)
            {
                Ok(mut resp) => {
                    let mut data = if let Some(lenght) = resp.content_length() {
                        Vec::with_capacity(lenght as _)
                    } else {
                        vec![]
                    };
                    while let Ok(Some(chunk)) = resp.chunk().await {
                        data.extend(chunk);
                    }
                    let data_checksum = checksum_from_bytes(&data);
                    if !compare_checksums(data_checksum.as_bytes(), checksum.as_bytes()) {
                        ignore_err!(tx.send(AsyncMsg::error("Data Checksum is not matching.")));
                        return;
                    }
                    match BufferData::decompress(data)
                        .map_err(AsyncMsg::error)
                        .map(AsyncMsg::open_buffer)
                    {
                        Ok(ok) => ignore_err!(tx.send(ok)),
                        Err(err) => ignore_err!(tx.send(err)),
                    }
                }
                Err(err) => ignore_err!(tx.send(err)),
            }
        };

        tokio::spawn(async_spawn);
    }
}
