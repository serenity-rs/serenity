//! This example shows how you can use `rillrate` to create a web dashboard for your bot!
//!
//! This example is considered advanced and requires the knowledge of other examples.
//! Example 5 is needed for the Gateway latency and Framework usage.
//! Example 7 is needed because tracing is being used.
//! Example 12 is needed because global data and atomic are used.
//! Example 13 is needed for the parallel loops that are running to update data from the dashboard.
#![allow(deprecated)] // We recommend migrating to poise, instead of using the standard command framework.

// be lazy, import all macros globally!
#[macro_use]
extern crate tracing;

use std::collections::HashMap;
use std::env;
use std::error::Error;
use std::sync::atomic::*;
use std::sync::Arc;
use std::time::Instant;

use rillrate::prime::table::{Col, Row};
use rillrate::prime::*;
use serenity::async_trait;
use serenity::framework::standard::macros::{command, group, hook};
use serenity::framework::standard::{CommandResult, Configuration, StandardFramework};
use serenity::gateway::ShardManager;
use serenity::model::prelude::*;
use serenity::prelude::*;
use tokio::time::{sleep, Duration};

// Name used to group dashboards.
// You could have multiple packages for different applications, such as a package for the bot
// dashboards, and another package for a web server running alongside the bot.
const PACKAGE: &str = "Bot Dashboards";
// Dashboards are a part inside of package, they can be used to group different types of dashboards
// that you may want to use, like a dashboard for system status, another dashboard for cache
// status, and another one to configure features or trigger actions on the bot.
const DASHBOARD_STATS: &str = "Statistics";
const DASHBOARD_CONFIG: &str = "Config Dashboard";
// This are collapsible menus inside the dashboard, you can use them to group specific sets of data
// inside the same dashboard.
// If you are using constants for this, make sure they don't end in _GROUP or _COMMAND, because
// serenity's command framework uses these internally.
const GROUP_LATENCY: &str = "1 - Discord Latency";
const GROUP_COMMAND_COUNT: &str = "2 - Command Trigger Count";
const GROUP_CONF: &str = "1 - Switch Command Configuration";
// All of the 3 configurable namescapes are sorted alphabetically.

#[derive(Debug, Clone)]
struct CommandUsageValue {
    index: usize,
    use_count: usize,
}

struct Components {
    data_switch: AtomicBool,
    double_link_value: AtomicU8,
    ws_ping_history: Pulse,
    get_ping_history: Pulse,
    #[cfg(feature = "post-ping")]
    post_ping_history: Pulse,
    command_usage_table: Table,
    command_usage_values: Mutex<HashMap<&'static str, CommandUsageValue>>,
}

struct RillRateComponents;

impl TypeMapKey for RillRateComponents {
    // RillRate element types have internal mutability, so we don't need RwLock nor Mutex!
    // We do still want to Arc the type so it can be cloned out of `ctx.data`.
    // If you wanna bind data between RillRate and the bot that doesn't have Atomics, use fields
    // that use RwLock or Mutex, rather than making the enirety of Components one of them, like
    // it's being done with `command_usage_values` this will make it considerably less likely to
    // deadlock.
    type Value = Arc<Components>;
}

struct ShardManagerContainer;

impl TypeMapKey for ShardManagerContainer {
    type Value = Arc<ShardManager>;
}

#[group]
#[commands(ping, switch)]
struct General;

struct Handler;

#[async_trait]
impl EventHandler for Handler {
    async fn ready(&self, _ctx: Context, ready: Ready) {
        info!("{} is connected!", ready.user.name);
    }

