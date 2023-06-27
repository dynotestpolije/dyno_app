use crate::AsyncMsg;
use dyno_core::{
    crypto::TokenDetails,
    reqwest::{Client, IntoUrl},
    users::{UserLogin, UserRegistration},
    ApiResponse,
};

pub async fn user_login(
    url: impl IntoUrl,
    client: Client,
    login: UserLogin,
) -> Result<ApiResponse<TokenDetails>, AsyncMsg> {
    let resp = client
        .post(url)
        .json(&login)
        .send()
        .await
        .map(|resp| (resp.status().is_success(), resp))
        .map_err(AsyncMsg::error);

    match resp {
        Ok((false, resp)) => Err(AsyncMsg::error(resp.text().await.unwrap_or("".to_owned()))),
        Ok((true, resp)) => resp
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
    let resp = client
        .post(url)
        .json(&register)
        .send()
        .await
        .map(|resp| (resp.status().is_success(), resp))
        .map_err(AsyncMsg::error);

    match resp {
        Ok((false, resp)) => Err(AsyncMsg::error(resp.text().await.unwrap_or("".to_owned()))),
        Ok((true, resp)) => resp
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
        .map(|resp| (resp.status().is_success(), resp))
        .map_err(AsyncMsg::error)
    {
        Ok((true, _resp)) => Ok(AsyncMsg::message("Logout is Success!")),
        Ok((false, resp)) => Err(AsyncMsg::error(resp.text().await.unwrap_or("".to_owned()))),
        Err(err) => Err(err),
    }
}
