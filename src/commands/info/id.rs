use crate::data::{Context, Error};

pub async fn run(ctx: &Context) -> Result<(), Error> {
    let (heading, ids) = match ctx.mentioned_jids().into_iter().next() {
        Some(jid) => ("O ID da pessoa mencionada é", vec![jid]),
        None => ("O seu ID de usuário é", ctx.author_jids()),
    };

    let lines = ids
        .iter()
        .map(|jid| format!("`{jid}`"))
        .collect::<Vec<_>>()
        .join("\n");

    ctx.reply(format!("{heading}:\n{lines}")).await?;

    Ok(())
}
