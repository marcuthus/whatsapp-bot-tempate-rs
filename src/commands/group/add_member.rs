use whatsapp_rust::{features::ParticipantChangeResponse, prelude::*};

use crate::{
    data::{Context, Error},
    utils::checks::require_group_admin,
};

type Outcome = (String, Option<String>);

pub async fn run(ctx: &Context, number: Option<&str>) -> Result<(), Error> {
    let mut numbers = number.map(str::to_owned).into_iter().collect::<Vec<_>>();

    for contact in ctx.quoted_contact_numbers() {
        if !numbers.contains(&contact) {
            numbers.push(contact);
        }
    }

    if numbers.is_empty() {
        ctx.reply(
            "Informe um número (!add-member 5511999999999) ou responda uma mensagem de contato com !add-member.",
        )
        .await?;
        return Ok(());
    }

    require_group_admin(ctx).await?;

    let targets = numbers
        .iter()
        .map(|number| Jid::pn(number.as_str()))
        .collect::<Vec<_>>();

    let results = ctx
        .message
        .client
        .groups()
        .add_participants(ctx.message.info.source.chat.clone(), &targets)
        .await?;

    let outcomes = numbers
        .iter()
        .enumerate()
        .map(|(index, number)| {
            let result = find_result(&results, number)
                .or_else(|| (results.len() == numbers.len()).then(|| &results[index]));

            (number.clone(), failure_reason(result))
        })
        .collect::<Vec<_>>();

    ctx.reply(build_report(&outcomes)).await?;

    Ok(())
}

fn find_result<'a>(
    results: &'a [ParticipantChangeResponse],
    number: &str,
) -> Option<&'a ParticipantChangeResponse> {
    results.iter().find(|result| {
        result.jid.user_base() == number
            || result
                .phone_number
                .as_ref()
                .is_some_and(|phone| phone.user_base() == number)
    })
}

fn failure_reason(result: Option<&ParticipantChangeResponse>) -> Option<String> {
    match result {
        None => Some("sem resposta do servidor".to_owned()),
        Some(result) => result.error.as_deref().map(describe_error),
    }
}

fn describe_error(code: &str) -> String {
    match code {
        "403" => "privacidade do contato (use !invite)".to_owned(),
        "408" => "saiu do grupo recentemente".to_owned(),
        "409" => "já está no grupo".to_owned(),
        other => format!("erro {other}"),
    }
}

fn build_report(outcomes: &[Outcome]) -> String {
    let added = outcomes
        .iter()
        .filter(|(_, reason)| reason.is_none())
        .map(|(number, _)| format!("• `{number}`"))
        .collect::<Vec<_>>();

    let failed = outcomes
        .iter()
        .filter_map(|(number, reason)| {
            reason
                .as_ref()
                .map(|reason| format!("• `{number}` - {reason}"))
        })
        .collect::<Vec<_>>();

    let mut sections = Vec::new();

    if !added.is_empty() {
        sections.push(format!(
            "✅ *Adicionados ({}):*\n{}",
            added.len(),
            added.join("\n")
        ));
    }

    if !failed.is_empty() {
        sections.push(format!(
            "❌ *Falharam ({}):*\n{}",
            failed.len(),
            failed.join("\n")
        ));
    }

    sections.join("\n\n")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lists_who_was_added_and_who_failed() {
        let outcomes = vec![
            ("5511999990001".to_owned(), None),
            ("5511999990002".to_owned(), Some(describe_error("403"))),
            ("5511999990003".to_owned(), Some(describe_error("999"))),
        ];

        let report = build_report(&outcomes);

        assert!(report.contains("✅ *Adicionados (1):*\n• `5511999990001`"));
        assert!(report.contains("❌ *Falharam (2):*"));
        assert!(report.contains("• `5511999990002` - privacidade do contato (use !invite)"));
        assert!(report.contains("• `5511999990003` - erro 999"));
    }

    #[test]
    fn omits_the_empty_sections() {
        let all_added = build_report(&[("5511999990001".to_owned(), None)]);
        let all_failed = build_report(&[("5511999990001".to_owned(), Some(describe_error("409")))]);

        assert!(!all_added.contains("Falharam"));
        assert!(!all_failed.contains("Adicionados"));
    }

    #[test]
    fn reports_a_missing_server_response() {
        assert_eq!(
            failure_reason(None),
            Some("sem resposta do servidor".to_owned())
        );
    }
}
