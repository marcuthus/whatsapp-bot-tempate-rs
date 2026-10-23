use crate::data::{Context, Error};

pub async fn run(ctx: &Context) -> Result<(), Error> {
    let Some(id) = ctx.author_ids().into_iter().next() else {
        return Ok(());
    };

    ctx.reply(format!("O seu ID de usuário é `{id}`")).await?;

    Ok(())
}
