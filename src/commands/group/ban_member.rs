use crate::{
    data::{Context, Error},
    utils::checks::{belongs_to, require_group_admin},
};

pub async fn run(ctx: &Context) -> Result<(), Error> {
    let Some(target) = ctx.mentioned_jids().into_iter().next() else {
        ctx.reply("Mencione quem deve ser banido. Uso: !ban-member @pessoa")
            .await?;
        return Ok(());
    };

    require_group_admin(ctx).await?;

    if belongs_to(&target, &ctx.author_ids()) {
        ctx.reply("Você não pode banir a si mesmo.").await?;
        return Ok(());
    }

    let results = ctx
        .message
        .client
        .groups()
        .remove_participants(
            ctx.message.info.source.chat.clone(),
            std::slice::from_ref(&target),
        )
        .await?;

    let reply = match results.first() {
        Some(result) if result.is_ok() => format!("`{target}` foi removido do grupo."),
        Some(result) => format!(
            "Não foi possível banir `{target}` ({}). O bot precisa ser administrador do grupo.",
            result.error.as_deref().unwrap_or("erro desconhecido")
        ),
        None => format!("Não foi possível banir `{target}`."),
    };

    ctx.reply(reply).await?;

    Ok(())
}
