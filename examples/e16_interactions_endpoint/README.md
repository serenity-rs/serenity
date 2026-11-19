# Interactions endpoint

This example demonstrates how to use HTTP-based interactions instead of the Gateway.

> **Tip:** By default, bots receive interactions from Discord over the Gateway, a persistent
WebSocket connection. However, you can also configure an [Interactions Endpoint URL] in your bot's
settings to receive interactions as HTTP POST requests sent directly to your server.

In this example, you will perform cryptographic verification, acknowledge pings, and handle
command interactions using an interactions endpoint URL.

[Interactions Endpoint URL]: https://docs.discord.com/developers/interactions/overview#configuring-an-interactions-endpoint-url
