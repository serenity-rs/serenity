use serenity::command;

command!(ping(ctx, msg) {
    let _ = msg.channel_id.say(&ctx.http, "Pong!");
});
