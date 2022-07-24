use serenity::builder::{CreateApplicationCommand, CreateApplicationCommandOption};
use serenity::model::prelude::command::CommandOptionType;

pub fn register() -> CreateApplicationCommand {
    CreateApplicationCommand::default()
        .name("numberinput")
        .description("Test command for number input")
        .add_option(
            CreateApplicationCommandOption::default()
                .name("int")
                .description("An integer from 5 to 10")
                .kind(CommandOptionType::Integer)
                .min_int_value(5)
                .max_int_value(10)
                .required(true),
        )
        .add_option(
            CreateApplicationCommandOption::default()
                .name("number")
                .description("A float from -3.3 to 234.5")
                .kind(CommandOptionType::Number)
                .min_number_value(-3.3)
                .max_number_value(234.5)
                .required(true),
        )
}
