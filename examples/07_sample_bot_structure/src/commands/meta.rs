command!(latency(ctx, msg) {
    let latency = ctx.shard.lock()
        .latency()
        .map_or_else(|| "N/A".to_string(), |s| {
            format!("{}.{}s", s.as_secs(), s.subsec_nanos())
        });

    let _ = msg.channel_id.say(latency);
});

command!(ping(_ctx, msg) {
    let _ = msg.channel_id.say("Pong!");
});
