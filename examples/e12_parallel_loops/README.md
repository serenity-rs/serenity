# Parallel loops

This example demonstrates how to run tasks in a loop with access to Context.

You will use `tokio::spawn` to spawn new green threads, or tasks, from within the event handler.
These tasks can run in parallel with the rest of the application, yet still have access to Context.

You will also see how to use `GenericChannelId` directly to send a message to a specific channel.
