use super::api_url;
use crate::AsyncMsg;
use dyno_core::{
    crypto::TokenDetails,
    reqwest::{Client, Response},
    users::{UserLogin, UserRegistration},
    ApiResponse,
};

pub async fn user_login(
    client: Client,
    login: UserLogin,
) -> Result<ApiResponse<TokenDetails>, AsyncMsg> {
    match client
        .post(api_url!("/auth/login"))
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
    client: Client,
    register: UserRegistration,
) -> Result<ApiResponse<i32>, AsyncMsg> {
    match client
        .post(api_url!("/auth/register"))
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

pub async fn user_logout(client: Client) -> Result<AsyncMsg, AsyncMsg> {
    match client
        .get(api_url!("/auth/logout"))
        .send()
        .await
        .and_then(Response::error_for_status)
        .map_err(AsyncMsg::error)
    {
        Ok(_resp) => Ok(AsyncMsg::message("Logout is Success!")),
        Err(err) => Err(err),
    }
}
