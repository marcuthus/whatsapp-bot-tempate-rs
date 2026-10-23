use crate::{
    commands::Command,
    data::{Context, Error},
};

pub async fn run(ctx: &Context) -> Result<(), Error> {
    ctx.reply(Command::menu(ctx.data.config.prefix, &ctx.author_name()))
        .await?;

    Ok(())
}
