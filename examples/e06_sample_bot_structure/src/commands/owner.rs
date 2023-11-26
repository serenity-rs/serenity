use serenity::framework::standard::macros::command;
use serenity::framework::standard::CommandResult;
use serenity::model::prelude::*;

use crate::{Context, Data};

#[command]
#[owners_only]
async fn quit<Data>(ctx: &Context, msg: &Message) -> CommandResult {
    let shard_manager = ctx.data.shard_manager.get().unwrap();
    msg.reply(ctx, "Shutting down!").await?;
    shard_manager.shutdown_all().await;

    Ok(())
}
