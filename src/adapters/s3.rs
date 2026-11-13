use crate::api::context::RequestContext;
use crate::common::error::{ServiceResult, unexpected};
use crate::settings::AppSettings;
use aws_sdk_s3::primitives::ByteStream;
use axum::body::Bytes;
use tracing::warn;

pub async fn upload(
    ctx: &RequestContext,
    file_name: &str,
    folder: &str,
    file_contents: Bytes,
) -> ServiceResult<()> {
    let settings = AppSettings::get();
    match ctx.s3_client.as_ref() {
        Some(s3_client) if settings.aws_bucket_name.is_some() => {
            let key = format!("{folder}/{file_name}");
            s3_client
                .put_object()
                .bucket(settings.aws_bucket_name.as_ref().unwrap())
                .key(key)
                .body(ByteStream::from(file_contents))
                .send()
                .await?;
            Ok(())
        }
        _ => {
            warn!("S3 client not configured");
            Ok(())
        }
    }
}

/// Downloads a file, if it exists
pub async fn download(
    ctx: &RequestContext,
    file_name: &str,
    folder: &str,
) -> ServiceResult<Option<Vec<u8>>> {
    let settings = AppSettings::get();
    match ctx.s3_client.as_ref() {
        Some(s3_client) if settings.aws_bucket_name.is_some() => {
            let key = format!("{folder}/{file_name}");
            let result = s3_client
                .get_object()
                .bucket(settings.aws_bucket_name.as_ref().unwrap())
                .key(key)
                .send()
                .await;
            let object = match result {
                Ok(object) => object,
                Err(e) if e.as_service_error().is_some_and(|e| e.is_no_such_key()) => {
                    return Ok(None);
                }
                Err(e) => return unexpected(e),
            };

            let bytes = object.body.collect().await?;
            Ok(Some(bytes.to_vec()))
        }
        _ => {
            warn!("S3 client not configured");
            Ok(None)
        }
    }
}
