use crate::data::{Context, Error};

pub async fn run(ctx: &Context, private_reply: bool) -> Result<(), Error> {
    let (heading, ids) = match ctx.mentioned_jids().into_iter().next() {
        Some(jid) => ("O ID da pessoa mencionada é", vec![jid]),
        None => ("O seu ID de usuário é", ctx.author_jids()),
    };

    let lines = ids
        .iter()
        .map(|jid| format!("`{jid}`"))
        .collect::<Vec<_>>()
        .join("\n");

    let text = format!("{heading}:\n{lines}");

    if private_reply && ctx.is_group() {
        ctx.send_privately(text).await?;
        ctx.react("📩").await?;
    } else {
        ctx.reply(text).await?;
    }

    Ok(())
}
