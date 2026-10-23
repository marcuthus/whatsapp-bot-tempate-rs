use tracing::debug;
use whatsapp_rust::{features::GroupMetadata, prelude::*};

use crate::{
    data::{Context, Error},
    error::CommandError,
    utils::phone,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Protected {
    Bot,
    Owner,
}

pub fn is_bot_owner_id(id: &str, admin_number: &str) -> bool {
    phone::same_number(id, admin_number)
}

async fn resolve_numbers(ctx: &Context, jids: &[Jid]) -> Vec<String> {
    let mut numbers: Vec<String> = Vec::new();

    for jid in jids {
        let mut candidates = vec![jid.user_base().to_owned()];

        if let Ok(Some(entry)) = ctx.message.client.get_lid_pn_entry(&jid.to_non_ad()).await {
            candidates.push(entry.phone_number.to_string());
        }

        for candidate in candidates {
            if !numbers.contains(&candidate) {
                numbers.push(candidate);
            }
        }
    }

    numbers
}

pub async fn is_bot_owner(ctx: &Context) -> bool {
    let numbers = resolve_numbers(ctx, &ctx.author_jids()).await;

    let is_owner = numbers
        .iter()
        .any(|number| is_bot_owner_id(number, &ctx.data.config.admin_number));

    if !is_owner {
        debug!(
            target: "client",
            "The sender {numbers:?} is not the bot owner ({}).",
            ctx.data.config.admin_number
        );
    }

    is_owner
}

pub async fn require_bot_owner(ctx: &Context) -> Result<(), Error> {
    if is_bot_owner(ctx).await {
        Ok(())
    } else {
        Err(CommandError::PermissionDenied.into())
    }
}

pub fn belongs_to(jid: &Jid, ids: &[String]) -> bool {
    ids.iter().any(|id| id == jid.user_base())
}

pub async fn identities_of(ctx: &Context, metadata: &GroupMetadata, jid: &Jid) -> Vec<String> {
    let mut jids = vec![jid.clone()];

    if let Some(participant) = metadata.participants.iter().find(|participant| {
        [
            Some(&participant.jid),
            participant.lid.as_ref(),
            participant.phone_number.as_ref(),
        ]
        .into_iter()
        .flatten()
        .any(|known| known.user_base() == jid.user_base())
    }) {
        jids.push(participant.jid.clone());
        jids.extend(participant.lid.clone());
        jids.extend(participant.phone_number.clone());
    }

    resolve_numbers(ctx, &jids).await
}

pub async fn protection_of(
    ctx: &Context,
    metadata: &GroupMetadata,
    jid: &Jid,
) -> Option<Protected> {
    let identities = identities_of(ctx, metadata, jid).await;

    let client = &ctx.message.client;
    let bot_ids = [client.pn(), client.lid()]
        .into_iter()
        .flatten()
        .map(|own| own.user_base().to_owned())
        .collect::<Vec<_>>();

    if identities
        .iter()
        .any(|identity| bot_ids.iter().any(|bot| phone::same_number(identity, bot)))
    {
        return Some(Protected::Bot);
    }

    identities
        .iter()
        .any(|identity| is_bot_owner_id(identity, &ctx.data.config.admin_number))
        .then_some(Protected::Owner)
}

async fn is_bot_owner_in_group(ctx: &Context, metadata: &GroupMetadata) -> bool {
    if is_bot_owner(ctx).await {
        return true;
    }

    for author in ctx.author_jids() {
        let identities = identities_of(ctx, metadata, &author).await;

        if identities
            .iter()
            .any(|identity| is_bot_owner_id(identity, &ctx.data.config.admin_number))
        {
            return true;
        }
    }

    false
}

pub async fn require_group_admin(ctx: &Context) -> Result<GroupMetadata, Error> {
    let source = &ctx.message.info.source;

    if !source.is_group {
        return Err(CommandError::NotInGroup.into());
    }

    let metadata = ctx
        .message
        .client
        .groups()
        .get_metadata(&source.chat)
        .await?;

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

    if is_group_admin || is_bot_owner_in_group(ctx, &metadata).await {
        Ok(metadata)
    } else {
        Err(CommandError::PermissionDenied.into())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn matches_only_the_configured_admin() {
        assert!(is_bot_owner_id("5511999999999", "5511999999999"));
        assert!(!is_bot_owner_id("5511888888888", "5511999999999"));
        assert!(!is_bot_owner_id("", ""));
    }

    #[test]
    fn matches_the_admin_with_or_without_the_brazilian_ninth_digit() {
        assert!(is_bot_owner_id("551199999999", "5511999999999"));
        assert!(is_bot_owner_id("5511999999999", "551199999999"));
        assert!(is_bot_owner_id("5511999999999", "+55 (11) 99999-9999"));
    }
}
