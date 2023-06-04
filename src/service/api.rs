use std::sync::Arc;

use dyno_core::{
    asyncify,
    chrono::NaiveDateTime,
    crossbeam_channel::Sender,
    dynotests::DynoTestDataInfo,
    ignore_err,
    reqwest::Client,
    users::{UserLogin, UserRegistration},
    ApiResponse, BufferData, CompresedSaver as _, DynoConfig, DynoErr, DynoResult, UserSession,
};
use eframe::epaint::mutex::Mutex;

use crate::AsyncMsg;

#[allow(unused)]
pub static SERVER_URL: &str = env!("API_SERVER_URL");
#[allow(unused)]
pub static SERVER_URL_API_ENDPOINT: &str = concat!(env!("API_SERVER_URL"), "/api");

static APP_USER_AGENT: &str = concat!(env!("CARGO_PKG_NAME"), "/", env!("CARGO_PKG_VERSION"),);

macro_rules! api_url {
    ($paths:literal) => {
        concat!(env!("API_SERVER_URL"), "/api", $paths)
    };
}

#[derive(Clone)]
pub struct ApiService {
    client: Client,
    token: Arc<Mutex<Option<String>>>,
    session: Arc<Mutex<Option<UserSession>>>,
}

impl ApiService {
    pub fn new() -> DynoResult<Self> {
        let client = Client::builder()
            .user_agent(APP_USER_AGENT)
            .build()
            .map_err(DynoErr::service_error)?;
        Ok(Self {
            client,
            token: Default::default(),
            session: Default::default(),
        })
    }
    pub fn is_logined(&self) -> bool {
        self.token.lock().is_some()
    }
}

impl ApiService {
    pub fn check_health(&self, tx: Sender<AsyncMsg>) {
        let client = self.client.clone();
        dyno_core::tokio::spawn(async move {
            match client
                .get(api_url!("/health"))
                .send()
                .await
                .map_err(DynoErr::service_error)
            {
                Ok(resp) => tx.send(AsyncMsg::check_health(resp.status())),
                Err(err) => tx.send(AsyncMsg::error(err)),
            }
        });
    }

    pub fn login(&self, login: UserLogin, tx: Sender<AsyncMsg>) {
        let client = self.client.clone();
        let token = self.token.clone();
        let user_session = self.session.clone();

        dyno_core::tokio::spawn(async move {
            match client
                .post(api_url!("/auth/login"))
                .json(&login)
                .send()
                .await
                .map_err(DynoErr::service_error)
            {
                Ok(resp) => {
                    match resp
                        .json::<ApiResponse<(UserSession, String)>>()
                        .await
                        .map_err(DynoErr::service_error)
                    {
                        Ok(user_session_resp) => {
                            if user_session_resp.status_ok() {
                                user_session.lock().replace(user_session_resp.payload.0);
                                token.lock().replace(user_session_resp.payload.1);
                                ignore_err!(tx.send(AsyncMsg::message("Login Success!")))
                            } else {
                                ignore_err!(tx.send(AsyncMsg::error(DynoErr::service_error(
                                    "Something wrong on user_session payload response"
                                ))))
                            }
                        }
                        Err(err) => ignore_err!(tx.send(AsyncMsg::error(err))),
                    }
                }
                Err(err) => ignore_err!(tx.send(AsyncMsg::error(err))),
            }
        });
    }

    pub fn register(&self, register: UserRegistration, tx: Sender<AsyncMsg>) {
        let client = self.client.clone();

        dyno_core::tokio::spawn(async move {
            match client
                .post(api_url!("/auth/register"))
                .json(&register)
                .send()
                .await
            {
                Ok(resp) => match resp.json::<ApiResponse<i32>>().await {
                    Ok(_resp) => {
                        ignore_err!(tx.send(AsyncMsg::message("Registration is Success!")))
                    }
                    Err(err) => ignore_err!(tx.send(AsyncMsg::error(err))),
                },
                Err(err) => ignore_err!(tx.send(AsyncMsg::error(err))),
            }
        });
    }

    pub fn logout(&self, tx: Sender<AsyncMsg>) {
        if !self.is_logined() {
            return;
        }

        let client = self.client.clone();
        dyno_core::tokio::spawn(async move {
            match client.get(api_url!("/auth/logout")).send().await {
                Ok(_resp) => ignore_err!(tx.send(AsyncMsg::message("Logout is Success!"))),
                Err(err) => ignore_err!(tx.send(AsyncMsg::error(err))),
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
        use dyno_core::{reqwest::multipart, serde_json, tokio};
        let uuid = match self.session.lock().as_ref().map(|x| x.uuid) {
            Some(uuid) => uuid,
            None => return,
        };

        let client = self.client.clone();
        tokio::spawn(async move {
            let (file_part, checksum_hex) =
                match asyncify!(move || data.compress().map(|compressed| {
                    let now = dyno_core::chrono::Utc::now().naive_utc();
                    let checksum = dyno_core::crypto::checksum_from_bytes(&compressed);
                    let compressed_len = compressed.len() as _;
                    (
                        multipart::Part::stream_with_length(compressed, compressed_len)
                            .file_name(format!("{uuid}-{}", now.format("%s")))
                            .mime_str("application/octet-stream"),
                        checksum,
                    )
                })) {
                    Ok(ok) => match ok.0 {
                        Ok(part) => (part, ok.1),
                        Err(err) => {
                            ignore_err!(tx.send(AsyncMsg::error(err)));
                            return;
                        }
                    },
                    Err(err) => {
                        ignore_err!(tx.send(AsyncMsg::error(err)));
                        return;
                    }
                };

            let info_part = match serde_json::to_vec(&DynoTestDataInfo {
                checksum_hex,
                config,
                start,
                stop,
            }) {
                Ok(info) => match multipart::Part::bytes(info).mime_str("application/json") {
                    Ok(part) => part,
                    Err(err) => {
                        ignore_err!(tx.send(AsyncMsg::error(err)));
                        return;
                    }
                },
                Err(err) => {
                    ignore_err!(tx.send(AsyncMsg::error(err)));
                    return;
                }
            };

            match client
                .post(api_url!("/dyno"))
                .multipart(
                    multipart::Form::new()
                        .part("data", file_part)
                        .part("info", info_part),
                )
                .send()
                .await
            {
                Ok(resp) => {
                    let status = resp.status();
                    if status.is_success() {
                        match resp.json::<ApiResponse<i32>>().await {
                            Ok(id) => ignore_err!(tx.send(AsyncMsg::message(format!(
                                "Save data is Success with id - {}",
                                id.payload
                            )))),
                            Err(err) => ignore_err!(tx.send(AsyncMsg::error(err))),
                        }
                    } else {
                        ignore_err!(tx.send(AsyncMsg::error(format!(
                            "Response Error with status: {status}"
                        ))))
                    }
                }
                Err(err) => ignore_err!(tx.send(AsyncMsg::error(err))),
            }
        });
    }
}
