use std::sync::{Arc, Mutex};

use dyno_types::{
    reqwest::{header::HeaderValue, redirect::Policy, Client, Response},
    server::{Session, APP_USER_AGENT},
    DynoResult,
};

#[derive(Debug, Clone)]
pub struct ApiConnection {
    pub client: Client,
    pub session: Arc<Mutex<Option<Session>>>,
}

impl ApiConnection {
    pub fn new() -> DynoResult<Self> {
        let client = Client::builder()
            .redirect(Policy::none())
            .user_agent(APP_USER_AGENT)
            .build()?;
        Ok(Self {
            client,
            session: Arc::new(Mutex::new(None)),
        })
    }
}

impl ApiConnection {
    pub fn is_logged(&self) -> bool {
        if let Ok(sess) = self.session.lock() {
            sess.is_some()
        } else {
            false
        }
    }
}

macro_rules! impl_api_protocol {
    ($($name:ident),*) => {
        $(
            #[inline]
            pub async fn $name<Ser>(&mut self, url: &str, json: Option<Ser>) -> DynoResult<Response>
            where
                Ser: serde::Serialize,
            {
                let mut client = self.client.$name(url).header("DESKTOP-WORKSPACE", 1);
                if let Ok(Some(sess)) = self.session.lock().map(|m| m.as_ref().map(|s| s.verifier.clone())) {
                    if let Ok(value) = HeaderValue::from_str(sess.as_str()) {
                        client = client.header("Authorization", value);
                    }
                }
                if let Some(json) = json {
                    client = client.json(&json);
                }

                client.send().await.map_err(From::from)
            }
        )*
    };
}
impl ApiConnection {
    impl_api_protocol!(get, post, put, delete, patch);
}
