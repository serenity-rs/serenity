#[deprecated(note = "use `model::application::interaction::application_command")]
pub mod application_command {
    use crate::model::application::interaction::application_command;

    /// An interaction when a user invokes a slash command.
    #[deprecated(
        note = "use `model::application::interaction::application_command::ApplicationCommandInteraction`"
    )]
    pub type ApplicationCommandInteraction = application_command::ApplicationCommandInteraction;

    /// The command data payload.
    #[deprecated(note = "use `model::application::interaction::application_command::CommandData`")]
    pub type ApplicationCommandInteractionData = application_command::CommandData;

    /// The resolved value of a [`ApplicationCommandInteractionData::target_id`].
    #[deprecated(
        note = "use `model::application::interaction::application_command::ResolvedTarget`"
    )]
    pub type ResolvedTarget = application_command::ResolvedTarget;

    /// The resolved data of a command data interaction payload.
    /// It contains the objects of [`ApplicationCommandInteractionDataOption`]s.
    #[deprecated(
        note = "use `model::application::interaction::application_command::CommandDataResolved`"
    )]
    pub type ApplicationCommandInteractionDataResolved = application_command::CommandDataResolved;

    /// A set of a parameter and a value from the user.
    ///
    /// All options have names and an option can either be a parameter and input `value` or it can denote a sub-command or group, in which case it will contain a
    /// top-level key and another vector of `options`.
    ///
    /// Their resolved objects can be found on [`ApplicationCommandInteractionData::resolved`].
    #[deprecated(
        note = "use `model::application::interaction::application_command::CommandDataOption`"
    )]
    pub type ApplicationCommandInteractionDataOption = application_command::CommandDataOption;

    /// The resolved value of an [`ApplicationCommandInteractionDataOption`].
    #[deprecated(
        note = "use `model::application::interaction::application_command::CommandDataOptionValue`"
    )]
    pub type ApplicationCommandInteractionDataOptionValue =
        application_command::CommandDataOptionValue;

    use crate::model::application::command;

    /// The base command model that belongs to an application.
    #[deprecated(note = "use `model::application::command::Command`")]
    pub type ApplicationCommand = command::Command;

    /// The type of an application command.
    #[deprecated(note = "use `model::application::command::CommandType`")]
    pub type ApplicationCommandType = command::CommandType;

    /// The parameters for an [`ApplicationCommand`].
    #[deprecated(note = "use `model::application::command::CommandOption`")]
    pub type ApplicationCommandOption = command::CommandOption;

    /// The type of an [`ApplicationCommandOption`].
    #[deprecated(note = "use `model::application::command::CommandOptionType`")]
    pub type ApplicationCommandOptionType = command::CommandOptionType;

    /// The only valid values a user can pick in an [`ApplicationCommandOption`].
    #[deprecated(note = "use `model::application::command::CommandOptionChoice`")]
    pub type ApplicationCommandOptionChoice = command::CommandOptionChoice;

    /// An [`ApplicationCommand`] permission.
    #[deprecated(note = "use `model::application::command::CommandPermission`")]
    pub type ApplicationCommandPermission = command::CommandPermission;

    /// The [`ApplicationCommandPermission`] data.
    #[deprecated(note = "use `model::application::command::CommandPermissionData`")]
    pub type ApplicationCommandPermissionData = command::CommandPermissionData;

    /// The type of an [`ApplicationCommandPermissionData`].
    #[deprecated(note = "use `model::application::command::CommandPermissionType`")]
    pub type ApplicationCommandPermissionType = command::CommandPermissionType;
}

#[deprecated(note = "use `model::application::interaction::autocomplete")]
pub mod autocomplete {
    use crate::model::application::interaction::autocomplete;

    /// An interaction received when the user fills in an autocomplete option
    #[deprecated(
        note = "use `model::application::interaction::autocomplete::AutocompleteInteraction`"
    )]
    pub type AutocompleteInteraction = autocomplete::AutocompleteInteraction;
}

