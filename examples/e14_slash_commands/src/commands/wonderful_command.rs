use serenity::builder::CreateCommand;

pub fn register() -> CreateCommand {
    CreateCommand::new("wonderful_command").description("An amazing command")
}
