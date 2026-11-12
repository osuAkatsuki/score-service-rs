use crate::adapters::beatmaps_service;
use crate::adapters::beatmaps_service::BeatmapsetSearchArgs;
use crate::api::context::RequestContext;
use crate::common::error::{AppError, ServiceResult};
use crate::models::osu::beatmaps::{
    DirectBeatmapset, DirectBeatmapsetArgs, DirectSearchArgs, DirectSearchView,
};
use crate::usecases::{beatmaps, users};

const PAGE_SIZE: i32 = 100;

/// The tabs osu!direct opens with are sent as if they were searched for
const DEFAULT_QUERIES: [&str; 3] = ["Newest", "Top Rated", "Most Played"];

/// Translates the status filter of osu!direct into the one of the beatmap mirror
fn mirror_status(direct_status: i32) -> Option<i32> {
    match direct_status {
        // Ranked and ranked beatmaps the user has played before
        0 | 7 => Some(1),
        // Pending and graveyarded
        2 | 5 => Some(0),
        // Qualified
        3 => Some(3),
        // Loved
        8 => Some(4),
        // Any status
        _ => None,
    }
}

pub async fn search(ctx: &RequestContext, args: DirectSearchArgs) -> ServiceResult<DirectSearchView> {
    users::fetch_one_and_auth(ctx, &args.username, &args.password_md5).await?;

    let is_default_query = DEFAULT_QUERIES.contains(&args.query.as_str());
    let search_args = BeatmapsetSearchArgs {
        query: (!is_default_query).then_some(&args.query),
        mode: (args.mode != -1).then_some(args.mode),
        status: mirror_status(args.ranked_status),
        // The mirror derives the page from the offset of its last result
        offset: (args.page + 1) * PAGE_SIZE - 1,
        amount: PAGE_SIZE,
    };
    let results = beatmaps_service::search_beatmapsets(search_args).await?;

    let result_count = results.len();
    let beatmapsets = results
        .into_iter()
        .filter(|beatmapset| !beatmapset.children_beatmaps.is_empty())
        .map(DirectBeatmapset::from)
        .collect();

    Ok(DirectSearchView {
        result_count,
        beatmapsets,
        page_size: PAGE_SIZE as _,
    })
}

/// Fetches a beatmapset by its id or by the id of one of its beatmaps
pub async fn fetch_beatmapset(
    ctx: &RequestContext,
    args: DirectBeatmapsetArgs,
) -> ServiceResult<DirectBeatmapset> {
    users::fetch_one_and_auth(ctx, &args.username, &args.password_md5).await?;

    let beatmapset_id = match (args.beatmapset_id, args.beatmap_id) {
        (Some(beatmapset_id), _) => beatmapset_id,
        (None, Some(beatmap_id)) => beatmaps::fetch_by_id(beatmap_id).await?.beatmap_set_id,
        (None, None) => return Err(AppError::MalformedRequest),
    };

    let beatmapset = beatmaps_service::fetch_beatmapset(beatmapset_id).await?;
    Ok(DirectBeatmapset::from(beatmapset))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn translates_the_status_filters_of_the_client() {
        assert_eq!(mirror_status(0), Some(1));
        assert_eq!(mirror_status(7), Some(1));
        assert_eq!(mirror_status(2), Some(0));
        assert_eq!(mirror_status(5), Some(0));
        assert_eq!(mirror_status(3), Some(3));
        assert_eq!(mirror_status(8), Some(4));
        assert_eq!(mirror_status(4), None);
    }
}
