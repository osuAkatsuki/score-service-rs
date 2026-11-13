use crate::common::multipart::{FromMultipart, FromMultipartError, FromMultipartField};
use axum::body::Bytes;
use axum::extract::Multipart;
use axum::http::header::CONTENT_TYPE;
use axum::response::{IntoResponse, Response};

#[derive(Debug)]
pub struct ScreenshotForm {
    pub u: String,
    pub p: String,
    pub v: String,
    pub ss: Bytes,
}

/// The image formats the osu! client takes screenshots in
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum ScreenshotFormat {
    Jpeg,
    Png,
}

impl ScreenshotFormat {
    const PNG_SIGNATURE: &'static [u8] = b"\x89PNG\r\n\x1a\n";

    /// Recognises the format of an image by the header of the file
    pub fn detect(contents: &[u8]) -> Option<Self> {
        if contents.starts_with(Self::PNG_SIGNATURE) {
            Some(ScreenshotFormat::Png)
        } else if matches!(contents.get(6..10), Some(b"JFIF" | b"Exif")) {
            Some(ScreenshotFormat::Jpeg)
        } else {
            None
        }
    }

    pub const fn extension(&self) -> &'static str {
        match self {
            ScreenshotFormat::Jpeg => "jpeg",
            ScreenshotFormat::Png => "png",
        }
    }

    pub const fn content_type(&self) -> &'static str {
        match self {
            ScreenshotFormat::Jpeg => "image/jpeg",
            ScreenshotFormat::Png => "image/png",
        }
    }
}

pub struct Screenshot {
    pub format: ScreenshotFormat,
    pub contents: Vec<u8>,
}

impl IntoResponse for Screenshot {
    fn into_response(self) -> Response {
        let headers = [(CONTENT_TYPE, self.format.content_type())];
        (headers, self.contents).into_response()
    }
}

impl FromMultipart for ScreenshotForm {
    async fn from_multipart(mut form: Multipart) -> Result<Self, FromMultipartError> {
        let mut u = None;
        let mut p = None;
        let mut v = None;
        let mut ss = None;

        while let Some(field) = form.next_field().await? {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detects_the_formats_of_screenshots() {
        let png = b"\x89PNG\r\n\x1a\n\x00\x00\x00\rIHDR";
        assert_eq!(ScreenshotFormat::detect(png), Some(ScreenshotFormat::Png));

        let jfif = b"\xff\xd8\xff\xe0\x00\x10JFIF\x00\x01";
        assert_eq!(ScreenshotFormat::detect(jfif), Some(ScreenshotFormat::Jpeg));

        let exif = b"\xff\xd8\xff\xe1\x00\x10Exif\x00\x00";
        assert_eq!(ScreenshotFormat::detect(exif), Some(ScreenshotFormat::Jpeg));
    }

    #[test]
    fn rejects_files_that_are_not_screenshots() {
        assert_eq!(ScreenshotFormat::detect(b""), None);
        assert_eq!(ScreenshotFormat::detect(b"GIF89a\x01\x00\x01\x00"), None);
        assert_eq!(ScreenshotFormat::detect(b"<html></html>"), None);
    }
}
