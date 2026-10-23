use whatsapp_rust::{features::GroupMetadata, prelude::*};

use crate::{
    data::{Context, Error},
    error::CommandError,
};

pub fn is_bot_owner_id(id: &str, admin_number: &str) -> bool {
    !admin_number.is_empty() && id == admin_number
}

pub fn is_bot_owner(ctx: &Context) -> bool {
    ctx.author_ids()
        .iter()
        .any(|id| is_bot_owner_id(id, &ctx.data.config.admin_number))
}

pub fn require_bot_owner(ctx: &Context) -> Result<(), Error> {
    if is_bot_owner(ctx) {
        Ok(())
    } else {
        Err(CommandError::PermissionDenied.into())
    }
}

pub fn belongs_to(jid: &Jid, ids: &[String]) -> bool {
    ids.iter().any(|id| id == jid.user_base())
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

    if is_group_admin || is_bot_owner(ctx) {
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
}
