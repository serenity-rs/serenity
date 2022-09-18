use serenity::builder::CreateApplicationCommand;
use serenity::model::prelude::command::CommandOptionType;

pub fn register(command: &mut CreateApplicationCommand) -> &mut CreateApplicationCommand {
    command
        .name("welcome")
        .name_localized("de", "begrüßen")
        .description("Welcome a user")
        .description_localized("de", "Einen Nutzer begrüßen")
        .create_option(|option| {
            option
                .name("user")
                .name_localized("de", "nutzer")
                .description("The user to welcome")
                .description_localized("de", "Der zu begrüßende Nutzer")
                .kind(CommandOptionType::User)
                .required(true)
        })
        .create_option(|option| {
            option
                .name("message")
                .name_localized("de", "nachricht")
                .description("The message to send")
                .description_localized("de", "Die versendete Nachricht")
                .kind(CommandOptionType::String)
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
                )
        })
}
