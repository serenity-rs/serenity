use serenity::builder::*;
use serenity::model::prelude::*;
use serenity::prelude::*;

pub async fn run(ctx: &Context, interaction: &CommandInteraction) -> Result<(), serenity::Error> {
    let modal = CreateQuickModal::new("About you")
        .timeout(std::time::Duration::from_secs(600))
        .short_field("First name")
        .short_field("Last name")
        .paragraph_field("Hobbies and interests");
    let response = interaction.quick_modal(ctx, modal).await?.unwrap();

    let inputs = response.inputs;
    let (first_name, last_name, hobbies) = (&inputs[0], &inputs[1], &inputs[2]);

    response
        .interaction
        .create_response(
            ctx,
            CreateInteractionResponse::Message(CreateInteractionResponseMessage::new().content(
                format!("**Name**: {first_name} {last_name}\n\nHobbies and interests: {hobbies}"),
            )),
        )
        .await?;
    Ok(())
}

pub fn register() -> CreateCommand {
    CreateCommand::new("modal").description("Asks some details about you")
}
