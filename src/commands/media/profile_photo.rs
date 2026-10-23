use whatsapp_rust::{
    UploadOptions,
    download::MediaType,
    http::HttpRequest,
    media::{ImageOptions, image_message},
};

use crate::data::{Context, Error};

const OK: u16 = 200;

pub async fn run(ctx: &Context) -> Result<(), Error> {
    let mentioned = ctx.mentioned_jids().into_iter().next();
    let authors = ctx.author_jids();

    let Some(target) = mentioned.clone().or_else(|| {
        authors
            .iter()
            .find(|jid| jid.is_pn())
            .or(authors.first())
            .cloned()
    }) else {
        return Ok(());
    };

    let client = &ctx.message.client;

    let picture = client
        .contacts()
        .get_profile_picture(&target, false)
        .await
        .ok()
        .flatten();

    let Some(picture) = picture else {
        ctx.reply(unavailable_message(mentioned.is_some())).await?;
        return Ok(());
    };

    let response = client
        .http_client
        .execute(HttpRequest::get(picture.url))
        .await?;

    if response.status_code != OK {
        ctx.reply(unavailable_message(mentioned.is_some())).await?;
        return Ok(());
    }

    let upload = client
        .upload(response.body, MediaType::Image, UploadOptions::default())
        .await?;

    let mut context_info = ctx.message.build_quote_context();

    if mentioned.is_some() {
        context_info.mentioned_jid = vec![target.to_string()];
    }

    let message = image_message(
        upload,
        ImageOptions {
            caption: Some(caption(mentioned.is_some(), target.user_base())),
            mimetype: Some("image/jpeg".to_owned()),
            context_info: Some(Box::new(context_info)),
            ..Default::default()
        },
    );

    ctx.message.send_message(message).await?;

    Ok(())
}

fn caption(mentioned: bool, user: &str) -> String {
    if mentioned {
        format!("Foto de perfil de @{user}")
    } else {
        "Sua foto de perfil".to_owned()
    }
}

fn unavailable_message(mentioned: bool) -> &'static str {
    if mentioned {
        "Não foi possível obter a foto de perfil: a pessoa não tem foto ou ela está privada."
    } else {
        "Não foi possível obter a sua foto de perfil: você não tem foto ou ela está privada."
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mentions_the_person_in_the_caption() {
        assert_eq!(
            caption(true, "5511999990001"),
            "Foto de perfil de @5511999990001"
        );
        assert_eq!(caption(false, "5511999990001"), "Sua foto de perfil");
    }

    #[test]
    fn explains_why_the_photo_is_unavailable() {
        assert!(unavailable_message(true).contains("a pessoa"));
        assert!(unavailable_message(false).contains("você"));
    }
}
