use serenity::builder::{CreateApplicationCommand, CreateApplicationCommandOption};
use serenity::model::prelude::command::CommandOptionType;
use serenity::model::prelude::interaction::application_command::{ResolvedOption, ResolvedValue};

pub fn run(options: &[ResolvedOption]) -> String {
    if let Some(ResolvedOption {
        value: ResolvedValue::Attachment(attachment), ..
    }) = options.get(0)
    {
        format!("Attachment name: {}, attachment size: {}", attachment.filename, attachment.size)
    } else {
        "Please provide a valid attachment".to_string()
    }
}

pub fn register() -> CreateApplicationCommand {
    CreateApplicationCommand::new("attachmentinput")
        .description("Test command for attachment input")
        .add_option(
            CreateApplicationCommandOption::new(
                CommandOptionType::Attachment,
                "attachment",
                "A file",
            )
            .required(true),
        )
}
