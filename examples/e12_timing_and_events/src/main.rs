//! This example will showcase one way on how to extend Serenity with a
//! time-scheduler and an event-trigger-system.
//! We will create a remind-me command that will send a message after a
//! a demanded amount of time. Once the message has been sent, the user can
//! react to it, triggering an event to send another message.
use std::{collections::HashSet, env, hash::{Hash, Hasher},
    sync::Arc,
};
use serenity::{
    prelude::*,
    framework::standard::{
        Args, CommandResult, CommandGroup,
        DispatchError, HelpOptions, help_commands, StandardFramework,
        macros::{command, group, help},
    },
    http::Http,
    model::prelude::*,
};
// We will use this crate as event dispatcher.
use hey_listen::sync::{ParallelDispatcher as Dispatcher,
    ParallelDispatcherRequest as DispatcherRequest};
// And this crate to schedule our tasks.
use white_rabbit::{Utc, Scheduler, DateResult, Duration};

// This enum represents possible events a listener might wait for.
// In this case, we want to dispatch an event when a reaction is added.
// Serenity's event-enum is not suitable for this.
// First it offers too many variants we do not need, but most importantly,
// it lacks the `Default`-trait which makes sense
// as the enum-fields have no clear logical default value. But without it,
// constructing mock-variants becomes difficult.
//
// As a result, we make our own slick event-enum!
#[derive(Clone)]
enum DispatchEvent {
    ReactEvent(MessageId, UserId),
}

// We need to implement equality for our enum.
// One could test variants only. In this case, we want to know who reacted
// on which message.
impl PartialEq for DispatchEvent {
    fn eq(&self, other: &DispatchEvent) -> bool {
        match (self, other) {
            (DispatchEvent::ReactEvent(self_message_id, self_user_id),
            DispatchEvent::ReactEvent(other_message_id, other_user_id)) => {
                self_message_id == other_message_id &&
                self_user_id == other_user_id
            }
        }
    }
}

impl Eq for DispatchEvent {}

// See following Clippy-lint:
// https://rust-lang.github.io/rust-clippy/master/index.html#derive_hash_xor_eq
impl Hash for DispatchEvent {
    fn hash<H: Hasher>(&self, state: &mut H) {
        match self {
            DispatchEvent::ReactEvent(msg_id, user_id) => {
                msg_id.hash(state);
                user_id.hash(state);
            }
        }
    }
}

struct DispatcherKey;
impl TypeMapKey for DispatcherKey {
    type Value = Arc<RwLock<Dispatcher<DispatchEvent>>>;
}

struct SchedulerKey;
impl TypeMapKey for SchedulerKey {
    type Value = Arc<RwLock<Scheduler>>;
}

struct Handler;
impl EventHandler for Handler {
    // We want to dispatch an event whenever a new reaction has been added.
    fn reaction_add(&self, context: Context, reaction: Reaction) {
        let dispatcher = {
            let mut context = context.data.write();
            context.get_mut::<DispatcherKey>().expect("Expected Dispatcher.").clone()
        };

        // We may safely unwrap the user_id as the Reaction comes from an event and not
        // Message::react.
        dispatcher.write().dispatch_event(
            &DispatchEvent::ReactEvent(reaction.message_id, reaction.user_id.unwrap()));
    }
}

#[group("remind me")]
#[prefixes("rm", "reminder")]
#[commands(set_reminder)]
struct RemindMe;

