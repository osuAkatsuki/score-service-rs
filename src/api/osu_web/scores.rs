use crate::api::context::RequestContext;
use crate::common::error::{AppError, unexpected_err};
use crate::models::osu::leaderboards::{FetchLeaderboardArgs, UserLeaderboardView};
use crate::models::osu::score_submission::{
    ScoreSubmissionError, ScoreSubmissionResult, SubmitScoreArgs,
};
use crate::usecases::scores;
use axum::extract::Query;

pub async fn fetch_leaderboard(
    ctx: RequestContext,
    Query(args): Query<FetchLeaderboardArgs>,
) -> Result<UserLeaderboardView, &'static str> {
    scores::fetch_leaderboard_view(&ctx, args)
        .await
        .map_err(|_| "error")
}

pub async fn submit(ctx: RequestContext, args: SubmitScoreArgs) -> ScoreSubmissionResult {
    // We have to spawn the task since axum will cancel the handler task on connection close.
    let result = tokio::spawn(scores::submit(ctx, args))
        .await
        .map_err(|e| unexpected_err(e))
        .flatten();
    match result {
        Ok(result) => result,
        Err(AppError::BeatmapsNotFound) => {
            ScoreSubmissionResult::Err(ScoreSubmissionError::UnrankedBeatmap)
        }
        Err(AppError::UsersNotFound) => {
            ScoreSubmissionResult::Err(ScoreSubmissionError::InvalidUser)
        }
        Err(AppError::UsersInvalidCredentials) => {
            ScoreSubmissionResult::Err(ScoreSubmissionError::InvalidPassword)
        }
        Err(AppError::Unauthorized) => ScoreSubmissionResult::Err(ScoreSubmissionError::Disallowed),
        _ => ScoreSubmissionResult::Err(ScoreSubmissionError::Retry),
    }
}
