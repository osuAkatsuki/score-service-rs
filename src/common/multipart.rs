use axum::body::Bytes;
use axum::extract::{FromRequest, Multipart as MultipartExtractor, Request};
use axum::extract::multipart::{Field, MultipartError, MultipartRejection};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};

#[derive(Debug)]
pub struct Multipart<T>(pub T);

pub trait FromMultipart: Sized {
    fn from_multipart(form: MultipartExtractor) -> impl Future<Output = Result<Self, FromMultipartError>> + Send;
}

pub trait FromMultipartField: Sized {
    fn from_field(field: Field) -> impl Future<Output = Result<Self, FromMultipartError>> + Send;
}

pub enum FromMultipartError {
    MultipartRejection(MultipartRejection),
    MultipartParseError(MultipartError),
    MissingField(&'static str),
}

impl FromMultipartField for String {
    async fn from_field(field: Field<'_>) -> Result<Self, FromMultipartError> {
        let value = field.text().await?;
        Ok(value)
    }
}

impl FromMultipartField for Bytes {
    async fn from_field(field: Field<'_>) -> Result<Self, FromMultipartError> {
        let value = field.bytes().await?;
        Ok(value)
    }
}

impl IntoResponse for FromMultipartError {
    fn into_response(self) -> Response {
        match self {
            FromMultipartError::MultipartRejection(rejection) => rejection.into_response(),
            FromMultipartError::MultipartParseError(err) => err.into_response(),
            FromMultipartError::MissingField(_) => StatusCode::BAD_REQUEST.into_response(),
        }
    }
}

impl From<MultipartError> for FromMultipartError {
    fn from(err: MultipartError) -> Self {
        FromMultipartError::MultipartParseError(err)
    }
}

impl From<MultipartRejection> for FromMultipartError {
    fn from(err: MultipartRejection) -> Self {
        FromMultipartError::MultipartRejection(err)
    }
}

impl<S: Send + Sync, T: FromMultipart> FromRequest<S> for Multipart<T> {
    type Rejection = FromMultipartError;

    async fn from_request(req: Request, state: &S) -> Result<Self, Self::Rejection> {
        let form = MultipartExtractor::from_request(req, state).await?;
        T::from_multipart(form).await.map(Multipart)
    }
}