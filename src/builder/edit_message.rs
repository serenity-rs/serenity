use std::collections::HashMap;

use super::CreateEmbed;
#[cfg(feature = "unstable_discord_api")]
use crate::builder::CreateComponents;
use crate::http::AttachmentType;
use crate::internal::prelude::*;
use crate::json::{self, from_number};

/// A builder to specify the fields to edit in an existing message.
///
/// # Examples
///
/// Editing the content of a [`Message`] to `"hello"`:
///
/// ```rust,no_run
/// # use serenity::model::id::{ChannelId, MessageId};
/// # #[cfg(feature = "client")]
/// # use serenity::client::Context;
/// # #[cfg(feature = "framework")]
/// # use serenity::framework::standard::{CommandResult, macros::command};
/// #
/// # #[cfg(all(feature = "model", feature = "utils", feature = "framework"))]
/// # #[command]
/// # async fn example(ctx: &Context) -> CommandResult {
/// # let mut message = ChannelId(7).message(&ctx, MessageId(8)).await?;
/// message.edit(ctx, |m| m.content("hello")).await?;
/// # Ok(())
/// # }
/// ```
///
/// [`Message`]: crate::model::channel::Message
#[derive(Clone, Debug, Default)]
pub struct EditMessage<'a>(pub HashMap<&'static str, Value>, pub Vec<AttachmentType<'a>>);

impl<'a> EditMessage<'a> {
    /// Set the content of the message.
    ///
    /// **Note**: Message contents must be under 2000 unicode code points.
    #[inline]
    pub fn content<D: ToString>(&mut self, content: D) -> &mut Self {
        self.0.insert("content", Value::from(content.to_string()));
        self
    }

    fn _add_embed(&mut self, embed: CreateEmbed) -> &mut Self {
        let map = json::hashmap_to_json_map(embed.0);
        let embed = Value::from(map);

        let embeds = self.0.entry("embeds").or_insert_with(|| Value::from(Vec::<Value>::new()));
        let embeds_array = embeds.as_array_mut().expect("Embeds must be an array");

        embeds_array.push(embed);

        self
    }

    /// Add an embed for the message.
    ///
    /// **Note**: This will keep all existing embeds. Use [`Self::set_embed()`] to replace existing
    /// embeds.
    pub fn add_embed<F>(&mut self, f: F) -> &mut Self
    where
        F: FnOnce(&mut CreateEmbed) -> &mut CreateEmbed,
    {
        let mut embed = CreateEmbed::default();
        f(&mut embed);
        self._add_embed(embed)
    }

    /// Add multiple embeds for the message.
    ///
    /// **Note**: This will keep all existing embeds. Use [`Self::set_embeds()`] to replace existing
    /// embeds.
    pub fn add_embeds(&mut self, embeds: Vec<CreateEmbed>) -> &mut Self {
        for embed in embeds {
            self._add_embed(embed);
        }

        self
    }

    /// Set an embed for the message.
    ///
    /// Equivalent to [`Self::set_embed()`].
    ///
    /// **Note**: This will replace all existing embeds. Use
    /// [`Self::add_embed()`] to add an additional embed.
    pub fn embed<F>(&mut self, f: F) -> &mut Self
    where
        F: FnOnce(&mut CreateEmbed) -> &mut CreateEmbed,
    {
        let mut embed = CreateEmbed::default();
        f(&mut embed);
        self.0.insert("embeds", Value::from(Vec::<Value>::new()));
        self._add_embed(embed)
    }

    /// Set an embed for the message.
    ///
    /// Equivalent to [`Self::embed()`].
    ///
    /// **Note**: This will replace all existing embeds.
    /// Use [`Self::add_embed()`] to add an additional embed.
    pub fn set_embed(&mut self, embed: CreateEmbed) -> &mut Self {
        self.0.insert("embeds", Value::from(Vec::<Value>::new()));
        self._add_embed(embed)
    }

    /// Set multiple embeds for the message.
    ///
    /// **Note**: This will replace all existing embeds. Use [`Self::add_embeds()`] to keep existing
    /// embeds.
    pub fn set_embeds(&mut self, embeds: Vec<CreateEmbed>) -> &mut Self {
        self.0.insert("embeds", Value::from(Vec::<Value>::new()));
        for embed in embeds {
            self._add_embed(embed);
        }

        self
    }

    /// Suppress or unsuppress embeds in the message, this includes those generated by Discord
    /// themselves.
    pub fn suppress_embeds(&mut self, suppress: bool) -> &mut Self {
        // `1 << 2` is defined by the API to be the SUPPRESS_EMBEDS flag.
        // At the time of writing, the only accepted value in "flags" is `SUPPRESS_EMBEDS` for editing messages.
        let flags = if suppress { 1 << 2 } else { 0 };
        self.0.insert("flags", from_number(flags));

        self
    }

    /// Sets the components of this message.
    #[cfg(feature = "unstable_discord_api")]
    pub fn components<F>(&mut self, f: F) -> &mut Self
    where
        F: FnOnce(&mut CreateComponents) -> &mut CreateComponents,
    {
        let mut components = CreateComponents::default();
        f(&mut components);

        self.0.insert("components", Value::from(components.0));
        self
    }

    /// Add a new attachment for the message.
    ///
    /// This can be called multiple times.
    pub fn attachment(&mut self, attachment: impl Into<AttachmentType<'a>>) -> &mut Self {
        self.1.push(attachment.into());
        self
    }
}
