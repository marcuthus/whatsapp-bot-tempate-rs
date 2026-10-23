mod admin;
mod example;
mod group;
mod info;

use std::sync::Arc;

use thiserror::Error;
use tracing::{error, info};
use whatsapp_rust::prelude::*;

use crate::{
    data::{Context, Data, Error},
    error,
};

const CATEGORY_LABELS: [(&str, &str); 4] = [
    ("info", "ℹ️ Informações"),
    ("example", "🧪 Exemplos"),
    ("group", "👥 Grupo"),
    ("admin", "🔐 Administração"),
];

const CATALOG: [(&str, &str, &str); 8] = [
    ("info", "ping", "Envia o ping do bot."),
    (
        "info",
        "id",
        "Mostra o seu ID ou o de quem for mencionado. Uso: {prefix}id [@pessoa]",
    ),
    ("info", "help", "Exibe esta ajuda."),
    ("info", "menu", "Exibe o menu de comandos por categoria."),
    ("group", "groupid", "Mostra o ID do grupo."),
    (
        "group",
        "members",
        "Lista os membros do grupo e revela seus IDs.",
    ),
    (
        "example",
        "sum",
        "Soma dois números. Uso: {prefix}sum <a> <b>",
    ),
    (
        "admin",
        "uptime",
        "Mostra há quanto tempo o bot está online (apenas dono).",
    ),
];

#[derive(Debug, Clone, PartialEq)]
pub enum Command {
    Ping,
    Id,
    Help,
    Menu,
    Members,
    GroupId,
    Sum(f64, f64),
    Uptime,
}

#[derive(Debug, Error, PartialEq)]
pub enum ParseError {
    #[error("o comando `{0}` não existe!")]
    UnknownCommand(String),
    #[error("faltou o argumento `{0}`")]
    MissingArgument(&'static str),
    #[error("o argumento `{name}` é inválido: `{value}`")]
    InvalidArgument { name: &'static str, value: String },
    #[error("argumentos demais")]
    TooManyArguments,
}

impl Command {
    pub fn parse(text: &str, prefix: char) -> Option<Result<Self, ParseError>> {
        let body = text.trim().strip_prefix(prefix)?;
        let mut parts = body.split_whitespace();
        let name = parts.next()?.to_lowercase();

        Some(Self::from_parts(&name, parts))
    }

    fn from_parts<'a>(
        name: &str,
        mut arguments: impl Iterator<Item = &'a str>,
    ) -> Result<Self, ParseError> {
        let command = match name {
            "ping" => Self::Ping,
            "id" => {
                arguments.next();
                Self::Id
            }
            "help" => Self::Help,
            "menu" => Self::Menu,
            "members" => Self::Members,
            "groupid" => Self::GroupId,
            "sum" => Self::Sum(
                number(arguments.next(), "a")?,
                number(arguments.next(), "b")?,
            ),
            "uptime" => Self::Uptime,
            _ => return Err(ParseError::UnknownCommand(name.to_owned())),
        };

        match arguments.next() {
            Some(_) => Err(ParseError::TooManyArguments),
            None => Ok(command),
        }
    }

    pub fn descriptions(prefix: char) -> String {
        let lines = CATALOG
            .iter()
            .map(|(_, name, description)| describe(prefix, name, description));

        format!(
            "Comandos disponíveis:\n\n{}",
            lines.collect::<Vec<_>>().join("\n")
        )
    }

    pub fn menu(prefix: char, author_name: &str) -> String {
        let sections = CATEGORY_LABELS.iter().map(|(category, label)| {
            let lines = CATALOG
                .iter()
                .filter(|(command_category, _, _)| command_category == category)
                .map(|(_, name, description)| describe(prefix, name, description))
                .collect::<Vec<_>>();

            format!("*{label}*\n{}", lines.join("\n"))
        });

        format!(
            "*🤖 WhatsApp Bot*\n\n*📋 Menu*\n\nOlá {author_name}, estes são os meus comandos:\n\n{}\n\n_*👨‍💻 By Marcuth*_",
            sections.collect::<Vec<_>>().join("\n\n")
        )
    }

    fn name(&self) -> &'static str {
        match self {
            Self::Ping => "ping",
            Self::Id => "id",
            Self::Help => "help",
            Self::Menu => "menu",
            Self::Members => "members",
            Self::GroupId => "groupid",
            Self::Sum(..) => "sum",
            Self::Uptime => "uptime",
        }
    }
}

fn describe(prefix: char, name: &str, description: &str) -> String {
    format!(
        "• *{prefix}{name}* - {}",
        description.replace("{prefix}", &prefix.to_string())
    )
}