#[deprecated(note = "use `model::application::interaction::message_component")]
pub mod message_component {
    use crate::model::application::component;
    use crate::model::application::interaction::message_component;

    /// An interaction triggered by a message component.
    #[deprecated(
        note = "use `model::application::interaction::message_component::MessageComponentInteraction`"
    )]
    pub type MessageComponentInteraction = message_component::MessageComponentInteraction;

    /// A message component interaction data, provided by [`MessageComponentInteraction::data`]
    #[deprecated(
        note = "use `model::application::interaction::message_component::MessageComponentInteractionData`"
    )]
    pub type MessageComponentInteractionData = message_component::MessageComponentInteractionData;

    /// The type of a component
    #[deprecated(note = "use `model::application::component::ComponentType`")]
    pub type ComponentType = component::ComponentType;

    /// An action row.
    #[deprecated(note = "use `model::application::component::ActionRow`")]
    pub type ActionRow = component::ActionRow;

    // A component which can be inside of an [`ActionRow`].
    #[deprecated(note = "use `model::application::component::ActionRowComponent`")]
    pub type ActionRowComponent = component::ActionRowComponent;

    /// A button component.
    #[deprecated(note = "use `model::application::component::Button`")]
    pub type Button = component::Button;

    /// The style of a button.
    #[deprecated(note = "use `model::application::component::ButtonStyle`")]
    pub type ButtonStyle = component::ButtonStyle;

    /// A select menu component.
    #[deprecated(note = "use `model::application::component::SelectMenu`")]
    pub type SelectMenu = component::SelectMenu;

    /// A select menu component options.
    #[deprecated(note = "use `model::application::component::SelectMenuOption`")]
    pub type SelectMenuOption = component::SelectMenuOption;

    /// An input text component for modal interactions
    #[deprecated(note = "use `model::application::component::InputText`")]
    pub type InputText = component::InputText;

    /// The style of the input text
    #[deprecated(note = "use `model::application::component::InputTextStyle`")]
    pub type InputTextStyle = component::InputTextStyle;
}

#[deprecated(note = "use `model::application::interaction::modal")]
pub mod modal {
    use crate::model::application::interaction::modal;

    /// An interaction triggered by a modal submit.
    #[deprecated(note = "use `model::application::interaction::modal::ModalSubmitInteraction`")]
    pub type ModalSubmitInteraction = modal::ModalSubmitInteraction;

    /// A modal submit interaction data, provided by [`ModalSubmitInteraction::data`]
    #[deprecated(note = "use `model::application::interaction::modal::ModalSubmitInteractionData`")]
    pub type ModalSubmitInteractionData = modal::ModalSubmitInteractionData;
}

#[deprecated(note = "use `model::application::interaction::ping")]
pub mod ping {
    use crate::model::application::interaction::ping;

    /// A ping interaction, which can only be received through an endpoint url.
    #[deprecated(note = "use `model::application::interaction::ping::PingInteraction`")]
    pub type PingInteraction = ping::PingInteraction;
}

use crate::model::application::interaction;

#[deprecated(note = "use `model::application::interaction::Interaction`")]
pub type Interaction = interaction::Interaction;

/// The type of an Interaction.
#[deprecated(note = "use `model::application::interaction::InteractionType`")]
pub type InteractionType = interaction::InteractionType;

/// The flags for an interaction response.
#[deprecated(note = "use `model::application::interaction::MessageFlags`")]
pub type InteractionApplicationCommandCallbackDataFlags = interaction::MessageFlags;

/// Sent when a [`Message`] is a response to an [`Interaction`].
///
/// [`Message`]: crate::model::channel::Message
#[deprecated(note = "use `model::application::interaction::MessageInteraction`")]
pub type MessageInteraction = interaction::MessageInteraction;

/// The available responses types for an interaction response.
#[deprecated(note = "use `model::application::interaction::InteractionResponseType`")]
pub type InteractionResponseType = interaction::InteractionResponseType;
