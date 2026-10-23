use std::time::Instant;

use crate::data::{Context, Error};

pub async fn run(ctx: &Context) -> Result<(), Error> {
    let started = Instant::now();
    let sent = ctx.reply("Calculando ping...").await?;
    let message_latency = started.elapsed().as_millis();

    let reply = format!(
        "*🏓 Pong!*\n\nOlá {}, meu ping está em `{message_latency}ms`\n\n_*👨‍💻 By Marcuth*_",
        ctx.author_name()
    );

    ctx.edit(&sent, reply).await?;
    ctx.react("🏓").await
}
