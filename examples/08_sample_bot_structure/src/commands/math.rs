command!(multiply(_ctx, msg, _args, one: f64, two: f64) {
    let product = one * two;

    let _ = msg.channel_id.say(&product.to_string());
});
