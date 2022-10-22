use serenity::builder::CreateCommand;
use serenity::model::prelude::interaction::application_command::ResolvedOption;

pub fn run(_options: &[ResolvedOption]) -> String {
    "Hey, I'm alive!".to_string()
}

pub fn register() -> CreateCommand {
    CreateCommand::new("ping").description("A ping command")
}
