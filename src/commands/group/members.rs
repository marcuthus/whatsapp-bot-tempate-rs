use whatsapp_rust::prelude::*;

use crate::data::{Context, Error};

const MEMBERS_PER_MESSAGE: usize = 40;

pub async fn run(ctx: &Context, private_reply: bool) -> Result<(), Error> {
    let source = &ctx.message.info.source;

    if !source.is_group {
        ctx.reply("Este comando só funciona em grupos.").await?;
        return Ok(());
    }

    let metadata = ctx
        .message
        .client
        .groups()
        .get_metadata(&source.chat)
        .await?;

    let lines = metadata
        .participants
        .iter()
        .enumerate()
        .map(|(index, participant)| {
            let role = if participant.is_admin() { "👑 " } else { "" };
            let phone = participant
                .phone_number
                .as_ref()
                .map(|jid| format!(" | tel: `{}`", jid.user_base()))
                .unwrap_or_default();

            format!("{}. {role}`{}`{phone}", index + 1, participant.jid)
        })
        .collect::<Vec<_>>();

    let title = format!("👥 {} ({} membros)", metadata.subject, lines.len());

    for page in paginate(&title, &lines) {
        if private_reply {
            ctx.send_privately(page).await?;
        } else {
            ctx.message.send_message(wa::Message::text(page)).await?;
        }
    }

    if private_reply {
        ctx.react("📩").await?;
    }

    Ok(())
}

fn paginate(title: &str, lines: &[String]) -> Vec<String> {
    lines
        .chunks(MEMBERS_PER_MESSAGE)
        .map(|chunk| format!("*{title}*\n\n{}", chunk.join("\n")))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn splits_members_into_pages() {
        let lines = (1..=MEMBERS_PER_MESSAGE + 1)
            .map(|index| index.to_string())
            .collect::<Vec<_>>();

        let pages = paginate("Grupo", &lines);

        assert_eq!(pages.len(), 2);
        assert!(pages[0].starts_with("*Grupo*"));
        assert!(pages[1].ends_with(&(MEMBERS_PER_MESSAGE + 1).to_string()));
    }

    #[test]
    fn sends_nothing_for_an_empty_group() {
        assert!(paginate("Grupo", &[]).is_empty());
    }
}