    async fn cache_ready(&self, ctx: Context, _guilds: Vec<GuildId>) {
        info!("Cache is ready!");

        let switch = Switch::new(
            [PACKAGE, DASHBOARD_CONFIG, GROUP_CONF, "Toggle Switch"],
            SwitchOpts::default().label("Switch Me and run the `~switch` command!"),
        );
        let switch_instance = switch.clone();

        let ctx_clone = ctx.clone();

        tokio::spawn(async move {
            // There's currently no way to read the current data stored on RillRate types, so we
            // use our own external method of storage, in this case since a switch is essentially
            // just a boolean, we use an AtomicBool, stored on the same Components structure.
            let elements = {
                let data_read = ctx_clone.data.read().await;
                data_read.get::<RillRateComponents>().unwrap().clone()
            };

            switch.sync_callback(move |envelope| {
                if let Some(action) = envelope.action {
                    debug!("Switch action: {:?}", action);

                    // Here we toggle our internal state for the switch.
                    elements.data_switch.swap(action, Ordering::Relaxed);

                    // If you click the switch, it won't turn on by itself, it will just send an
                    // event about it's new status.
                    // We need to manually set the switch to that status.
                    // If we do it at the end, we can make sure the switch switches it's status
                    // only if the action was successful.
                    switch_instance.apply(action);
                }

                Ok(())
            });
        });

        let default_values = {
            let mut values = vec![];
            for i in u8::MIN..=u8::MAX {
                if i % 32 == 0 {
                    values.push(i.to_string())
                }
            }
            values
        };

        // You are also able to have different actions in different elements interact with the same
        // data.
        // In this example, we have a Selector with preset data, and a Slider for more fine grain
        // control of the value.
        let selector = Selector::new(
            [PACKAGE, DASHBOARD_CONFIG, GROUP_CONF, "Value Selector"],
            SelectorOpts::default()
                .label("Select from a preset of values!")
                .options(default_values),
        );
        let selector_instance = selector.clone();

        let slider = Slider::new(
            [PACKAGE, DASHBOARD_CONFIG, GROUP_CONF, "Value Slider"],
            SliderOpts::default()
                .label("Or slide me for more fine grain control!")
                .min(u8::MIN as f64)
                .max(u8::MAX as f64)
                .step(2),
        );
        let slider_instance = slider.clone();

        let ctx_clone = ctx.clone();

        tokio::spawn(async move {
            let elements = {
                let data_read = ctx_clone.data.read().await;
                data_read.get::<RillRateComponents>().unwrap().clone()
            };

            selector.sync_callback(move |envelope| {
                let mut value: Option<u8> = None;

                if let Some(action) = envelope.action {
                    debug!("Values action (selector): {:?}", action);
                    value = action.map(|val| val.parse().unwrap());
                }

                if let Some(val) = value {
                    elements.double_link_value.swap(val, Ordering::Relaxed);

                    // This is the selector callback, yet we are switching the data from the
                    // slider, this is to make sure both fields share the same look in the
                    // dashboard.
                    slider_instance.apply(val as f64);
                }

                // the sync_callback() closure wants a Result value returned.
                Ok(())
            });
        });

        let ctx_clone = ctx.clone();

        tokio::spawn(async move {
            let elements = {
                let data_read = ctx_clone.data.read().await;
                data_read.get::<RillRateComponents>().unwrap().clone()
            };

            // Because sync_callback() waits for an action to happen to it's element, we cannot
            // have both in the same thread, rather we need to listen to them in parallel, but
            // still have both modify the same value in the end.
            slider.sync_callback(move |envelope| {
                let mut value: Option<u8> = None;

                if let Some(action) = envelope.action {
                    debug!("Values action (slider): {:?}", action);
                    value = Some(action as u8);
                }

                if let Some(val) = value {
                    elements.double_link_value.swap(val, Ordering::Relaxed);

                    selector_instance.apply(Some(val.to_string()));
                }

                Ok(())
            });
        });

        let ctx_clone = ctx.clone();

        tokio::spawn(async move {
            let elements = {
                let data_read = ctx_clone.data.read().await;
                data_read.get::<RillRateComponents>().unwrap().clone()
            };

            loop {
                // Get the REST GET latency by counting how long it takes to do a GET request.
                let get_latency = {
                    let now = Instant::now();
                    // `let _` to suppress any errors. If they are a timeout, that will  be
                    // reflected in the plotted graph.
                    let _ = reqwest::get("https://discordapp.com/api/v6/gateway").await;
                    now.elapsed().as_millis() as f64
                };

                // POST Request is feature gated because discord doesn't like bots doing repeated
                // tasks in short time periods, as they are considered API abuse; this is specially
                // true on bigger bots. If you still wanna see this function though, compile the
                // code adding `--features post-ping` to the command.
                //
                // Get the REST POST latency by posting a message to #testing.
                //
                // If you don't want to spam, use the DM channel of some random bot, or use some
                // other kind of POST request such as reacting to a message, or creating an invite.
                // Be aware that if the http request fails, the latency returned may be incorrect.
                #[cfg(feature = "post-ping")]
                let post_latency = {
                    let now = Instant::now();
                    let _ =
                        ChannelId::new(381926291785383946).say(&ctx_clone, "Latency Test").await;
                    now.elapsed().as_millis() as f64
                };

                // Get the Gateway Heartbeat latency.
                // See example 5 for more information about the ShardManager latency.
                let ws_latency = {
                    let data_read = ctx.data.read().await;
                    let shard_manager = data_read.get::<ShardManagerContainer>().unwrap();

                    let runners = shard_manager.runners.lock().await;

                    let runner = runners.get(&ctx.shard_id).unwrap();

                    if let Some(duration) = runner.latency {
                        duration.as_millis() as f64
                    } else {
                        f64::NAN // effectively 0.0ms, it won't display on the graph.
                    }
                };

                elements.ws_ping_history.push(ws_latency);
                elements.get_ping_history.push(get_latency);
                #[cfg(feature = "post-ping")]
                elements.post_ping_history.push(post_latency);

                // Update every heartbeat, when the ws latency also updates.
                sleep(Duration::from_millis(42500)).await;
            }
        });
    }
}

