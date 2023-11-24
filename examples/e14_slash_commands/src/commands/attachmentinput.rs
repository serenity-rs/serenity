use serenity::builder::{CreateCommand, CreateCommandOption};
use serenity::model::application::{CommandOptionType, ResolvedOption, ResolvedValue};

pub fn run(options: &[ResolvedOption]) -> String {
    if let Some(ResolvedOption {
        value: ResolvedValue::Attachment(attachment), ..
    }) = options.first()
    {
        format!("Attachment name: {}, attachment size: {}", attachment.filename, attachment.size)
    } else {
        "Please provide a valid attachment".to_string()
    }
}

pub fn register() -> CreateCommand {
    CreateCommand::new("attachmentinput")
        .description("Test command for attachment input")
        .add_option(
            CreateCommandOption::new(CommandOptionType::Attachment, "attachment", "A file")
                .required(true),
        )
}
