use crate::AsyncMsg;
use dyno_core::{
    crypto::TokenDetails,
    reqwest::{Client, IntoUrl, Response},
    users::{UserLogin, UserRegistration},
    ApiResponse,
};

pub async fn user_login(
    url: impl IntoUrl,
    client: Client,
    login: UserLogin,
) -> Result<ApiResponse<TokenDetails>, AsyncMsg> {
    match client
        .post(url)
        .json(&login)
        .send()
        .await
        .and_then(Response::error_for_status)
        .map_err(AsyncMsg::error)
    {
        Ok(resp) => resp
            .json::<ApiResponse<TokenDetails>>()
            .await
            .map_err(AsyncMsg::error),
        Err(err) => Err(err),
    }
}

pub async fn user_register(
    url: impl IntoUrl,
    client: Client,
    register: UserRegistration,
) -> Result<ApiResponse<i32>, AsyncMsg> {
    match client
        .post(url)
        .json(&register)
        .send()
        .await
        .and_then(Response::error_for_status)
        .map_err(AsyncMsg::error)
    {
        Ok(resp) => resp
            .json::<ApiResponse<i32>>()
            .await
            .map_err(AsyncMsg::error),
        Err(err) => Err(err),
    }
}

pub async fn user_logout(
    url: impl IntoUrl,
    client: Client,
    token: impl std::fmt::Display,
) -> Result<AsyncMsg, AsyncMsg> {
    match client
        .get(url)
        .bearer_auth(token)
        .send()
        .await
        .and_then(Response::error_for_status)
        .map_err(AsyncMsg::error)
    {
        Ok(_resp) => Ok(AsyncMsg::message("Logout is Success!")),
        Err(err) => Err(err),
    }
}
