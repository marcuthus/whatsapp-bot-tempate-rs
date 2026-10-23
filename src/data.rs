use std::{sync::Arc, time::Instant};

use whatsapp_rust::prelude::*;

use crate::{helpers::config::Config, utils::vcard};

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
        let message = self.message.message.get_base_message();

        message.text_content().or_else(|| message.get_caption())
    }

    pub fn author_ids(&self) -> Vec<String> {
        let source = &self.message.info.source;

        std::iter::once(&source.sender)
            .chain(source.sender_alt.as_ref())
            .map(|jid| jid.user_base().to_owned())
            .collect()
    }

    pub fn author_jids(&self) -> Vec<Jid> {
        let source = &self.message.info.source;

        std::iter::once(&source.sender)
            .chain(source.sender_alt.as_ref())
            .cloned()
            .collect()
    }

    pub fn quoted_contact_numbers(&self) -> Vec<String> {
        let Some(quoted) = self
            .message
            .message
            .get_base_message()
            .extended_text_message
            .as_option()
            .and_then(|text| text.context_info.as_option())
            .and_then(|context_info| context_info.quoted_message.as_option())
            .map(|quoted| quoted.get_base_message())
        else {
            return Vec::new();
        };

        let single = quoted.contact_message.as_option().into_iter();
        let array = quoted
            .contacts_array_message
            .as_option()
            .into_iter()
            .flat_map(|contacts| contacts.contacts.iter());

        let mut numbers: Vec<String> = Vec::new();

        for card in single
            .chain(array)
            .filter_map(|contact| contact.vcard.as_deref())
        {
            for number in vcard::phone_numbers(card) {
                if !numbers.contains(&number) {
                    numbers.push(number);
                }
            }
        }

        numbers
    }

    pub fn mentioned_jids(&self) -> Vec<Jid> {
        self.message
            .message
            .get_base_message()
            .extended_text_message
            .as_option()
            .and_then(|text| text.context_info.as_option())
            .map(|context_info| {
                context_info
                    .mentioned_jid
                    .iter()
                    .filter_map(|jid| jid.parse().ok())
                    .collect()
            })
            .unwrap_or_default()
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
