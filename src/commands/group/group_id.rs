use crate::data::{Context, Error};

pub async fn run(ctx: &Context, private_reply: bool) -> Result<(), Error> {
    let source = &ctx.message.info.source;

    if !source.is_group {
        ctx.reply("Este comando só funciona em grupos.").await?;
        return Ok(());
    }

    let text = format!("O ID deste grupo é:\n`{}`", source.chat);

    if private_reply {
        ctx.send_privately(text).await?;
        ctx.react("📩").await?;
    } else {
        ctx.reply(text).await?;
    }

    Ok(())
}
