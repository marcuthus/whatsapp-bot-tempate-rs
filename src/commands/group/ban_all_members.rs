use whatsapp_rust::{features::ParticipantChangeResponse, prelude::*};

use crate::{
    data::{Context, Error},
    utils::checks::{belongs_to, protection_of, require_group_admin},
};

const BATCH_SIZE: usize = 50;
const MAX_LISTED_FAILURES: usize = 20;

type Failure = (String, String);

pub async fn run(ctx: &Context, confirmed: bool) -> Result<(), Error> {
    let metadata = require_group_admin(ctx).await?;
    let author_ids = ctx.author_ids();

    let mut targets: Vec<Jid> = Vec::new();

    for participant in &metadata.participants {
        let is_requester = [
            Some(&participant.jid),
            participant.lid.as_ref(),
            participant.phone_number.as_ref(),
        ]
        .into_iter()
        .flatten()
        .any(|jid| belongs_to(jid, &author_ids));

        if participant.is_admin()
            || is_requester
            || protection_of(ctx, &metadata, &participant.jid)
                .await
                .is_some()
        {
            continue;
        }

        targets.push(participant.jid.clone());
    }

    if targets.is_empty() {
        ctx.reply("Não há membros para banir: só restam administradores.")
            .await?;
        return Ok(());
    }

    if !confirmed {
        ctx.reply(format!(
            "⚠️ Isto removerá *{}* membros (administradores e você são poupados). Para continuar, envie:\n!ban-all-members confirm",
            targets.len()
        ))
        .await?;
        return Ok(());
    }

    let groups = ctx.message.client.groups();
    let chat = ctx.message.info.source.chat.clone();

    let mut removed = 0;
    let mut failures: Vec<Failure> = Vec::new();

    for batch in targets.chunks(BATCH_SIZE) {
        match groups.remove_participants(chat.clone(), batch).await {
            Ok(results) => {
                for participant in batch {
                    match find_result(&results, participant) {
                        Some(result) if result.is_ok() => removed += 1,
                        Some(result) => failures.push((
                            participant.user_base().to_owned(),
                            format!("erro {}", result.error.as_deref().unwrap_or("desconhecido")),
                        )),
                        None => failures.push((
                            participant.user_base().to_owned(),
                            "sem resposta do servidor".to_owned(),
                        )),
                    }
                }
            }
            Err(error) => failures.extend(
                batch
                    .iter()
                    .map(|participant| (participant.user_base().to_owned(), error.to_string())),
            ),
        }
    }

    ctx.reply(build_report(removed, &failures)).await?;

    Ok(())
}

fn find_result<'a>(
    results: &'a [ParticipantChangeResponse],
    participant: &Jid,
) -> Option<&'a ParticipantChangeResponse> {
    results
        .iter()
        .find(|result| result.jid.user_base() == participant.user_base())
}

fn build_report(removed: usize, failures: &[Failure]) -> String {
    let mut lines = vec![format!("✅ *Removidos:* {removed}")];

    if !failures.is_empty() {
        lines.push(String::new());
        lines.push(format!("❌ *Falharam ({}):*", failures.len()));
        lines.extend(
            failures
                .iter()
                .take(MAX_LISTED_FAILURES)
                .map(|(user, reason)| format!("• `{user}` - {reason}")),
        );

        if failures.len() > MAX_LISTED_FAILURES {
            lines.push(format!(
                "• ... e mais {}",
                failures.len() - MAX_LISTED_FAILURES
            ));
        }
    }

    lines.join("\n")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn failures(total: usize) -> Vec<Failure> {
        (0..total)
            .map(|index| (format!("u{index}"), "erro 403".to_owned()))
            .collect()
    }

    #[test]
    fn reports_only_the_removed_count_when_nothing_failed() {
        assert_eq!(build_report(3, &[]), "✅ *Removidos:* 3");
    }

    #[test]
    fn lists_the_failures_with_their_reasons() {
        let report = build_report(1, &failures(2));

        assert!(report.contains("❌ *Falharam (2):*"));
        assert!(report.contains("• `u0` - erro 403"));
        assert!(report.contains("• `u1` - erro 403"));
    }

    #[test]
    fn truncates_a_long_list_of_failures() {
        let report = build_report(0, &failures(MAX_LISTED_FAILURES + 2));

        assert!(report.contains(&format!("• `u{}`", MAX_LISTED_FAILURES - 1)));
        assert!(!report.contains(&format!("• `u{MAX_LISTED_FAILURES}`")));
        assert!(report.ends_with("• ... e mais 2"));
    }
}
