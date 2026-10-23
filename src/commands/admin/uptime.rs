use crate::{
    data::{Context, Error},
    utils::checks::require_bot_owner,
};

pub async fn run(ctx: &Context) -> Result<(), Error> {
    require_bot_owner(ctx).await?;

    let seconds = ctx.data.started_at.elapsed().as_secs();

    ctx.reply(format!(
        "Online há `{}h {}m {}s`",
        seconds / 3600,
        seconds % 3600 / 60,
        seconds % 60
    ))
    .await?;

    Ok(())
}
