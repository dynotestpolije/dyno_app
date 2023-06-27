use crate::AsyncMsg;
use dyno_core::{
    asyncify,
    crypto::{checksum_from_bytes, compare_checksums},
    dynotests::{DynoTest, DynoTestDataInfo},
    reqwest::{multipart, Client, IntoUrl},
    ApiResponse, BufferData, CompresedSaver as _, DynoErr, DynoResult,
};

#[inline]
pub(super) async fn get_info_part(config: DynoTestDataInfo) -> DynoResult<multipart::Part> {
    asyncify!(move || config.compress().and_then(|info| {
        let len = info.len() as _;
        multipart::Part::stream_with_length(info, len)
            .file_name(dyno_core::uuid::Uuid::new_v4().simple().to_string())
            .mime_str("application/octet-stream")
            .map_err(DynoErr::service_error)
    }))
}

#[inline]
pub(super) async fn get_data_part(data: BufferData) -> DynoResult<(multipart::Part, String)> {
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

pub async fn save(
    url: impl IntoUrl,
    client: Client,
    token: impl std::fmt::Display,
    multiparts: multipart::Form,
) -> AsyncMsg {
    let resp = client
        .post(url)
        .bearer_auth(token)
        .multipart(multiparts)
        .send()
        .await
        .map(|resp| (resp.status().is_success(), resp))
        .map_err(AsyncMsg::error);

    match resp {
        Ok((false, resp)) => AsyncMsg::error(resp.text().await.unwrap_or("".to_owned())),
        Ok((true, resp)) => {
            if resp.status().is_success() {
                match resp.json::<ApiResponse<i32>>().await {
                    Ok(ApiResponse { payload, .. }) => {
                        AsyncMsg::message(format!("Save data is Success with id {payload}"))
                    }
                    Err(err) => AsyncMsg::error(err),
                }
            } else {
                match resp.text().await {
                    Ok(ok) => AsyncMsg::error(DynoErr::api_error(ok)),
                    Err(err) => AsyncMsg::error(err),
                }
            }
        }
        Err(err) => err,
    }
}

pub async fn get(url: impl IntoUrl, client: Client, token: impl std::fmt::Display) -> AsyncMsg {
    let resp = client
        .get(url)
        .bearer_auth(token)
        .send()
        .await
        .map(|resp| (resp.status().is_success(), resp))
        .map_err(AsyncMsg::error);

    match resp {
        Ok((false, resp)) => AsyncMsg::error(resp.text().await.unwrap_or("".to_owned())),
        Ok((true, resp)) => {
            if resp.status().is_success() {
                resp.json::<ApiResponse<Vec<DynoTest>>>()
                    .await
                    .map_or_else(AsyncMsg::error, |ApiResponse { payload, .. }| {
                        AsyncMsg::on_load_dyno(payload)
                    })
            } else {
                AsyncMsg::error(resp.text().await.unwrap_or("".to_owned()))
            }
        }
        Err(err) => err,
    }
}

pub async fn load_file(
    url: impl IntoUrl,
    client: Client,
    token: impl std::fmt::Display,
    checksum: impl AsRef<[u8]>,
) -> AsyncMsg {
    let resp = client
        .get(url)
        .bearer_auth(token)
        .send()
        .await
        .map(|resp| (resp.status().is_success(), resp))
        .map_err(AsyncMsg::error);

    match resp {
        Ok((false, resp)) => AsyncMsg::error(resp.text().await.unwrap_or("".to_owned())),
        Ok((true, mut resp)) => {
            let mut buffer_data = if let Some(lenght) = resp.content_length() {
                Vec::with_capacity(lenght as _)
            } else {
                vec![]
            };
            while let Ok(Some(chunk)) = resp.chunk().await {
                buffer_data.extend(chunk);
            }
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
