use serenity::builder::CreateApplicationCommand;

pub fn register(command: &mut CreateApplicationCommand) -> &mut CreateApplicationCommand {
    command.name("wonderful_command").description("An amazing command")
}
