use crate::data::{Context, Error};

pub async fn run(ctx: &Context) -> Result<(), Error> {
    let source = &ctx.message.info.source;

    if !source.is_group {
        ctx.reply("Este comando só funciona em grupos.").await?;
        return Ok(());
    }

    ctx.reply(format!("O ID deste grupo é:\n`{}`", source.chat))
        .await?;

    Ok(())
}
