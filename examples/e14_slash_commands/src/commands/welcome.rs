use serenity::builder::{CreateCommand, CreateCommandOption};
use serenity::model::application::CommandOptionType;

pub fn register() -> CreateCommand {
    CreateCommand::new("welcome")
        .description("Welcome a user")
        .name_localized("de", "begrüßen")
        .description_localized("de", "Einen Nutzer begrüßen")
        .add_option(
            CreateCommandOption::new(CommandOptionType::User, "user", "The user to welcome")
                .name_localized("de", "nutzer")
                .description_localized("de", "Der zu begrüßende Nutzer")
                .required(true),
        )
        .add_option(
            CreateCommandOption::new(CommandOptionType::String, "message", "The message to send")
                .name_localized("de", "nachricht")
                .description_localized("de", "Die versendete Nachricht")
                .required(true)
                .add_string_choice_localized(
                    "Welcome to our cool server! Ask me if you need help",
                    "pizza",
                    [(
                        "de",
                        "Willkommen auf unserem coolen Server! Frag mich, falls du Hilfe brauchst",
                    )],
                )
                .add_string_choice_localized("Hey, do you want a coffee?", "coffee", [(
                    "de",
                    "Hey, willst du einen Kaffee?",
                )])
                .add_string_choice_localized(
                    "Welcome to the club, you're now a good person. Well, I hope.",
                    "club",
                    [(
                        "de",
                        "Willkommen im Club, du bist jetzt ein guter Mensch. Naja, hoffentlich.",
                    )],
                )
                .add_string_choice_localized(
                    "I hope that you brought a controller to play together!",
                    "game",
                    [("de", "Ich hoffe du hast einen Controller zum Spielen mitgebracht!")],
                ),
        )
}
