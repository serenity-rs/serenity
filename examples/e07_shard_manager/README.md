# Shard manager

This example shows how to interact with the client's `ShardManager`.

`ShardManager` is a struct that can be used to interact with shards. This allows an easy method of
retrieving shards' current status, restarting them, or shutting them down.

In this example, we run two shards; this means that there will be two WebSocket connections to
Discord, and each will receive events for _approximately_ 1/2 of all guilds that the bot is in.

> **Tip:** This is primarily useful for large bots that may need to split load on separate VPSs or
dedicated servers. Additionally, Discord requires that there be at least one shard for every 2500
guilds that a bot is in.

For the purposes of this example, we'll print the current statuses of the two shards to the
terminal every 30 seconds. This includes the ID of the shard, the current connection stage
(e.g., "Connecting" or "Connected"), and the approximate WebSocket latency (time between when a
heartbeat is sent to Discord and when a heartbeat acknowledgement is received).

> **Note:** It may take a minute or more for a latency to be recorded or to update, depending on
how often Discord tells the client to send a heartbeat.
