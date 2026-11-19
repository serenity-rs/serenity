# Sample bot structure (with slash commands)

This example shows the recommended structure for a basic bot, while also demonstrating
how to use Serenity's low-level slash command functions.

> **Tip:** While it is possible to create and use slash commands with Serenity alone, we recommend
using [Poise], a command framework with advanced features like edit tracking, single function slash
and prefix commands, and flexible argument parsing.

Individual commands introduce different features or concepts, e.g.:

| Command | Feature/Concept |
|---|---|
| `attachmentinput` | Accepting an `Attachment` as input |
| `id` | Accepting a `User` as input |
| `modal` | Using the `QuickModal` trait to respond with a modal |
| `numberinput` | Accepting different types of numbers and restricting input |
| `welcome` | Localizing commands |

[poise]: https://github.com/serenity-rs/poise/
