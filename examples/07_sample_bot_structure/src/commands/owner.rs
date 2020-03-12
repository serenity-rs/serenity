use crate::ShardManagerContainer;
use serenity::prelude::*;
use serenity::model::prelude::*;
use serenity::framework::standard::{
    CommandResult,
    macros::command,
};

#[command]
#[owners_only]
async fn quit(ctx: &mut Context, msg: &Message) -> CommandResult {
    let data = ctx.data.read().await;

    if let Some(manager) = data.get::<ShardManagerContainer>() {
        manager.lock().await.shutdown_all().await;
    } else {
        let _ = msg.reply(&ctx, "There was a problem getting the shard manager").await;

        return Ok(());
    }

    let _ = msg.reply(&ctx, "Shutting down!").await;

    Ok(())
}
