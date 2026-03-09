use axum::body::Bytes;
use axum::extract::Multipart;
use crate::common::multipart::{FromMultipart, FromMultipartError, FromMultipartField};

#[derive(Debug)]
pub struct ScreenshotForm {
    pub u: String,
    pub p: String,
    pub v: String,
    pub ss: Bytes,
}

impl FromMultipart for ScreenshotForm {
    async fn from_multipart(mut form: Multipart) -> Result<Self, FromMultipartError> {
        let mut u = None;
        let mut p = None;
        let mut v = None;
        let mut ss = None;

        while let Some(field) =form.next_field().await? {
            let name = field.name();
            match name {
                Some("u") => u = Some(String::from_field(field).await?),
                Some("p") => p = Some(String::from_field(field).await?),
                Some("v") => v = Some(String::from_field(field).await?),
                Some("ss") => ss = Some(Bytes::from_field(field).await?),
                _ => continue,
            }
        }

        Ok(Self {
            u: u.ok_or(FromMultipartError::MissingField("u"))?,
            p: p.ok_or(FromMultipartError::MissingField("p"))?,
            v: v.ok_or(FromMultipartError::MissingField("v"))?,
            ss: ss.ok_or(FromMultipartError::MissingField("ss"))?,
        })
    }
}