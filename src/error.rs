use thiserror::Error;
use tracing::error;

use crate::data::{Context, Error};

#[derive(Debug, Error)]
pub enum CommandError {
    #[error("permissão negada")]
    PermissionDenied,
    #[error("comando restrito a grupos")]
    NotInGroup,
}

pub async fn handle(ctx: &Context, command_name: &str, error: Error) {
    let reply = match error.downcast_ref::<CommandError>() {
        Some(CommandError::PermissionDenied) => format!(
            "```Permission Error:\n{}, você não possui permissão para usar este comando.```",
            ctx.author_name()
        ),
        Some(CommandError::NotInGroup) => String::from("Este comando só funciona em grupos."),
        None => {
            error!(target: "client", "Erro no comando `{command_name}`: {error:?}");

            format!("Ocorreu um erro ao tentar executar o comando `{command_name}`!")
        }
    };

    if let Err(error) = ctx.reply(reply).await {
        error!(target: "client", "Falha ao responder ao usuário: {error:?}");
    }
}
