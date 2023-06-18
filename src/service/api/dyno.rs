use dyno_core::{
    asyncify,
    crypto::{checksum_from_bytes, compare_checksums},
    dynotests::{DynoTest, DynoTestDataInfo},
    reqwest::{multipart, Client, IntoUrl, Response},
    ApiResponse, BufferData, CompresedSaver as _, DynoErr, DynoResult,
};

use crate::AsyncMsg;

#[inline]
pub(super) async fn get_info_part(config: DynoTestDataInfo) -> DynoResult<multipart::Part> {
    asyncify!(move || config.compress().and_then(|info| {
        let len = info.len() as _;
        multipart::Part::stream_with_length(info, len)
            .mime_str("application/json")
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
) -> Result<AsyncMsg, AsyncMsg> {
    match client
        .post(url)
        .multipart(multiparts)
        .bearer_auth(token)
        .send()
        .await
        .and_then(Response::error_for_status)
        .map_err(AsyncMsg::error)
    {
        Ok(resp) => match resp.json::<ApiResponse<i32>>().await {
            Ok(id) => Ok(AsyncMsg::message(format!(
                "Save data is Success with id {}",
                id.payload
            ))),
            Err(err) => Err(AsyncMsg::error(err)),
        },
        Err(err) => Err(err),
    }
}

pub async fn get(url: impl IntoUrl, client: Client, token: impl std::fmt::Display) -> AsyncMsg {
    match client
        .get(url)
        .bearer_auth(token)
        .send()
        .await
        .and_then(Response::error_for_status)
        .map_err(AsyncMsg::error)
    {
        Ok(resp) => match resp
            .json::<ApiResponse<Vec<DynoTest>>>()
            .await
            .map(|x| AsyncMsg::on_load_dyno(x.payload))
            .map_err(AsyncMsg::error)
        {
            Ok(ok) => ok,
            Err(err) => err,
        },
        Err(err) => err,
    }
}

pub async fn load_file(
    url: impl IntoUrl,
    client: Client,
    token: impl std::fmt::Display,
    checksum: impl AsRef<[u8]>,
) -> AsyncMsg {
    match client
        .get(url)
        .bearer_auth(token)
        .send()
        .await
        .and_then(Response::error_for_status)
        .map_err(AsyncMsg::error)
    {
        Ok(mut resp) => {
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
