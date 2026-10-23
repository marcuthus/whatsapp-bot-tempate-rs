use whatsapp_rust::prelude::*;

use crate::{
    data::{Context, Error},
    error::CommandError,
    utils::checks::is_bot_owner,
};

pub async fn run(ctx: &Context) -> Result<(), Error> {
    let source = &ctx.message.info.source;

    if !source.is_group {
        ctx.reply("Este comando só funciona em grupos.").await?;
        return Ok(());
    }

    let Some(target) = ctx.mentioned_jids().into_iter().next() else {
        ctx.reply("Mencione quem deve ser banido. Uso: !ban @pessoa")
            .await?;
        return Ok(());
    };

    let groups = ctx.message.client.groups();
    let metadata = groups.get_metadata(&source.chat).await?;
    let author_ids = ctx.author_ids();

    let is_group_admin = metadata.participants.iter().any(|participant| {
        participant.is_admin()
            && [
                Some(&participant.jid),
                participant.lid.as_ref(),
                participant.phone_number.as_ref(),
            ]
            .into_iter()
            .flatten()
            .any(|jid| belongs_to(jid, &author_ids))
    });

    if !is_group_admin && !is_bot_owner(ctx) {
        return Err(CommandError::PermissionDenied.into());
    }

    if belongs_to(&target, &author_ids) {
        ctx.reply("Você não pode banir a si mesmo.").await?;
        return Ok(());
    }

    let results = groups
        .remove_participants(source.chat.clone(), std::slice::from_ref(&target))
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

fn belongs_to(jid: &Jid, ids: &[String]) -> bool {
    ids.iter().any(|id| id == jid.user_base())
}
