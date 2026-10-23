use whatsapp_rust::features::{GroupCreateOptions, GroupParticipantOptions, MemberAddMode};

use crate::data::{Context, Error};

pub async fn run(ctx: &Context, name: &str) -> Result<(), Error> {
    let owner = ctx
        .author_jids()
        .into_iter()
        .min_by_key(|jid| !jid.is_pn())
        .ok_or("não foi possível identificar o remetente")?;

    let options = GroupCreateOptions::builder()
        .subject(name)
        .participants(vec![GroupParticipantOptions {
            jid: owner.clone(),
            phone_number: None,
            privacy: None,
        }])
        .member_add_mode(MemberAddMode::AdminAdd)
        .build();

    let groups = ctx.message.client.groups();
    let result = groups.create_group(options).await?;
    let group = result.metadata.id.clone();

    groups.promote_participants(group.clone(), &[owner]).await?;
    groups.set_locked(group.clone(), true).await?;
    groups
        .set_member_add_mode(group, MemberAddMode::AdminAdd)
        .await?;

    ctx.reply(format!(
        "Grupo *{}* criado!\nID: `{}`\n\nVocê é administrador. Somente administradores editam os dados do grupo e adicionam membros.",
        result.metadata.subject, result.metadata.id
    ))
    .await?;

    Ok(())
}
