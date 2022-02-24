use std::collections::HashMap;

use super::{CreateAllowedMentions, CreateEmbed};
#[cfg(feature = "unstable_discord_api")]
use crate::builder::CreateComponents;
use crate::internal::prelude::*;
use crate::model::channel::MessageFlags;
use crate::utils;

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
pub struct EditMessage(pub HashMap<&'static str, Value>);

impl EditMessage {
    /// Set the content of the message.
    ///
    /// **Note**: Message contents must be under 2000 unicode code points.
    #[inline]
    pub fn content<D: ToString>(&mut self, content: D) -> &mut Self {
        self.0.insert("content", Value::String(content.to_string()));
        self
    }

    fn _add_embed(&mut self, embed: CreateEmbed) -> &mut Self {
        let map = utils::hashmap_to_json_map(embed.0);
        let embed = Value::Object(map);

        let embeds = self.0.entry("embeds").or_insert_with(|| Value::Array(Vec::new()));
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
        self.0.insert("embeds", Value::Array(Vec::new()));
        self._add_embed(embed)
    }

    /// Set an embed for the message.
    ///
    /// Equivalent to [`Self::embed()`].
    ///
    /// **Note**: This will replace all existing embeds.
    /// Use [`Self::add_embed()`] to add an additional embed.
    pub fn set_embed(&mut self, embed: CreateEmbed) -> &mut Self {
        self.0.insert("embeds", Value::Array(Vec::new()));
        self._add_embed(embed)
    }

    /// Set multiple embeds for the message.
    ///
    /// **Note**: This will replace all existing embeds. Use [`Self::add_embeds()`] to keep existing
    /// embeds.
    pub fn set_embeds(&mut self, embeds: Vec<CreateEmbed>) -> &mut Self {
        self.0.insert("embeds", Value::Array(Vec::new()));
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
        self.0.insert("flags", serde_json::Value::Number(serde_json::Number::from(flags)));

        self
    }

    /// Set the allowed mentions for the message.
    pub fn allowed_mentions<F>(&mut self, f: F) -> &mut Self
    where
        F: FnOnce(&mut CreateAllowedMentions) -> &mut CreateAllowedMentions,
    {
        let mut allowed_mentions = CreateAllowedMentions::default();
        f(&mut allowed_mentions);
        let map = utils::hashmap_to_json_map(allowed_mentions.0);
        let allowed_mentions = Value::Object(map);

        self.0.insert("allowed_mentions", allowed_mentions);
        self
    }

    /// Creates components for this message.
    #[cfg(feature = "unstable_discord_api")]
    pub fn components<F>(&mut self, f: F) -> &mut Self
    where
        F: FnOnce(&mut CreateComponents) -> &mut CreateComponents,
    {
        let mut components = CreateComponents::default();
        f(&mut components);

        self.0.insert("components", Value::Array(components.0));
        self
    }

    /// Sets the components of this message.
    #[cfg(feature = "unstable_discord_api")]
    pub fn set_components(&mut self, components: CreateComponents) -> &mut Self {
        self.0.insert("components", Value::Array(components.0));
        self
    }

    /// Sets the flags for the message.
    pub fn flags(&mut self, flags: MessageFlags) -> &mut Self {
        self.0.insert("flags", Value::Number(serde_json::Number::from(flags.bits)));
        self
    }
}
