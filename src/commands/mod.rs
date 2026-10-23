mod admin;
mod example;
mod group;
mod info;
mod media;

use std::sync::Arc;

use thiserror::Error;
use tracing::{error, info};
use whatsapp_rust::prelude::*;

use media::sticker::StickerOptions;

use crate::{
    data::{Context, Data, Error},
    error,
};

const CATEGORY_LABELS: [(&str, &str); 5] = [
    ("info", "ℹ️ Informações"),
    ("example", "🧪 Exemplos"),
    ("group", "👥 Grupo"),
    ("media", "🖼️ Mídia"),
    ("admin", "🔐 Administração"),
];

const CATALOG: [(&str, &str, &str); 18] = [
    ("info", "ping", "Envia o ping do bot."),
    (
        "info",
        "id",
        "Mostra o seu ID ou o de quem for mencionado. Uso: {prefix}id [@pessoa] [private-reply: true|false, padrão true]",
    ),
    ("info", "help", "Exibe esta ajuda."),
    ("info", "menu", "Exibe o menu de comandos por categoria."),
    ("group", "group-id", "Mostra o ID do grupo."),
    (
        "group",
        "members",
        "Lista os membros do grupo e revela seus IDs. Uso: {prefix}members [private-reply: true|false, padrão true]",
    ),
    (
        "group",
        "ban-all-members",
        "Bane todos os membros que não são administradores (apenas administradores). Uso: {prefix}ban-all-members confirm",
    ),
    (
        "group",
        "ban-member",
        "Bane um membro do grupo (apenas administradores). Uso: {prefix}ban-member @pessoa",
    ),
    (
        "group",
        "add-member",
        "Adiciona ao grupo um número ou os contatos da mensagem respondida (apenas administradores). Uso: {prefix}add-member <número> ou responda um contato com {prefix}add-member",
    ),
    (
        "group",
        "invite",
        "Mostra o link de convite do grupo, ou o envia no privado (apenas administradores). Uso: {prefix}invite [número]",
    ),
    (
        "group",
        "leave",
        "Faz o bot sair do grupo (apenas administradores).",
    ),
    (
        "group",
        "everyone",
        "Menciona todos os membros do grupo (apenas administradores). Uso: {prefix}everyone [mensagem]",
    ),
    (
        "group",
        "create-group",
        "Cria um grupo, adiciona você como admin e o trava para admins (apenas dono). Uso: {prefix}create-group <nome>",
    ),
    (
        "media",
        "sticker",
        "Cria uma figurinha a partir de uma imagem (legenda ou resposta). Uso: {prefix}sticker [crop: true|false|dented] [center: true|false]",
    ),
    (
        "media",
        "sticker2media",
        "Transforma um sticker em imagem (ou em vídeo, se for animado). Responda um sticker com {prefix}sticker2media.",
    ),
    (
        "media",
        "profile-photo",
        "Envia a sua foto de perfil ou a da pessoa mencionada. Uso: {prefix}profile-photo [@pessoa]",
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
    Id(bool),
    Help,
    Menu,
    Members(bool),
    BanAllMembers(bool),
    GroupId,
    BanMember,
    AddMember(Option<String>),
    Invite(Option<String>),
    Leave,
    Everyone(String),
    CreateGroup(String),
    Sticker(StickerOptions),
    Sticker2Media,
    ProfilePhoto,
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
            "id" => Self::Id(private_reply(arguments.by_ref(), true)?),
            "help" => Self::Help,
            "menu" => Self::Menu,
            "members" => Self::Members(private_reply(arguments.by_ref(), false)?),
            "ban-all-members" => Self::BanAllMembers(confirmation(arguments.next())?),
            "group-id" => Self::GroupId,
            "ban-member" => {
                arguments.next();
                Self::BanMember
            }
            "add-member" => Self::AddMember(
                arguments
                    .next()
                    .map(|value| phone_number(Some(value)))
                    .transpose()?,
            ),
            "invite" => Self::Invite(
                arguments
                    .next()
                    .map(|value| phone_number(Some(value)))
                    .transpose()?,
            ),
            "leave" => Self::Leave,
            "everyone" => Self::Everyone(arguments.by_ref().collect::<Vec<_>>().join(" ")),
            "create-group" => {
                let group_name = arguments.by_ref().collect::<Vec<_>>().join(" ");

                if group_name.is_empty() {
                    return Err(ParseError::MissingArgument("nome"));
                }

                Self::CreateGroup(group_name)
            }
            "sticker" => Self::Sticker(StickerOptions::parse(arguments.next(), arguments.next())?),
            "sticker2media" => Self::Sticker2Media,
            "profile-photo" => {
                arguments.next();
                Self::ProfilePhoto
            }
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
            Self::Id(_) => "id",
            Self::Help => "help",
            Self::Menu => "menu",
            Self::Members(_) => "members",
            Self::BanAllMembers(_) => "ban-all-members",
            Self::GroupId => "group-id",
            Self::BanMember => "ban-member",
            Self::AddMember(_) => "add-member",
            Self::Invite(_) => "invite",
            Self::Leave => "leave",
            Self::Everyone(_) => "everyone",
            Self::CreateGroup(_) => "create-group",
            Self::Sticker(_) => "sticker",
            Self::Sticker2Media => "sticker2media",
            Self::ProfilePhoto => "profile-photo",
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

fn private_reply<'a>(
    arguments: impl Iterator<Item = &'a str>,
    accepts_mention: bool,
) -> Result<bool, ParseError> {
    let mut private = true;
    let mut mentions = 0;

    for token in arguments {
        let lowercase = token.to_lowercase();
        let value = lowercase
            .strip_prefix("private-reply=")
            .unwrap_or(&lowercase);

        if value == "true" || value == "false" {
            private = value == "true";
        } else if lowercase.starts_with("private-reply=") {
            return Err(ParseError::InvalidArgument {
                name: "private-reply",
                value: token.to_owned(),
            });
        } else if accepts_mention && token.starts_with('@') && mentions == 0 {
            mentions += 1;
        } else {
            return Err(ParseError::InvalidArgument {
                name: "argumento",
                value: token.to_owned(),
            });
        }
    }

    Ok(private)
}

fn confirmation(value: Option<&str>) -> Result<bool, ParseError> {
    match value {
        None => Ok(false),
        Some(token) if token.eq_ignore_ascii_case("confirm") => Ok(true),
        Some(token) => Err(ParseError::InvalidArgument {
            name: "confirmação",
            value: token.to_owned(),
        }),
    }
}

fn phone_number(value: Option<&str>) -> Result<String, ParseError> {
    let value = value.ok_or(ParseError::MissingArgument("número"))?;
    let digits = value
        .chars()
        .filter(char::is_ascii_digit)
        .collect::<String>();

    if (8..=15).contains(&digits.len()) {
        Ok(digits)
    } else {
        Err(ParseError::InvalidArgument {
            name: "número",
            value: value.to_owned(),
        })
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
        Command::Id(private_reply) => info::id::run(ctx, *private_reply).await,
        Command::Help => info::help::run(ctx).await,
        Command::Menu => info::menu::run(ctx).await,
        Command::Members(private_reply) => group::members::run(ctx, *private_reply).await,
        Command::BanAllMembers(confirmed) => group::ban_all_members::run(ctx, *confirmed).await,
        Command::GroupId => group::group_id::run(ctx).await,
        Command::BanMember => group::ban_member::run(ctx).await,
        Command::AddMember(number) => group::add_member::run(ctx, number.as_deref()).await,
        Command::Invite(number) => group::invite::run(ctx, number.as_deref()).await,
        Command::Leave => group::leave::run(ctx).await,
        Command::Everyone(announcement) => group::everyone::run(ctx, announcement).await,
        Command::CreateGroup(name) => group::create_group::run(ctx, name).await,
        Command::Sticker(options) => media::sticker::run(ctx, *options).await,
        Command::Sticker2Media => media::sticker2media::run(ctx).await,
        Command::ProfilePhoto => media::profile_photo::run(ctx).await,
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
        assert_eq!(parse("!id"), Some(Ok(Command::Id(true))));
        assert_eq!(parse("!id @5511999999999"), Some(Ok(Command::Id(true))));
        assert!(matches!(parse("!id @1 @2"), Some(Err(_))));
    }

    #[test]
    fn id_and_members_reply_privately_by_default() {
        assert_eq!(parse("!members"), Some(Ok(Command::Members(true))));
        assert_eq!(parse("!members false"), Some(Ok(Command::Members(false))));
        assert_eq!(parse("!members TRUE"), Some(Ok(Command::Members(true))));
        assert_eq!(
            parse("!members private-reply=false"),
            Some(Ok(Command::Members(false)))
        );
        assert_eq!(parse("!id @5511 false"), Some(Ok(Command::Id(false))));
        assert_eq!(
            parse("!id private-reply=false"),
            Some(Ok(Command::Id(false)))
        );
    }

    #[test]
    fn rejects_invalid_private_reply_values() {
        assert!(matches!(
            parse("!members private-reply=maybe"),
            Some(Err(ParseError::InvalidArgument {
                name: "private-reply",
                ..
            }))
        ));
        assert!(matches!(parse("!members @5511"), Some(Err(_))));
        assert!(matches!(parse("!id false false2"), Some(Err(_))));
    }

    #[test]
    fn ban_all_members_needs_the_confirm_argument_to_run() {
        assert_eq!(
            parse("!ban-all-members"),
            Some(Ok(Command::BanAllMembers(false)))
        );
        assert_eq!(
            parse("!ban-all-members CONFIRM"),
            Some(Ok(Command::BanAllMembers(true)))
        );
        assert!(matches!(parse("!ban-all-members yes"), Some(Err(_))));
    }

    #[test]
    fn ban_member_accepts_a_single_mention() {
        assert_eq!(
            parse("!ban-member @5511999999999"),
            Some(Ok(Command::BanMember))
        );
        assert_eq!(
            parse("!ban-member @1 @2"),
            Some(Err(ParseError::TooManyArguments))
        );
    }

    #[test]
    fn parses_group_id() {
        assert_eq!(parse("!group-id"), Some(Ok(Command::GroupId)));
    }

    #[test]
    fn add_member_number_is_optional_but_must_be_valid() {
        assert_eq!(
            parse("!add-member +55-11-99999-9999"),
            Some(Ok(Command::AddMember(Some(String::from("5511999999999")))))
        );
        assert_eq!(parse("!add-member"), Some(Ok(Command::AddMember(None))));
        assert!(matches!(parse("!add-member 123"), Some(Err(_))));
    }
    #[test]
    fn invite_number_is_optional() {
        assert_eq!(parse("!invite"), Some(Ok(Command::Invite(None))));
        assert_eq!(
            parse("!invite 5511999999999"),
            Some(Ok(Command::Invite(Some(String::from("5511999999999")))))
        );
    }

    #[test]
    fn create_group_joins_the_remaining_words() {
        assert_eq!(
            parse("!create-group Meu grupo legal"),
            Some(Ok(Command::CreateGroup(String::from("Meu grupo legal"))))
        );
        assert_eq!(
            parse("!create-group"),
            Some(Err(ParseError::MissingArgument("nome")))
        );
    }

    #[test]
    fn everyone_takes_an_optional_announcement() {
        assert_eq!(
            parse("!everyone"),
            Some(Ok(Command::Everyone(String::new())))
        );
        assert_eq!(
            parse("!everyone Reunião às 20h"),
            Some(Ok(Command::Everyone(String::from("Reunião às 20h"))))
        );
    }

    #[test]
    fn parses_sticker2media_and_profile_photo() {
        assert_eq!(parse("!sticker2media"), Some(Ok(Command::Sticker2Media)));
        assert_eq!(parse("!profile-photo"), Some(Ok(Command::ProfilePhoto)));
        assert_eq!(
            parse("!profile-photo @5511999999999"),
            Some(Ok(Command::ProfilePhoto))
        );
        assert_eq!(
            parse("!profile-photo @1 @2"),
            Some(Err(ParseError::TooManyArguments))
        );
    }

    #[test]
    fn parses_commands_without_arguments() {
        assert_eq!(parse("!leave"), Some(Ok(Command::Leave)));
        assert_eq!(parse("!leave now"), Some(Err(ParseError::TooManyArguments)));
    }

    #[test]
    fn sticker_accepts_optional_crop_and_center() {
        let sticker = |crop, center| {
            Some(Ok(Command::Sticker(
                StickerOptions::parse(crop, center).unwrap(),
            )))
        };

        assert_eq!(parse("!sticker"), sticker(None, None));
        assert_eq!(parse("!sticker dented"), sticker(Some("dented"), None));
        assert_eq!(
            parse("!sticker false false"),
            sticker(Some("false"), Some("false"))
        );
        assert!(matches!(
            parse("!sticker maybe"),
            Some(Err(ParseError::InvalidArgument { name: "crop", .. }))
        ));
        assert_eq!(
            parse("!sticker true true extra"),
            Some(Err(ParseError::TooManyArguments))
        );
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
