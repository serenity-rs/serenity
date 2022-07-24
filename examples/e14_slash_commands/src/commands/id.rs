use serenity::builder::{CreateApplicationCommand, CreateApplicationCommandOption};
use serenity::model::prelude::command::CommandOptionType;
use serenity::model::prelude::interaction::application_command::{ResolvedOption, ResolvedValue};

pub fn run(options: &[ResolvedOption]) -> String {
    if let Some(ResolvedOption {
        value: ResolvedValue::User(user, _), ..
    }) = options.get(0)
    {
        format!("{}'s id is {}", user.tag(), user.id)
    } else {
        "Please provide a valid user".to_string()
    }
}

pub fn register() -> CreateApplicationCommand {
    CreateApplicationCommand::default().name("id").description("Get a user id").add_option(
        CreateApplicationCommandOption::default()
            .name("id")
            .description("The user to lookup")
            .kind(CommandOptionType::User)
            .required(true),
    )
}