fn number(value: Option<&str>, name: &'static str) -> Result<f64, ParseError> {
    let value = value.ok_or(ParseError::MissingArgument(name))?;

    value.parse().map_err(|_| ParseError::InvalidArgument {
        name,
        value: value.to_owned(),
    })
}

pub async fn handle_message(message: MessageContext, data: Arc<Data>) {
    if message.info.source.is_from_me {
        return;
    }

    let ctx = Context::new(message, data);

    let Some(text) = ctx.text() else {
        return;
    };

    let Some(parsed) = Command::parse(text, ctx.data.config.prefix) else {
        return;
    };

    let command = match parsed {
        Ok(command) => command,
        Err(ParseError::UnknownCommand(name)) => {
            info!(target: "client", "Unknown command '{name}'.");
            return;
        }
        Err(parse_error) => {
            let reply = format!("*Erro de argumentos:*\n\n{parse_error}");

            if let Err(reply_error) = ctx.reply(reply).await {
                error!(target: "client", "Falha ao responder ao usuário: {reply_error:?}");
            }

            return;
        }
    };

    info!(target: "client", "Running the command '{}'.", command.name());

    if let Err(command_error) = run(&ctx, &command).await {
        error::handle(&ctx, command.name(), command_error).await;
    }
}

async fn run(ctx: &Context, command: &Command) -> Result<(), Error> {
    match command {
        Command::Ping => info::ping::run(ctx).await,
        Command::Id => info::id::run(ctx).await,
        Command::Help => info::help::run(ctx).await,
        Command::Menu => info::menu::run(ctx).await,
        Command::Members => group::members::run(ctx).await,
        Command::GroupId => group::group_id::run(ctx).await,
        Command::Sum(a, b) => example::sum::run(ctx, *a, *b).await,
        Command::Uptime => admin::uptime::run(ctx).await,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parse(text: &str) -> Option<Result<Command, ParseError>> {
        Command::parse(text, '!')
    }

    #[test]
    fn parses_sum_with_two_numbers() {
        assert_eq!(parse("!sum 2 3.5"), Some(Ok(Command::Sum(2.0, 3.5))));
    }

    #[test]
    fn rejects_sum_with_invalid_arguments() {
        assert!(matches!(parse("!sum x 1"), Some(Err(_))));
        assert_eq!(parse("!sum 1"), Some(Err(ParseError::MissingArgument("b"))));
        assert_eq!(parse("!sum 1 2 3"), Some(Err(ParseError::TooManyArguments)));
    }

    #[test]
    fn ignores_messages_without_the_prefix() {
        assert_eq!(parse("ping"), None);
        assert_eq!(parse("/ping"), None);
        assert_eq!(parse("!"), None);
    }

    #[test]
    fn id_accepts_an_optional_mention() {
        assert_eq!(parse("!id"), Some(Ok(Command::Id)));
        assert_eq!(parse("!id @5511999999999"), Some(Ok(Command::Id)));
        assert_eq!(parse("!id @1 @2"), Some(Err(ParseError::TooManyArguments)));
    }

    #[test]
    fn parses_groupid() {
        assert_eq!(parse("!groupid"), Some(Ok(Command::GroupId)));
    }

    #[test]
    fn is_case_insensitive() {
        assert_eq!(parse("!PING"), Some(Ok(Command::Ping)));
    }

    #[test]
    fn reports_unknown_commands() {
        assert_eq!(
            parse("!nope"),
            Some(Err(ParseError::UnknownCommand(String::from("nope"))))
        );
    }

    #[test]
    fn descriptions_list_every_command() {
        let descriptions = Command::descriptions('!');

        assert!(descriptions.contains("*!ping*"));
        assert!(descriptions.contains("*!menu*"));
        assert!(descriptions.contains("Uso: !sum <a> <b>"));
    }

    #[test]
    fn menu_groups_commands_by_category() {
        let menu = Command::menu('!', "Ana");

        assert!(menu.contains("Olá Ana"));
        assert!(menu.find("Informações").unwrap() < menu.find("*!ping*").unwrap());
        assert!(menu.find("*!ping*").unwrap() < menu.find("Exemplos").unwrap());
        assert!(menu.find("Exemplos").unwrap() < menu.find("*!sum*").unwrap());
        assert!(menu.find("Administração").unwrap() < menu.find("*!uptime*").unwrap());
    }

    #[test]
    fn every_catalog_command_has_a_known_category() {
        for (category, name, _) in CATALOG {
            assert!(
                CATEGORY_LABELS.iter().any(|(known, _)| *known == category),
                "{name}"
            );
            assert!(Command::parse(&format!("!{name}"), '!').is_some());
        }
    }
}
