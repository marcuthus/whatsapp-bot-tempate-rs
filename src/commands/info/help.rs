use crate::{
    commands::Command,
    data::{Context, Error},
};

pub async fn run(ctx: &Context) -> Result<(), Error> {
    ctx.reply(Command::descriptions(ctx.data.config.prefix))
        .await?;

    Ok(())
}
