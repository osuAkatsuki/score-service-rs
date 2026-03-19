use axum::Json;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use serde::Serialize;
use tracing::error;

pub type ServiceResult<T> = Result<T, AppError>;
pub type ServiceResponse<T> = ServiceResult<Json<T>>;

#[derive(Debug)]
pub enum AppError {
    Unexpected,
    MalformedRequest,

    BeatmapsNotFound,

    PerformanceFailedCalculating,

    ScoresAnticheatTokenMissing,
    ScoresMissingReplay,
    ScoresIllegalScore,
    ScoresNotRanked,

    UsersNotFound,
    UsersInvalidCredentials,
}

#[cold]
#[track_caller]
pub fn unexpected<T, E: Into<anyhow::Error>>(e: E) -> ServiceResult<T> {
    let caller = std::panic::Location::caller();
    error!("An unexpected error has occurred at {caller}: {}", e.into());
    Err(AppError::Unexpected)
}

impl<E: Into<anyhow::Error>> From<E> for AppError {
    #[track_caller]
    fn from(e: E) -> Self {
        unexpected::<(), E>(e).unwrap_err()
    }
}

impl AppError {
    pub const fn code(&self) -> &'static str {
        todo!()
    }

    pub const fn message(&self) -> &'static str {
        todo!()
    }

    pub const fn http_status_code(&self) -> StatusCode {
        todo!()
    }

    pub const fn response_parts(&self) -> (StatusCode, Json<ErrorResponse>) {
        let status = self.http_status_code();
        let response = ErrorResponse {
            code: self.code(),
            message: self.message(),
        };
        (status, Json(response))
    }
}

#[derive(Serialize)]
pub struct ErrorResponse {
    pub code: &'static str,
    pub message: &'static str,
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        self.response_parts().into_response()
    }
}
