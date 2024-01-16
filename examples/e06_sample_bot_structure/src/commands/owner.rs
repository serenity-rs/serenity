use serenity::framework::standard::macros::command;
use serenity::framework::standard::CommandResult;
use serenity::model::prelude::*;
use serenity::prelude::*;

use crate::UserData;

#[command]
#[owners_only]
async fn quit(ctx: &Context, msg: &Message) -> CommandResult {
    let data = ctx.data::<UserData>();
    let shard_manager = data.shard_manager.get().expect("should be init before startup");

    msg.reply(ctx, "Shutting down!").await?;
    shard_manager.shutdown_all().await;

    Ok(())
}
