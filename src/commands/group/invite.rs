use whatsapp_rust::prelude::*;

use crate::{
    data::{Context, Error},
    utils::checks::require_group_admin,
};

pub async fn run(ctx: &Context, number: Option<&str>) -> Result<(), Error> {
    let metadata = require_group_admin(ctx).await?;

    let link = ctx
        .message
        .client
        .groups()
        .get_invite_link(ctx.message.info.source.chat.clone(), false)
        .await?;

    let Some(number) = number else {
        ctx.reply(format!(
            "Link de convite de *{}*:\n{link}",
            metadata.subject
        ))
        .await?;
        return Ok(());
    };

    let invitation = format!(
        "Você foi convidado para o grupo *{}*:\n{link}",
        metadata.subject
    );

    ctx.message
        .client
        .send_message(Jid::pn(number), wa::Message::text(invitation))
        .await?;

    ctx.reply(format!("Convite enviado para `{number}`."))
        .await?;

    Ok(())
}
