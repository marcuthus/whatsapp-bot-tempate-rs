use whatsapp_rust::prelude::*;

use crate::{
    data::{Context, Error},
    utils::checks::require_group_admin,
};

pub async fn run(ctx: &Context, number: &str) -> Result<(), Error> {
    require_group_admin(ctx).await?;

    let target = Jid::pn(number);

    let results = ctx
        .message
        .client
        .groups()
        .add_participants(
            ctx.message.info.source.chat.clone(),
            std::slice::from_ref(&target),
        )
        .await?;

    let outcome = match results.first().and_then(|result| result.error.as_deref()) {
        None => "foi adicionado ao grupo.".to_owned(),
        Some("403") => {
            "não pôde ser adicionado diretamente (privacidade). Envie um convite com !invite."
                .to_owned()
        }
        Some("408") => "saiu do grupo recentemente e não pôde ser adicionado.".to_owned(),
        Some("409") => "já está no grupo.".to_owned(),
        Some(code) => format!("não pôde ser adicionado (erro {code})."),
    };

    ctx.reply(format!("`{number}` {outcome}")).await?;

    Ok(())
}
