use crate::AsyncMsg;
use dyno_core::{
    crypto::TokenDetails,
    users::{UserLogin, UserRegistration},
};
use reqwest::{Client, IntoUrl};

use super::Resp;

pub async fn user_login<U>(
    url: U,
    client: Client,
    login: UserLogin,
) -> Result<TokenDetails, AsyncMsg>
where
    U: IntoUrl,
{
    let resp = client
        .post(url)
        .json(&login)
        .send()
        .await
        .map(Resp::from_resp)
        .map_err(super::map_resp_error);

    match resp {
        Ok(resp) => resp
            .get_json::<TokenDetails>()
            .await
            .map_err(AsyncMsg::error),
        Err(err) => Err(err),
    }
}

pub async fn user_register<U>(url: U, client: Client, register: UserRegistration) -> AsyncMsg
where
    U: IntoUrl,
{
    let resp = client
        .post(url)
        .json(&register)
        .send()
        .await
        .map(Resp::from_resp)
        .map_err(super::map_resp_error);

    match resp {
        Ok(resp) => resp
            .get_json::<i32>()
            .await
            .map_or_else(AsyncMsg::error, |_| AsyncMsg::OnApiRegister),
        Err(err) => err,
    }
}

pub async fn user_logout<U>(
    url: U,
    client: Client,
    token: impl std::fmt::Display,
) -> Result<AsyncMsg, AsyncMsg>
where
    U: IntoUrl,
{
    match client
        .get(url)
        .bearer_auth(token)
        .send()
        .await
        .map(Resp::from_resp)
        .map_err(super::map_resp_error)
    {
        Ok(Resp::Success(_)) => Ok(AsyncMsg::message("Logout is Success!")),
        Ok(Resp::Error(resp)) => Err(AsyncMsg::error(resp.text().await.unwrap_or("".to_owned()))),
        Err(err) => Err(err),
    }
}
