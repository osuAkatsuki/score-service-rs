use crate::api::context::RequestContext;
use crate::models::scores::SubmitScoreArgs;

pub async fn fetch(ctx: RequestContext) {
    // TODO: Implement score fetching
}

pub async fn create(
    ctx: RequestContext,
    args: SubmitScoreArgs,
) {
    println!("{:?}", args.score);
    println!("{:?}", args.visual_settings);
}
