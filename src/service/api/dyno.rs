use crate::AsyncMsg;
use dyno_core::{
    asyncify,
    chrono::NaiveDateTime,
    crypto::{checksum_from_bytes, compare_checksums},
    dynotests::{DynoTest, DynoTestDataInfo},
    BufferData, CompresedSaver as _, DynoConfig, DynoErr, DynoResult,
};
use reqwest::{multipart, Client, IntoUrl};

use super::Resp;

#[inline]
pub(super) async fn get_parts(
    data: BufferData,
    config: DynoConfig,
    start: NaiveDateTime,
    stop: NaiveDateTime,
) -> DynoResult<multipart::Form> {
    let (checksum_hex, data) = asyncify!(move || data.compress().and_then(|comp| {
        let checksum = dyno_core::crypto::checksum_from_bytes(&comp);
        multipart::Part::bytes(comp)
            .file_name("data.dyno")
            .mime_str("application/octet-stream")
            .map_err(DynoErr::api_error)
            .map(|e| (checksum, e))
    }))?;
    let info_data = DynoTestDataInfo {
        checksum_hex,
        config,
        start,
        stop,
    };
    let info = asyncify!(move || info_data.compress().and_then(|comp| {
        multipart::Part::bytes(comp)
            .mime_str("application/octet-stream")
            .map_err(DynoErr::api_error)
    }))?;
    Ok(multipart::Form::default()
        .part("data", data)
        .part("info", info))
}

pub async fn save<U>(
    url: U,
    client: Client,
    token: impl std::fmt::Display,
    data: BufferData,
    config: DynoConfig,
    start: NaiveDateTime,
    stop: NaiveDateTime,
) -> AsyncMsg
where
    U: IntoUrl,
{
    let multiparts = match get_parts(data, config, start, stop).await {
        Ok(ok) => ok,
        Err(err) => return AsyncMsg::error(err),
    };
    let resp = client
        .post(url)
        .bearer_auth(token)
        .multipart(multiparts)
        .send()
        .await
        .map(Resp::from_resp)
        .map_err(super::map_resp_error);

    match resp {
        Ok(resp) => resp
            .get_json::<i32>()
            .await
            .map_or_else(AsyncMsg::error, |id| {
                AsyncMsg::message(format!("Save data is Success with id {id}"))
            }),
        Err(err) => err,
    }
}

pub async fn get<U>(url: U, client: Client, token: impl std::fmt::Display) -> AsyncMsg
where
    U: IntoUrl,
{
    let resp = client
        .get(url)
        .bearer_auth(token)
        .send()
        .await
        .map(Resp::from_resp)
        .map_err(super::map_resp_error);

    match resp {
        Ok(resp) => resp
            .get_json::<Vec<DynoTest>>()
            .await
            .map_or_else(AsyncMsg::error, AsyncMsg::on_load_dyno),
        Err(err) => err,
    }
}

pub async fn load_file<U>(
    url: U,
    client: Client,
    token: impl std::fmt::Display,
    checksum: impl AsRef<[u8]>,
) -> AsyncMsg
where
    U: IntoUrl,
{
    let resp = client
        .get(url)
        .bearer_auth(token)
        .send()
        .await
        .map(Resp::from_resp)
        .map_err(super::map_resp_error);

    match resp {
        Ok(Resp::Error(resp)) => {
            AsyncMsg::error(resp.text().await.unwrap_or("Failed on Request".to_owned()))
        }
        Ok(Resp::Success(resp)) => {
            let buffer_data = match resp.bytes().await {
                Ok(body) => body,
                Err(err) => return AsyncMsg::error(DynoErr::api_error(err)),
            };
            let data_checksum = checksum_from_bytes(&buffer_data);
            if !compare_checksums(data_checksum.as_bytes(), checksum.as_ref()) {
                return AsyncMsg::error("Data Checksum is not matching.");
            }
            match BufferData::decompress(buffer_data)
                .map_err(AsyncMsg::error)
                .map(AsyncMsg::open_buffer)
            {
                Ok(ok) => ok,
                Err(err) => err,
            }
        }
        Err(err) => err,
    }
}
