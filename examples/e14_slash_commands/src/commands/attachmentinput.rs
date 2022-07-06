use serenity::builder::CreateApplicationCommand;
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

pub fn register(command: &mut CreateApplicationCommand) -> &mut CreateApplicationCommand {
    command.name("attachmentinput").description("Test command for attachment input").create_option(
        |option| {
            option
                .name("attachment")
                .description("A file")
                .kind(CommandOptionType::Attachment)
                .required(true)
        },
    )
}
