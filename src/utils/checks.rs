use crate::{
    data::{Context, Error},
    error::CommandError,
};

pub fn is_bot_owner_id(id: &str, admin_number: &str) -> bool {
    !admin_number.is_empty() && id == admin_number
}

pub fn require_bot_owner(ctx: &Context) -> Result<(), Error> {
    let is_owner = ctx
        .author_ids()
        .iter()
        .any(|id| is_bot_owner_id(id, &ctx.data.config.admin_number));

    if is_owner {
        Ok(())
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
