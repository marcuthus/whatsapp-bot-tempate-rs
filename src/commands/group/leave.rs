use crate::{
    data::{Context, Error},
    utils::checks::require_group_admin,
};

pub async fn run(ctx: &Context) -> Result<(), Error> {
    require_group_admin(ctx).await?;

    ctx.reply("Até mais! 👋").await?;

    ctx.message
        .client
        .groups()
        .leave(ctx.message.info.source.chat.clone())
        .await?;

    Ok(())
}
