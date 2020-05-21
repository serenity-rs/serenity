//! This is an example of the SimpleFramework, it is intended to be
//! less complex than the StandardFramework but has less configrability
//! Requires the 'framework' feature flag be enabled in your project's
//! `Cargo.toml`.
//!
//! This can be enabled by specifying the feature in the dependency section:
//!
//! ```toml
//! [dependencies.serenity]
//! git = "https://github.com/serenity-rs/serenity.git"
//! features = ["framework", "simple_framework"]
//! ```

use std::env;

use serenity::client::{Client, Context};
use serenity::framework::simple::{SimpleFramework, Args, CommandResult};
use serenity::model::channel::Message;

// A function called before every command function, returned bool
// is if the named command should be run or not
async fn before(_ctx: &Context, _msg: &Message, cmd_name: &str) -> bool {
    println!("Recieved a {} command", cmd_name);
    true
}

// A basic ping command
async fn ping(ctx: &Context, msg: &Message, _args: Args) -> CommandResult {
    msg.channel_id.say(ctx, "pong!").await?;
    Ok(())
}

// This function is called after every command is run
// it is given the Result returned by each command
async fn after(_ctx: &Context, _msg: &Message, cmd_name: &str, res: CommandResult) {
    if let Err(why) = res {
        eprintln!("The command \"{}\" returned the following error: {:?}", cmd_name, why);
    }
}

async fn normal_message(_ctx: &Context, msg: &Message) {
    println!("Message is not a command '{}'", msg.content);
}

#[tokio::main]
async fn main() {
    // Configure the client with your Discord bot token in the environment.
    let token = env::var("DISCORD_TOKEN").expect(
        "Expected a token in the environment",
    );

    let framework = SimpleFramework::new()
        // tells the framework to set all command names to lowercase before comparing
        .case_insensitivity(true)
        // sets the command prefix to use, the default is "!"
        .prefix("~")
        // sets the delimiter to use to determine command arguments
        // default is " "
        .delimiter(",")
        // sets the "before" function
        .before(before)
        // adds the "ping" command to the framework
        .add("ping", ping)
        // sets the "after" function
        .after(after)
        // sets the function to call when a message is not a command
        .normal_message(normal_message)
        // tells the framework to use the default help command
        // which sends a list of all commands
        .with_default_help();

    let mut client = Client::new(&token)
        .framework(framework)
        .await
        .expect("Err creating client");
    
    if let Err(why) = client.start().await {
        println!("Client error: {:?}", why);
    }
}