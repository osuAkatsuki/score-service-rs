use axum::extract::Query;
use crate::api::context::RequestContext;
use crate::common::error::AppError;
use crate::models::osu::leaderboards::{FetchLeaderboardArgs, UserLeaderboardView};
use crate::models::osu::score_submission::{ScoreSubmissionResult, SubmitScoreArgs};
use crate::usecases::scores;

pub async fn fetch_leaderboard(
    ctx: RequestContext,
    Query(args): Query<FetchLeaderboardArgs>,
) -> Result<UserLeaderboardView, &'static str> {
    scores::fetch_leaderboard_view(&ctx, args).await.map_err(|_| "error")
}

const ERROR_RETRY: &str = "";
const ERROR_INVALID_USER: &str = "error: nouser";
const ERROR_INVALID_PASSWORD: &str = "error: pass";
const ERROR_USER_RESTRICTED: &str = "error: ban";
const ERROR_INVALID_BEATMAP: &str = "error: beatmap";
const ERROR_VERSION_TOO_OLD: &str = "error: oldver";
const ERROR_SUBMISSION_DISALLOWED: &str = "error: no";
const ERROR_SUBMISSION_DISABLED: &str = "error: disabled";
pub async fn submit(
    ctx: RequestContext,
    args: SubmitScoreArgs,
) -> Result<ScoreSubmissionResult, &'static str> {
    match scores::submit(&ctx, args).await {
        Ok(result) => Ok(result),
        Err(AppError::Unexpected) => Err(ERROR_RETRY),
        Err(AppError::UsersNotFound) => Err(ERROR_INVALID_USER),
        Err(AppError::UsersInvalidCredentials) => Err(ERROR_INVALID_PASSWORD),
        Err(AppError::BeatmapsNotFound | AppError::ScoresNotRanked) => Err(ERROR_INVALID_BEATMAP),
        Err(AppError::ScoresIllegalScore | AppError::ScoresMissingReplay) => {
            Err(ERROR_USER_RESTRICTED)
        }
        Err(AppError::MalformedRequest | AppError::ScoresAnticheatTokenMissing) => {
            Err(ERROR_SUBMISSION_DISALLOWED)
        }
        Err(AppError::PerformanceFailedCalculating | AppError::ScoresNotFound) => unreachable!(),
    }
}
