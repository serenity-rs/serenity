use serenity::builder::{CreateCommand, CreateCommandOption};
use serenity::model::application::CommandOptionType;

pub fn register() -> CreateCommand<'static> {
    CreateCommand::new("numberinput")
        .description("Test command for number input")
        .add_option(
            CreateCommandOption::new(CommandOptionType::Integer, "int", "An integer from 5 to 10")
                .min_int_value(5)
                .max_int_value(10)
                .required(true),
        )
        .add_option(
            CreateCommandOption::new(
                CommandOptionType::Number,
                "number",
                "A float from -3.3 to 234.5",
            )
            .min_number_value(-3.3)
            .max_number_value(234.5)
            .required(true),
        )
}
