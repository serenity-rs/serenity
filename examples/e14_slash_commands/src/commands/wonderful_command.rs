use serenity::builder::CreateApplicationCommand;

pub fn register() -> CreateApplicationCommand {
    CreateApplicationCommand::default().name("wonderful_command").description("An amazing command")
}