#[help]
fn my_help(
    context: &Context,
    msg: &Message,
    args: Args,
    help_options: &'static HelpOptions,
    groups: &[&'static CommandGroup],
    owners: HashSet<UserId>
) -> CommandResult {
    let _ = help_commands::with_embeds(context, msg, args, &help_options, groups, owners);
    Ok(())
}

fn main() {
    // Configure the client with your Discord bot token in the environment.
    let token = env::var("DISCORD_TOKEN").expect(
        "Expected a token in the environment",
    );
    let mut client = Client::new(&token, Handler)
        .expect("Err creating client");

    {
        let mut data = client.data.write();
        // We create a new scheduler with 4 internal threads. Why 4? It really
        // is just an arbitrary number, you are often better setting this
        // based on your CPU.
        // When a task is due, a thread from the threadpool will be used to
        // avoid blocking the scheduler thread.
        let scheduler = Scheduler::new(4);
        let scheduler = Arc::new(RwLock::new(scheduler));

        let mut dispatcher: Dispatcher<DispatchEvent> = Dispatcher::default();
        // Once receiving an event to dispatch, the amount of threads
        // set via `num_threads` will dispatch in parallel.
        dispatcher.num_threads(4).expect("Could not construct threadpool");

        data.insert::<DispatcherKey>(Arc::new(RwLock::new(dispatcher)));
        data.insert::<SchedulerKey>(scheduler);
    }

    // We will fetch your bot's id.
    let bot_id = match client.cache_and_http.http.get_current_application_info() {
        Ok(info) => {
            info.id
        },
        Err(why) => panic!("Could not access application info: {:?}", why),
    };

    client.with_framework(
        // Configures the client, allowing for options to mutate how the
        // framework functions.
        StandardFramework::new()
        .configure(|c| c
            .with_whitespace(true)
            .on_mention(Some(bot_id))
            .prefix("~")
            .delimiters(vec![", ", ","]))
        .on_dispatch_error(|ctx, msg, error| {
            if let DispatchError::Ratelimited(seconds) = error {
                let _ = msg.channel_id.say(&ctx.http, &format!("Try this again in {} seconds.", seconds));
            }
        })
        .after(|_ctx, _msg, cmd_name, error| {

        if let Err(why) = error {
            println!("Error in {}: {:?}", cmd_name, why);
        }
    })
        .help(&MY_HELP)
        .group(&REMINDME_GROUP)
    );

    if let Err(why) = client.start() {
        println!("Client error: {:?}", why);
    }
}

// Just a helper-function for creating the closure we want to use as listener.
// It saves us from writing the same trigger twice for repeated and non-repeated
// tasks (see remind-me command below).
fn thanks_for_reacting(http: Arc<Http>, channel: ChannelId) ->
    Box<dyn Fn(&DispatchEvent) -> Option<DispatcherRequest> + Send + Sync> {

    Box::new(move |_| {
        if let Err(why) = channel.say(&http, "Thanks for reacting!") {
            println!("Could not send message: {:?}", why);
        }

        Some(DispatcherRequest::StopListening)
    })
}

#[command]
#[aliases("add")]
fn set_reminder(context: &Context, msg: &Message, mut args: Args) -> CommandResult {
    // It might be smart to set a moderately high minimum value for `time`
    // to avoid abuse like tasks that repeat every 100ms, especially since
    // channels have send-message rate limits.
    let time: u64 = args.single()?;
    let repeat: bool = args.single()?;
    let args = args.rest().to_string();

    let scheduler = {
        let mut context = context.data.write();
        context.get_mut::<SchedulerKey>().expect("Expected Scheduler.").clone()
    };

    let dispatcher = {
        let mut context = context.data.write();
        context.get_mut::<DispatcherKey>().expect("Expected Dispatcher.").clone()
    };

    let http = context.http.clone();
    let msg = msg.clone();

    let mut scheduler = scheduler.write();

    // First, we check if the user wants a repeated task or not.
    if repeat {
        // Chrono's duration can also be negative
        // and therefore we cast to `i64`.
        scheduler.add_task_duration(Duration::milliseconds(time as i64), move |_| {
            let bot_msg = match msg.channel_id.say(&http, &args) {
                Ok(msg) => msg,
                // We could not send the message, thus we will try sending it
                // again in five seconds.
                // It might be wise to keep a counter for maximum tries.
                // If the channel got deleted, trying to send a message will
                // always fail.
                Err(why) => {
                    println!("Error sending message: {:?}.", why);

                    return DateResult::Repeat(
                        Utc::now() + Duration::milliseconds(5000))
                },
            };

            let http = http.clone();

            // We add a function to dispatch for a certain event.
            dispatcher.write()
                .add_fn(DispatchEvent::ReactEvent(bot_msg.id, msg.author.id),
                    // The `thanks_for_reacting`-function creates a function
                    // to schedule.
                    thanks_for_reacting(http, bot_msg.channel_id));

            // We return that our date shall happen again, therefore we need
            // to tell when this shall be.
            DateResult::Repeat(Utc::now() + Duration::milliseconds(time as i64))
        });
    } else {
        // Pretty much identical with the `true`-case except for the returned
        // variant.
        scheduler.add_task_duration(Duration::milliseconds(time as i64), move |_| {
            let bot_msg = match msg.channel_id.say(&http, &args) {
                Ok(msg) => msg,
                Err(why) => {
                    println!("Error sending message: {:?}.", why);

                    return DateResult::Repeat(
                        Utc::now() + Duration::milliseconds(5000)
                    )
                },
            };
            let http = http.clone();

            dispatcher.write()
                .add_fn(DispatchEvent::ReactEvent(bot_msg.id, msg.author.id),
                    thanks_for_reacting(http, bot_msg.channel_id));

            // The task is done and that's it, we don't need to repeat it.
            DateResult::Done
        });
    };

    Ok(())
}
