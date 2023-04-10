use std::sync::mpsc;

use super::{handler::RequestStatus, URL_SERVER};
use dyno_types::{
    server::{ApiResponse, Login, Session, SIGN_IN_URL, SIGN_OUT_URL},
    tokio::spawn,
};

pub struct LoginHandler {
    receiv: mpsc::Receiver<RequestStatus>,
    sender: mpsc::Sender<RequestStatus>,
    login_info: Login,
}

impl LoginHandler {
    pub fn new(login_info: Login) -> Self {
        let (sender, receiv) = mpsc::channel();
        Self {
            sender,
            receiv,
            login_info,
        }
    }
}

impl super::handler::ApiHandler for LoginHandler {
    fn start_request(&self, connection: &super::ApiConnection) {
        dyno_types::log::trace!(
            "async client => login endpoint [json: {json}]",
            json = &self.login_info
        );

        let mut conn = connection.clone();
        let mut url = URL_SERVER.clone();
        url.push_str(SIGN_IN_URL);

        let json = self.login_info.clone();
        let sender = self.sender.clone();

        let async_login_op = async move {
            let response = conn.post(&url, Some(json)).await;
            match response {
                Ok(resp) => {
                    let status = resp.status();
                    if !status.is_success() {
                        return sender
                            .send(
                                format!("request responses is not successed with {}", status)
                                    .into(),
                            )
                            .ok();
                    }
                    match resp.json::<ApiResponse<Session>>().await {
                        Ok(json) => {
                            if let Ok(mut sess) = conn.session.lock() {
                                *sess = Some(json.data);
                                sender.send(RequestStatus::Success).ok()
                            } else {
                                sender.send("connection session is locked".into()).ok()
                            }
                        }
                        Err(err) => sender.send(err.into()).ok(),
                    }
                }
                Err(err) => sender.send(err.into()).ok(),
            }
        };
        spawn(async_login_op);
    }

    fn status_request(&self) -> RequestStatus {
        if let Ok(status) = self.receiv.try_recv() {
            status
        } else {
            RequestStatus::Processing
        }
    }
}

pub struct LogoutHandler {
    receiv: mpsc::Receiver<RequestStatus>,
    sender: mpsc::Sender<RequestStatus>,
}

impl LogoutHandler {
    pub fn new() -> Self {
        let (sender, receiv) = mpsc::channel();
        Self { sender, receiv }
    }
}

impl super::handler::ApiHandler for LogoutHandler {
    fn start_request(&self, connection: &super::ApiConnection) {
        let mut conn = connection.clone();
        let mut url = URL_SERVER.clone();
        url.push_str(SIGN_OUT_URL);

        let sender = self.sender.clone();
        let sess = if let Ok(sess) = connection.session.lock() {
            sess.clone()
        } else {
            None
        };

        let async_login_op = async move {
            let response = conn.post(&url, sess).await;
            match response {
                Ok(resp) => {
                    let status = resp.status();
                    if !status.is_success() {
                        return sender
                            .send(
                                format!("request responses is not successed with {}", status)
                                    .into(),
                            )
                            .ok();
                    }
                    match resp.json::<ApiResponse<Session>>().await {
                        Ok(json) => {
                            if let Ok(mut sess) = conn.session.lock() {
                                *sess = Some(json.data);
                                sender.send(RequestStatus::Success).ok()
                            } else {
                                sender.send("connection session is locked".into()).ok()
                            }
                        }
                        Err(err) => sender.send(err.into()).ok(),
                    }
                }
                Err(err) => sender.send(err.into()).ok(),
            }
        };
        spawn(async_login_op);
    }

    fn status_request(&self) -> RequestStatus {
        if let Ok(status) = self.receiv.try_recv() {
            status
        } else {
            RequestStatus::Processing
        }
    }
}
