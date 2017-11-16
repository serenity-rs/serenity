command!(quit(ctx, msg, _args) {
    match ctx.quit() {
        Ok(()) => {
            let _ = msg.reply("Shutting down!");
        },
        Err(why) => {
            let _ = msg.reply(&format!("Failed to shutdown: {:?}", why));
        },
    }
});
