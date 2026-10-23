use std::{sync::Arc, time::Instant};

use whatsapp_rust::prelude::*;

use crate::helpers::config::Config;

pub type Error = Box<dyn std::error::Error + Send + Sync>;

pub struct Data {
    pub config: Arc<Config>,
    pub started_at: Instant,
}

pub struct Context {
    pub message: MessageContext,
    pub data: Arc<Data>,
}

impl Context {
    pub fn new(message: MessageContext, data: Arc<Data>) -> Self {
        Self { message, data }
    }

    pub fn text(&self) -> Option<&str> {
        self.message.message.text_content()
    }

    pub fn author_ids(&self) -> Vec<String> {
        let source = &self.message.info.source;

        std::iter::once(&source.sender)
            .chain(source.sender_alt.as_ref())
            .map(|jid| jid.user_base().to_owned())
            .collect()
    }

    pub fn author_name(&self) -> String {
        match self.message.info.push_name.trim() {
            "" => String::from("usuário"),
            name => name.to_owned(),
        }
    }

    pub async fn reply(&self, text: impl Into<String>) -> Result<SendResult, Error> {
        Ok(self.message.reply_quoting(text).await?)
    }

    pub async fn edit(&self, target: &SendResult, text: impl Into<String>) -> Result<(), Error> {
        self.message
            .edit_message(&target.message_id, wa::Message::text(text))
            .await?;

        Ok(())
    }

    pub async fn react(&self, emoji: &str) -> Result<(), Error> {
        self.message.react(emoji).await?;

        Ok(())
    }
}
