use serenity::builder::CreateApplicationCommand;
use serenity::model::prelude::command::CommandOptionType;
use serenity::model::prelude::interaction::application_command::{
    CommandDataOption,
    CommandDataOptionValue,
};

pub fn run(options: &[CommandDataOption]) -> String {
    let option = options
        .get(0)
        .expect("Expected attachment option")
        .resolved
        .as_ref()
        .expect("Expected attachment object");

    if let CommandDataOptionValue::Attachment(attachment) = option {
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
