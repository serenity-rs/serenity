use serenity::builder::CreateCommand;

pub fn register() -> CreateCommand<'static> {
    CreateCommand::new("wonderful_command").description("An amazing command")
}
