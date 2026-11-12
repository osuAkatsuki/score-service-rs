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
    ScoresNotFound,

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
        match self {
            AppError::Unexpected => "unexpected",
            AppError::MalformedRequest => "malformed_request",

            AppError::BeatmapsNotFound => "beatmaps.not_found",

            AppError::PerformanceFailedCalculating => "performance.failed_calculating",

            AppError::ScoresAnticheatTokenMissing => "scores.anticheat_token_missing",
            AppError::ScoresMissingReplay => "scores.missing_replay",
            AppError::ScoresIllegalScore => "scores.illegal_score",
            AppError::ScoresNotRanked => "scores.not_ranked",
            AppError::ScoresNotFound => "scores.not_found",

            AppError::UsersNotFound => "users.not_found",
            AppError::UsersInvalidCredentials => "users.invalid_credentials",
        }
    }

    pub const fn message(&self) -> &'static str {
        match self {
            AppError::Unexpected => "An unexpected error has occurred.",
            AppError::MalformedRequest => "The request is malformed.",

            AppError::BeatmapsNotFound => "Beatmap not found.",

            AppError::PerformanceFailedCalculating => "Failed to calculate performance.",

            AppError::ScoresAnticheatTokenMissing => "The anticheat token is missing.",
            AppError::ScoresMissingReplay => "The score is missing its replay.",
            AppError::ScoresIllegalScore => "The score is illegal.",
            AppError::ScoresNotRanked => "The score can not be ranked.",
            AppError::ScoresNotFound => "Score not found.",

            AppError::UsersNotFound => "User not found.",
            AppError::UsersInvalidCredentials => "Invalid credentials.",
        }
    }

    pub const fn http_status_code(&self) -> StatusCode {
        match self {
            AppError::Unexpected | AppError::PerformanceFailedCalculating => {
                StatusCode::INTERNAL_SERVER_ERROR
            }

            AppError::MalformedRequest
            | AppError::ScoresAnticheatTokenMissing
            | AppError::ScoresMissingReplay
            | AppError::ScoresIllegalScore
            | AppError::ScoresNotRanked => StatusCode::BAD_REQUEST,

            AppError::BeatmapsNotFound | AppError::ScoresNotFound | AppError::UsersNotFound => {
                StatusCode::NOT_FOUND
            }

            AppError::UsersInvalidCredentials => StatusCode::UNAUTHORIZED,
        }
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn responds_with_the_status_and_code_of_the_error() {
        let (status, Json(response)) = AppError::UsersInvalidCredentials.response_parts();
        assert_eq!(status, StatusCode::UNAUTHORIZED);
        assert_eq!(response.code, "users.invalid_credentials");
        assert_eq!(response.message, "Invalid credentials.");
    }

    #[test]
    fn converts_foreign_errors_into_unexpected_ones() {
        let error = AppError::from(std::io::Error::other("connection reset"));
        assert!(matches!(error, AppError::Unexpected));
        assert_eq!(error.http_status_code(), StatusCode::INTERNAL_SERVER_ERROR);
    }
}