#[hook]
async fn before_hook(ctx: &Context, _: &Message, cmd_name: &str) -> bool {
    let elements = {
        let data_read = ctx.data.read().await;
        data_read.get::<RillRateComponents>().unwrap().clone()
    };

    let command_count_value = {
        let mut count_write = elements.command_usage_values.lock().await;
        let command_count_value = count_write.get_mut(cmd_name).unwrap();
        command_count_value.use_count += 1;
        command_count_value.clone()
    };

    elements.command_usage_table.set_cell(
        Row(command_count_value.index as u64),
        Col(1),
        command_count_value.use_count,
    );

    info!("Running command {}", cmd_name);

    true
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error + Send + Sync>> {
    env::set_var(
        "RUST_LOG",
        // TODO: If you are going to copy this to your crate, update the crate name in the string
        // with the name of the crate you are using it with.
        // This are the recommended log settings for rillrate, otherwise be prepared to be spammed
        // with a ton of events.
        "info,e15_simple_dashboard=trace,meio=warn,rate_core=warn,rill_engine=warn",
    );

    // Initialize the logger to use environment variables.
    //
    // In this case, a good default is setting the environment variable `RUST_LOG` to `debug`, but
    // for production, use the variable defined above.
    tracing_subscriber::fmt::init();

    // Start a server on `http://0.0.0.0:6361/`
    // Currently the port is not configurable, but it will be soon enough; thankfully it's not a
    // common port, so it will be fine for most users.
    rillrate::install("serenity")?;

    // Because you probably ran this without looking at the source :P
    let _ = webbrowser::open("http://localhost:6361");

    let framework = StandardFramework::new().before(before_hook).group(&GENERAL_GROUP);
    framework.configure(Configuration::new().prefix("~"));

    let token = env::var("DISCORD_TOKEN")?;

    // These 3 Pulse are the graphs used to plot the latency overtime.
    let ws_ping_tracer = Pulse::new(
        [PACKAGE, DASHBOARD_STATS, GROUP_LATENCY, "Websocket Ping Time"],
        Default::default(),
        PulseOpts::default()
        // The seconds of data to retain, this is 30 minutes.
        .retain(1800_u32)

        // Column value range
        .min(0)
        .max(200)

        // Label used along the values on the column.
        .suffix("ms".to_string())
        .divisor(1.0),
    );

    let get_ping_tracer = Pulse::new(
        [PACKAGE, DASHBOARD_STATS, GROUP_LATENCY, "Rest GET Ping Time"],
        Default::default(),
        PulseOpts::default().retain(1800_u32).min(0).max(200).suffix("ms".to_string()).divisor(1.0),
    );

    #[cfg(feature = "post-ping")]
    let post_ping_tracer = Pulse::new(
        [PACKAGE, DASHBOARD_STATS, GROUP_LATENCY, "Rest POST Ping Time"],
        Default::default(),
        PulseOpts::default()
        .retain(1800_u32)
        .min(0)
        // Post latency is on average higher, so we increase the max value on the graph.
        .max(500)
        .suffix("ms".to_string())
        .divisor(1.0),
    );

    let command_usage_table = Table::new(
        [PACKAGE, DASHBOARD_STATS, GROUP_COMMAND_COUNT, "Command Usage"],
        Default::default(),
        TableOpts::default()
            .columns(vec![(0, "Command Name".to_string()), (1, "Number of Uses".to_string())]),
    );

    let mut command_usage_values = HashMap::new();

    // Iterate over the commands of the General group and add them to the table.
    for (idx, i) in GENERAL_GROUP.options.commands.iter().enumerate() {
        command_usage_table.add_row(Row(idx as u64));
        command_usage_table.set_cell(Row(idx as u64), Col(0), i.options.names[0]);
        command_usage_table.set_cell(Row(idx as u64), Col(1), 0);
        command_usage_values.insert(i.options.names[0], CommandUsageValue {
            index: idx,
            use_count: 0,
        });
    }

    let components = Arc::new(Components {
        ws_ping_history: ws_ping_tracer,
        get_ping_history: get_ping_tracer,
        #[cfg(feature = "post-ping")]
        post_ping_history: post_ping_tracer,
        data_switch: AtomicBool::new(false),
        double_link_value: AtomicU8::new(0),
        command_usage_table,
        command_usage_values: Mutex::new(command_usage_values),
    });

    let intents = GatewayIntents::GUILD_MESSAGES
        | GatewayIntents::DIRECT_MESSAGES
        | GatewayIntents::MESSAGE_CONTENT;
    let mut client = Client::builder(&token, intents)
        .event_handler(Handler)
        .framework(framework)
        .type_map_insert::<RillRateComponents>(components)
        .await?;

    {
        let mut data = client.data.write().await;

        data.insert::<ShardManagerContainer>(Arc::clone(&client.shard_manager));
    }

    client.start().await?;

    Ok(())
}

/// You can use this command to read the current value of the Switch, Slider and Selector.
#[command]
async fn switch(ctx: &Context, msg: &Message) -> CommandResult {
    let elements = {
        let data_read = ctx.data.read().await;
        data_read.get::<RillRateComponents>().unwrap().clone()
    };

    msg.reply(
        ctx,
        format!(
            "The switch is {} and the current value is {}",
            if elements.data_switch.load(Ordering::Relaxed) { "ON" } else { "OFF" },
            elements.double_link_value.load(Ordering::Relaxed),
        ),
    )
    .await?;

    Ok(())
}

#[command]
#[aliases("latency", "pong")]
async fn ping(ctx: &Context, msg: &Message) -> CommandResult {
    let latency = {
        let data_read = ctx.data.read().await;
        let shard_manager = data_read.get::<ShardManagerContainer>().unwrap();

        let runners = shard_manager.runners.lock().await;

        let runner = runners.get(&ctx.shard_id).unwrap();

        if let Some(duration) = runner.latency {
            format!("{:.2}ms", duration.as_millis())
        } else {
            "?ms".to_string()
        }
    };

    msg.reply(ctx, format!("The shard latency is {latency}")).await?;

    Ok(())
}
