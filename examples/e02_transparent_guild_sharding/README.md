# Transparent guild sharding

This example shows how guild sharding works.

Serenity implements transparent sharding in a way that eliminates the need to handle separate
processes or connections manually. Transparent sharding is useful for a shared cache. Instead of
having caches with duplicated data, a shared cache means all your data can be easily accessible
across all shards.

> **Tip:** If your bot is in many guilds, then you should use guild sharding. When it surpasses
the [single-shard maximum of 2500], sharding becomes a requirement.

For the effects to be clear in this example, your bot should be in at least two guilds.

Assuming a scenario of two guilds: If you say "!ping" in one guild, it should print either
"Shard 0" or "Shard 1" in the console. Saying "!ping" in the other guild should print the
other number in the console. This confirms that guild sharding works.

[single-shard maximum of 2500]: https://docs.discord.com/developers/events/gateway#sharding
