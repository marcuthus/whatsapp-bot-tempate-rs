mod admin;
mod example;
mod info;

use std::sync::Arc;

use thiserror::Error;
use tracing::{error, info};
use whatsapp_rust::prelude::*;

use crate::{
    data::{Context, Data, Error},
    error,
};

const DESCRIPTIONS: [(&str, &str); 5] = [
    ("ping", "Envia o ping do bot."),
    ("id", "Mostra o seu ID de usuário."),
    ("help", "Exibe esta ajuda."),
    ("sum", "Soma dois números. Uso: {prefix}sum <a> <b>"),
    (
        "uptime",
        "Mostra há quanto tempo o bot está online (apenas dono).",
    ),
];

#[derive(Debug, Clone, PartialEq)]
pub enum Command {
    Ping,
    Id,
    Help,
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
            "id" => Self::Id,
            "help" => Self::Help,
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
        let lines = DESCRIPTIONS.iter().map(|(name, description)| {
            format!(
                "*{prefix}{name}* - {}",
                description.replace("{prefix}", &prefix.to_string())
            )
        });

        format!(
            "Comandos disponíveis:\n\n{}",
            lines.collect::<Vec<_>>().join("\n")
        )
    }

    fn name(&self) -> &'static str {
        match self {
            Self::Ping => "ping",
            Self::Id => "id",
            Self::Help => "help",
            Self::Sum(..) => "sum",
            Self::Uptime => "uptime",
        }
    }
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
        assert!(descriptions.contains("Uso: !sum <a> <b>"));
    }
}
