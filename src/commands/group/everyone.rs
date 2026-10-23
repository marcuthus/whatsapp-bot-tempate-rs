use whatsapp_rust::prelude::*;

use crate::{
    data::{Context, Error},
    utils::checks::require_group_admin,
};

const MENTIONS_PER_MESSAGE: usize = 100;
const DEFAULT_ANNOUNCEMENT: &str = "📢 Atenção, pessoal!";

pub async fn run(ctx: &Context, announcement: &str) -> Result<(), Error> {
    let metadata = require_group_admin(ctx).await?;

    let announcement = if announcement.is_empty() {
        DEFAULT_ANNOUNCEMENT
    } else {
        announcement
    };

    let participants = metadata
        .participants
        .iter()
        .map(|participant| participant.jid.clone())
        .collect::<Vec<_>>();

    let users = participants
        .iter()
        .map(|jid| jid.user_base().to_owned())
        .collect::<Vec<_>>();

    for (page, jids) in build_pages(announcement, &users)
        .into_iter()
        .zip(participants.chunks(MENTIONS_PER_MESSAGE))
    {
        let context_info = wa::ContextInfo {
            mentioned_jid: jids.iter().map(ToString::to_string).collect(),
            ..Default::default()
        };

        ctx.message
            .send_message(wa::Message::text_with_context(page, context_info))
            .await?;
    }

    Ok(())
}

fn build_pages(announcement: &str, users: &[String]) -> Vec<String> {
    users
        .chunks(MENTIONS_PER_MESSAGE)
        .enumerate()
        .map(|(index, chunk)| {
            let mentions = chunk
                .iter()
                .map(|user| format!("@{user}"))
                .collect::<Vec<_>>()
                .join(" ");

            if index == 0 {
                format!("{announcement}\n\n{mentions}")
            } else {
                mentions
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn users(total: usize) -> Vec<String> {
        (0..total)
            .map(|index| (5511900000000 + index).to_string())
            .collect()
    }

    #[test]
    fn splits_the_mentions_into_pages_of_one_hundred() {
        let pages = build_pages("Reunião agora!", &users(230));

        assert_eq!(pages.len(), 3);
        assert_eq!(pages[0].matches('@').count(), 100);
        assert_eq!(pages[1].matches('@').count(), 100);
        assert_eq!(pages[2].matches('@').count(), 30);
    }

    #[test]
    fn puts_the_announcement_only_in_the_first_page() {
        let pages = build_pages("Reunião agora!", &users(101));

        assert!(pages[0].starts_with("Reunião agora!\n\n@5511900000000"));
        assert!(pages[1].starts_with('@'));
    }

    #[test]
    fn sends_nothing_for_an_empty_group() {
        assert!(build_pages("x", &[]).is_empty());
    }
}
