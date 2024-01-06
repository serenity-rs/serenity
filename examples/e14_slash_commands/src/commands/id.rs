use serenity::builder::{CreateCommand, CreateCommandOption};
use serenity::model::application::{CommandOptionType, ResolvedOption, ResolvedValue};

pub fn run(options: &[ResolvedOption]) -> String {
    if let Some(ResolvedOption {
        value: ResolvedValue::User(user, _), ..
    }) = options.first()
    {
        format!("{}'s id is {}", user.tag(), user.id)
    } else {
        "Please provide a valid user".to_string()
    }
}

pub fn register() -> CreateCommand<'static> {
    CreateCommand::new("id").description("Get a user id").add_option(
        CreateCommandOption::new(CommandOptionType::User, "id", "The user to lookup")
            .required(true),
    )
}
