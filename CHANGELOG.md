# Change Log

All notable changes to this project will be documented in this file.
This project mostly adheres to [Semantic Versioning][semver].

## [0.2.0] - 2017-04-16

This is a very large release with a number of rewritten components. The cache
has been rewritten to make use of memory more efficiently, the models directory
has been re-organized, structures are now deserialized via serde and
`serde_derive` - instead of the custom decoding build script we had - with a
number of bugfixes and other various changes and additions.

Thanks to the following for their contributions this release:

- [@acdenisSK]
- [@Flat]
- [@fwrs]
- [@hsiW]
- [@Roughsketch]
- [@sschroe]

### Upgrade Path

Replace uses of `ext::cache::ChannelRef` with `model::Channel`.

The following `ext::cache::Cache` method signatures are now encased in
`Arc<RwLock>`s and should be handled appropriately:

- `call`
- `channel`
- `guild`
- `guild_channel`
- `group`
- `member`
- `role`
- `user`

Additionally, `GuildId::find` and `UserId::find` now return
`Option<Arc<RwLock>>`s.

`Member::display_name` now returns a `Cow<String>` instead of a `&str`.

`client::Context` has had most of its methods removed. The methods were mostly
a copy of those on `ChannelId`. Upgrade by instead calling methods on
`ChannelId`:

```rust
command!(foo(ctx) {
    let _ = ctx.say("hello");
});

// is now written as:

command!(bar(_ctx, msg) {
    let _ = msg.channel_id.say("hello");
});
```

`CreateMessage::nonce` has been removed. Instead, simply do not provide a nonce.

`ChannelId::edit_message` now has an argument signature of:

```rust
&self, message_id: M, f: F
where F: FnOnce(CreateMessage) -> CreateMessage, M: Into<MessageId>
```

instead of

```rust
&self, message_id: M, text: &str, f: F
where F: FnOnce(CreateEmbed) -> CreateEmbed, M: Into<MessageId>
```

To account for this change, modify code like so:

```rust
channel_id.edit_message(message_id, "new content", |e| e);

// now:

channel_id.edit_message(message_id, |m| m.content("new content"));
```

`Message::edit` has also had an argument signature updated to:

```rust
&mut self, f: F where F: FnOnce(CreateMessage) -> CreateMessage
```

from:

```rust
&mut self, new_content: &str, embed: F where F: FnOnce(CreateEmbed) -> CreateEmbed
```

To account for this change, modify code like so:

```rust
message.edit("new content", |e| e.description("test"));

// now:

message.edit(|m| m.content("new content").embed(|e| e.description("test")));
```

`client::rest::accept_invite`, `Invite::accept`, and `RichInvite::accept` have
been removed. Instead, do not attempt this, as they were userbot functions.

Selfbot support has been completely removed. Review the
[commit message][c:d9118c0] for the long list of details.

Group calls and guild sync have [also been removed][c:c74cc15]. Read the commit
message for all the details.

Instead of defining multiple separate error messages for command framework
message dispatches, match the dispatch error in a single method:

```rust
// old code:
client.with_framework(|f| f
    .configure(|c| c
        .command_disabled_message("The command `%command%` was disabled")
        .blocked_guild_message("The owner of this guild has been blocked")
        .invalid_permission_message("You don't have permission to use this command")));

// new code:
client.with_framework(|f| f.on_dispatch_error(|_, msg, err| {
    match err {
        DispatchError::CommandDisabled(command_name) => {
            let _ = msg.channel_id.say(&format!("The command `{}` was disabled", command_name));
        },
        DispatchError::BlockedGuild => {
            // this change also allows for more intelligent error messages:
            if let Some(guild) = msg.guild() {
                let owner_id = guild.read().unwrap().owner_id;

                if let Some(user) = CACHE.read().unwrap().user(owner_id) {
                    let c = format!("The owner - {} - has been blocked", user.name);
                    let _ = msg.channel_id.say(&c);

                    return;
                }
            }

            let _ = msg.channel_id.say("The owner of this guild has been blocked");
        },
        DispatchError::LackOfPermissions(_) => {
            let _ = msg.channel_id.say("You don't have permission to use this command");
        },
    }
}));
```

All functions prefixed with `get_` have had the prefix removed. For example,
`Guild::get_webhooks()` is now `Guild::webhooks()`.

Instead of using `model::permissions::general()`, `model::permissions::text()`,
and `model::permissions::voice()`, use
`model::permissions::{PRESET_GENERAL, PRESET_TEXT, PRESET_VOICE}`.

### Added

- Add `say` method to `Group`, `GuildChannel`, `PrivateChannel` [c:a0bb306]
- Add missing `send_file`/`send_message` impls [c:bad9ac3]
- Add `Message::guild` [c:9ef5522]
- Add Shard Id helpers [c:1561f9e]
- Implement `From<&str> for ReactionType` [c:e7110ad]
- Check for embed lengths on message sends [c:e1079e9]
- Add `is_nsfw` check to channels [c:9268f9c]
- Add missing `Member::kick` helper [c:83b1d96]
- Derive `Eq`, `Hash`, `PartialEq` on `ReactionType` [c:86a4e00] ([@acdenisSK])

### Fixed

- Handle unsuccessful responses before decoding [c:7e254c5]
- Uniquely ratelimit message deletions [c:01f6872]
- Fix Member methods due to variant `joined_at` values [c:cd914f5]
- Fix deadlock on channel create event for DMs [c:6b0b9b2] ([@sschroe])
- Default to using `[0, 1]` shards [c:f0d1157]
- Fix ratelimiting for `Route::None` routes [c:5bf6c2d]
- Fix guild leaving result [c:ae352ea]
- Fix permissions when sending to DMs or groups [c:404a089] ([@acdenisSK])
- Check if message starts with `dynamic_prefix` result [c:9ec05e7] ([@Roughsketch])
- Trim content before parsing framework args [c:e6712c9] ([@Roughsketch])

### Changed

- Optimize caching [c:0c9ec37]
- Remove most `Context` methods [c:585af23]
- Remove sending message nonces [c:9c04a19]
- Standardize message editing methods [c:3c7c575]
- Remove invite accepting [c:e4b484f]
- Remove selfbot support [c:d9118c0] [c:c74cc15]
- Switch to using serde for deserialization [c:f6b27eb]
- Update the ways errors are handled in dispatch [c:31aae7d] ([@fwrs])
- Deprecate methods prefixed with `get_` [c:3f03f9a]
- Framework help commands now accept a slice of args [c:ff4437a]
- Make `User.discriminator` a `u16` [c:0f41ffc]
- Use constants for preset permissions [c:70d4e75]

### Misc.

- Make logo more better [c:6e11a10] ([@Flat])
- Fix incorrect cache example [c:b96f85c]
- Rework the models directory [c:9114963]
- Change permission values to byte literals [c:c8536c1]
- Fix example in README [c:d4fc8b6]

## [0.1.5] - 2017-02-08

This is a release to fix broken nightly builds, due to a change in how rustc
handles lifetimes, with a few performance optimizations and other fixes.

### Upgrade Path

For `Group::send_message`, `PrivateChannel::send_message`,
and `GuildChannel::send_message`, instead of passing in only a `&str` of
content, use a `CreateMessage` builder:

```rust
// assuming a `channel` is bound

// old signature:
channel.send_message("hello");

// new signature:
channel.send_message(|m| m.content("hello"));
```

Instead of calling `message_id.get_reaction_users` and passing in a `ChannelId`,
call `channel_id.get_reaction_users` and pass in the `MessageId`. Note that the
latter already existed.

```rust
// assuming `channel_id`, `message_id`, and `reaction_type` are bound

// removed method:
message_id.get_reaction_users(channel_id, reaction_type, Some(10), None);

// alternative method:
channel_id.get_reaction_users(message_id, reaction_type, Some(10), None);
```

### Added

- Register the `status` user setting for user accounts (e.g. online, invisible)
  [c:0b9bf91]
- Expose and document ratelimiting structures [c:eb09f2d]
- Add method to `EditGuild` to transfer ownership [c:f00e165]

### Fixed

- Fix potential unreachable pattern warning in `command!` macro [c:97f9bd1]
- Fix value of 'browser' in shard identify [c:4cf8338]
- Remove lifetime on Search builder [c:6f33a35]

### Changed

- Standardize methods for creating messages [c:c8c6b83]
- Remove `MessageId::get_reaction_users` [c:268f356]

### Misc.

- Avoid re-requesting the gateway URL when autosharding (optimization)
  [c:e891ebe]
- Avoid cloning on non-framework message create events (opt.) [c:b7cbf75]
- Avoid cloning the context on event dispatches (opt.) [c:5ee5fef]
- Optimize presence update for current user in cache (opt.) [c:9392f61]
- Make `GLOBAL` ratelimit mutex a unit (opt.) [c:55ccaca]
- Resume when restarting WS sender/receiver [c:04cfaa9]


## [0.1.4] - 2017-01-26

This is a general release for pretty much everything, from new features to
bugfixes to a switch to a more OOP style. The current minimum supported version
is rustc 1.13+.

The next release will be v0.2.0, which will feature serde codegen support along
with a rewrite of the framework. It will be a more modularized version of the
library. v0.2.0 will require rustc 1.15+, due to the stabilization of Macros
1.1.

Thanks to the following for contributions this release:

- [@acdenisSK]
- [@bippum]
- [@DeltaEvo]
- [@emoticon]
- [@foxbot]
- [@fwrs]
- [@hsiW]
- [@indiv0]
- [@SunDwarf]

Two of the major highlights of this release are that the broken pipe issue has
been fixed, and the library is more OOP now and therefore no longer relies on
the `Context` to get stuff done. The `methods` feature flag has been removed.

### Upgrade Path

When formatting using `Display` for `ChannelId`s, `RoleId`s, and `UserId`,
instead of formatting use their `Mentionable` equivilants:

```rust
use serenity::model::{ChannelId, RoleId, UserId};

// old
assert_eq!(format!("{}", ChannelId(1)), "<#1>");
assert_eq!(format!("{}", RoleId(2)), "<@&2>");
assert_eq!(format!("{}", UserId(3)), "<@3>");

// new
assert_eq!(format!("{}", ChannelId(1).mention()), "<#1>");
assert_eq!(format!("{}", RoleId(2)).mention()), "<@&2>");
assert_eq!(format!("{}", UserId(3).mention()), "<@3>");
```

When using `EmbedBuilder::{image, thumbnail}`, instead of calling another
builder, provide `url`s directly:

```rust
use serenity::model::Embed;

// old
Embed::fake(|e| e
    .image(|i| i.url("https://not.zey.moe/me.png"))
    .thumbnail(|t| t.url("https://not.zey.moe/me2.png")));

// new
Embed::fake(|e| e
    .image("https://not.zey.moe/me.png")
    .thumbnail("https://not.zey.moe/me2.png"));
```

When specifying a sharding method, instead of passing a `u8` for sharding info,
pass a `u64`:

```rust
use serenity::Client;

let client = Client::login_bot(&env::var("DISCORD_TOKEN").unwrap());

// old
client.start_shard(1u8, 5u8); // or
client.start_shards(5u8); // or
client.start_shard_range([1u8, 3u8], 8u8);

// new
client.start_shard(1u64, 5u64); // or
client.start_shards(5u64); // or
client.start_shard_range([1u64, 3u64], 8u64);
```

`Client.shards` is now private. Instead of accessing it, don't.

When creating a `Colour` struct yourself, instead of specifying a single `value`
field, pass a single tuple value:

```rust
use serenity::utils::Colour;

// old
Colour {
    value: 0,
}

// new
Colour(0);
```

Instead of using `Attachment::download_to_directory` to download an attachment
to a directory, do it yourself:

```rust
use std::fs::File;
use std::io::Write;

// assuming an `attachment` has already been bound

// old
attachment.download_to_directory("./attachments");

// new
let bytes = attachment.download().unwrap();
let filepath: PathBuf = path.as_ref().join(&attachment.filename);
let mut f = File::create(&filepath);
let _ = f.write(&bytes);
```

Instead of calling `Message::is_webhook()`:

```rust
// assuming a `message` has already been bound

// old
let _ = message.is_webhook();

// new
let _ = message.webhook_id.is_some();
```

Instead of `PartialGuild::find_role(role_id)`:

```rust
use serenity::model::RoleId;

// assuming a `guild` has already been bound

// old
let _ = guild.find_role(RoleId(1));

// new
let _ = guild.roles.get(RoleId(1));
```

Instead of `Guild::{get_channel, get_member}`, call:

```rust
use serenity::model::{ChannelId, UserId};

// assuming a `guild` has already been bound

// old
let _ = guild.get_channel(ChannelId(1));
let _ = guild.get_member(UserId(2));

// new
let _ = guild.channels.get(ChannelId(1));
let _ = guild.members.get(UserId(2));
```

Instead of using `Context` methods, use their `Id` or other struct equivalents.

### Added

- the `voice` feature no longer requires the `cache` feature to be enabled
  [c:7b45f16]
- the `framework` feature no longer requires the `cache` feature to be enabled
  [c:86cd00f]
- `Guild`, `InviteGuild`, and `PartialGuild` now have `splash_url` methods
  [c:d58c544]
- Expose `Message::webhook_id` for messages sent via webhooks ([@fwrs])
  [c:a2cbeb6]
- Framework: add option to ignore webhooks or DMs ([@fwrs]) [c:8e2c052]
- Added documentation for creating embed timestamps ([@foxbot]) [c:66546d3]
- Allow `time::Tm` to be passed into the embed timestamp field, in addition to
  a direct string [c:b001234]
- Add `Client::on_message()` example ([@indiv0]) [c:bcb70e8]
- Support webp/gif avatars/icons in URL methods [c:ab778f8]
- Update current user presence in cache on set [c:5b275fc]
- Add `CurrentUser`/`User::static_avatar_url()` methods to generate webp URLs
  [c:c36841d]
- Command (batch) alias support ([@fwrs]) [c:f96b6cc]
- Command example field for help command ([@fwrs]) [c:f96b6cc]
- Added "Meibi Pink" to the `Colour` struct ([@hsiW]) [c:2cb607d]
- Register support for `4011` code (too many shards) ([@SunDwarf]) [c:93f3c60]
- Added "Rohrkatze Blue" to the `Colour` struct ([@bippum]) [c:345e140]
- Add `User::default_avatar_url()` [c:e85e901]
- Add `Message::content_safe()` to avoid `@everyone`/`@here`s ([@fwrs])
  [c:e5a83dd]
- Add `Member::distinct()`, `User::distinct()` ([@fwrs]) [c:e5a83dd]
- Document that messages can't be older than 14 days when bulk deleting
  ([@fwrs]) [c:0a2f5ab]
- Add shard latency tracking (~~stolen~~ borrowed from brayzure/Eris)
  [c:096b0f5]
- Add guild chunking [c:3ca7ad9]

### Fixed

- `User::avatar_url` no longer mentions the user in the generated URL
  [c:0708ccf]
- Framework: `owners_only` check now functions only if the author of a message
  is an owner ([@fwrs]) [c:6355288]
- Framework: fix command cooldown timer (would always say to wait `i64::MAX`
  seconds) [c:fafa363]
- Framework: the `before` closure is now properly run when a message is sent by
  the owner [c:760a47a]
- `CurrentApplicationInfo` now properly decodes due to `flags` no longer being
  sent [c:2a743ce]
- Fix `Message::delete()` permission check [c:4229034]
- Framework: properly split messages on character boundary limits; aka thanks
  Unicode [c:c01f238]
- Remove need to import Context/Message in command macros ([@acdenisSK])
  [c:abd22d2]
- Fix a ton of gateway stuff [c:94fc85b], [c:f894cfd], [c:f894cfd]
- Specify `command!` macro signature as returning `std::result::Result`
  [c:e9aae9c]
- Fix dependency description in example 06 ([@DeltaEvo]) [c:92309b2]
- Return a `User` from `rest::get_user` -- not a `CurrentUser` [c:f57a187]
- Fix shards always booting at index 0 [c:83b29d5]
- Wait 5 seconds between shard boots to avoid session invalidations [c:fb4d411]
- Use CDN for default avatars [c:69ec62a]
- Fix `Resumed` event payload decoding [c:c2e8b69]
- Fix `CurrentApplicationInfo` decoding without `rpc_origins` [c:38db32e]
- Reboot shard on broken pipe; fixes a lot of gateway problems [c:76f9095]
- Make `rest::execute_webhook` be a POST [c:c050c59]

### Changed

- Framework: argument number is now displayed on parsing error ([@fwrs])
  [c:fb07751]
- Id display formatters use the direct u64 instead of mentioning;
  `format!("{}", UserId(7))` will format into `"7"` instead of `"<@7>"`
  [c:933ee89]
- Default the framework's `use_quotes` for quote parsing to `false` (was `true`)
  [c:38a484d]
- The `CreateEmbed` builder now has direct `image` and `thumbnail` methods
  instead of one-method builders [c:68c473d]
- Accept `u64` shard counts to allow using more than 255 shards (instead of
  `u8`s) [c:ada07fa]
- Remove user logout endpoint [c:70bf22a]
- Don't abuse unicode for message content sanitization ([@fwrs]) [c:2b237e7]
- Change `Colour` struct to be a tuplestruct [c:a8acd61]
- Make a single POST on guild role create [c:147cf01]
- Switch to a mostly-fully OOP approach [c:651c618]
- Rename `webhooks` methods to `get_webhooks`
  (eg: `GuildChannel::webhooks()` --> `GuildChannel::get_webhooks()`)
  [c:e8a9086]
- Make `Guild::create_channel` and related functions return a `GuildChannel`
  [c:5918d01]

### Misc.

- Cleaned up YAML definition layouts [c:00fb61b]
- Gateway identify compression code is now simplified [c:2416813]
- Gateway Event decoders are now abstracted to individual struct implementations
  [c:5fe6a39]
- Simplify `Role`'s' `Ord` impl ([@emoticon]) [c:6a887b2]
- Slightly simplify `Shard::set_presence` [c:5c40e85]
- Rename repo references from `serenity.rs` to `serenity` ([@fwrs]) [c:3348178]
- Simplify `Reaction::delete()` [c:1594961]
- Abstract large threshold number to a constant [c:f3f74ce]
- Avoid a needless string clone on login [c:d3389be]
- Avoid a lot of `Arc`/`Message`/`RwLock` clones [c:8c5ee70]


## [0.1.3] - 2016-12-14

This is a hotfix for applying a PR and fixing a major bug in the plain help
command.

Thanks to the following for contributions this release:

- [@fwrs]

### Upgrade Path

None.

### Added

- Blocking individual users and guilds in commands, add blocking commands, and
  configuring owners of bots ([@fwrs]) [c:a39647d]

### Fixed

- The plain help command now properly sends a message when requesting
  information about a command [c:7b4b154]

### Misc.

- Groups are now on their own lines in the plain help command [c:16bd765]

## [0.1.2] - 2016-12-14

This release focuses on revamping the framework, adding a large amount of
configuration and overall features. v0.1.3 will be focused on performance
optimizations and code cleanup.

Thanks to the following for contributions this release:

- [@acdenisSK]
- [@fwrs]

v0.1.2 can now be retrieved from the [crates.io listing].

### Upgrade Path

When using `EmbedBuilder::{image, thumbnail}`, instead of calling another
builder, provide `url`s directly:

```rust
use serenity::model::Embed;

// old
Embed::fake(|e| e
    .image(|i| i.url("https://not.zey.moe/me.png"))
    .thumbnail(|t| t.url("https://not.zey.moe/me2.png")));

// new
Embed::fake(|e| e
    .image("https://not.zey.moe/me.png")
    .thumbnail("https://not.zey.moe/me2.png"));
```

### Added

- Allow mentionable structs to be used as command arguments ([@fwrs])
  [c:626ffb2]
- Implemented `From<Embed> for CreateEmbed` [c:7914274]
- Framework command builder, quoted arguments, multi-prefixes ([@fwrs])
  [c:8f24aa3]
- `{Emoji,EmojiIdentifier}::url` [c:ef6eba3]
- Command groups and buckets [c:daf92ed]

### Fixed

- Fix mentioning in the `MessageBuilder` ([@fwrs]) [c:13de5c2]
- Don't mutate token for bots on profile change [c:8effc91]

### Changed

- Deprecate `CreateEmbedImage::{height, width}` and
  `CreateEmbedThumbnail::{height, width}`

### Misc.

- All internal `try!`s have been converted to use `?` syntax ([@acdenisSK])
  [c:f69512b]

## [0.1.1] - 2016-12-05

v0.1.1 is a "features that v0.1.0 should have had" and "miscellaneous work"
release. v0.1.2 will be focused on the framework, while v0.1.3 will be focused
on performance optimizations.

Thanks to the following for contributions this release:

- [@abalabahaha]
- [@Flat]
- [@fwrs]
- [@GetRektByMe]
- [@iCrawl]
- [@indiv0]
- [@khazhyk]
- [@SunDwarf]

v0.1.1 can now be retrieved from the [crates.io listing].

[v0.1.1:example 06]: https://github.com/zeyla/serenity/tree/ccb9d16e5dbe965e5a604e1cb402cd3bc7de0df5/examples/06_command_framework

### Upgrade Path

When calling `rest::get_guilds`, instead of passing no parameters, pass a
`GuildPagination` variant and a `limit`:

```rust
use serenity::client::rest::{self, GuildPagination};
use serenity::model::GuildId;

// before
rest::get_guilds();

// after
rest::get_guilds(GuildPagination::After(GuildId(777)), 50);
```

### Added

- The following colours to the Colour struct:
  - "Kerbal" ([@indiv0]) [c:c032fbe]
  - "Blurple" ([@GetRektByMe]) [c:e9282d3]
  - "Blitz Blue" ([@iCrawl]) [c:f53124e]
  - "Fabled Pink" ([@Flat]) [c:9aa357f]
  - "Fooyoo" ([@SunDwarf]) [c:49a6841]
  - "Rosewater" ([@fwrs]) [c:2eaa415]
- `Message::guild_id` as a quick method for retrieving the Id of a message's
  guild [c:bceb049]
- `CurrentUser::guilds()` to get the current user's guilds. Meant for use with
  selfbots [c:57c060f]
- `CurrentUser::edit()` to edit the current user's profile settings [c:16d1b3c]
- `User::distinct` to format a string with the `username#discriminator`
  combination ([@fwrs]) [c:31becb1]
- `Member::colour` to retrieve the member's colour ([@fwrs]) [c:43a5c5d]
- Roles can now be directly compared (`role1 < role2`) for hierarchy [c:143337a]
- Documentation:
  - `EditMember` and `EditProfile` ([@Kiseii]) [c:e2557ac]
  - Documentation for 19 model definitions ([@fwrs]) [c:2844ae1]
  - Context + permission requirements [c:d144136]
- A custom shared state (not the Cache) can now be accessed and mutated across
  commands/contexts, through the use of `Context.data`'s ShareMap. See
  [example 06][v0.1.1:example 06] for an example

### Fixed

- `rest::start_integration_sync`/`Context::start_integration_sync` now properly
  work ([@abalabahaha]) [c:7f04179]
- Role positions can now be negative; fixes issues where a guild's @everyone
  role (and other roles) are negative [c:f847638]
- `Context::move_member`'s signature is now correct [c:4de39da]
- The `command!` macro now publicly exports functions. This allows commands
  created via this macro to be separated into different modules or crates
  [c:62ed564]

### Changed

- `rest::get_guilds` now supports pagination of guilds, as the output is now
  limited to 100 [c:57c060f]

### Misc.

- `Colour::dark_green` is now sorted alphabetically ([@khazhyk]) [c:4a14b92]
- Simplify the colour macro [c:bb97211]
- Capitalize the hex value for `Colour::blitz_blue` ([@Kiseii]) [c:daa24ec]

## [0.1.0] - 2016-11-30

Initial commit.

[0.2.0]: https://github.com/zeyla/serenity/compare/v0.1.5...v0.2.0
[0.1.5]: https://github.com/zeyla/serenity/compare/v0.1.4...v0.1.5
[0.1.4]: https://github.com/zeyla/serenity/compare/v0.1.3...v0.1.4
[0.1.3]: https://github.com/zeyla/serenity/compare/v0.1.2...v0.1.3
[0.1.2]: https://github.com/zeyla/serenity/compare/v0.1.1...v0.1.2
[0.1.1]: https://github.com/zeyla/serenity/compare/v0.1.0...v0.1.1
[0.1.0]: https://github.com/zeyla/serenity/tree/403d65d5e98bdfa9f0c018610000c4a0b0c7d8d5
[crates.io listing]: https://crates.io/crates/serenity
[semver]: http://semver.org

[@abalabahaha]: https://github.com/abalabahaha
[@acdenisSK]: https://github.com/acdenisSK
[@bippum]: https://github.com/bippum
[@DeltaEvo]: https://github.com/DeltaEvo
[@emoticon]: https://github.com/emoticon
[@Flat]: https://github.com/Flat
[@foxbot]: https://github.com/foxbot
[@fwrs]: https://github.com/fwrs
[@GetRektByMe]: https://github.com/GetRektByMe
[@hsiW]: https://github.com/hsiW
[@iCrawl]: https://github.com/iCrawl
[@indiv0]: https://github.com/indiv0
[@khazhyk]: https://github.com/khazhyk
[@sschroe]: https://github.com/sschroe
[@SunDwarf]: https://github.com/SunDwarf
[@Roughsketch]: https://github.com/Roughsketch

[c:00fb61b]: https://github.com/zeyla/serenity/commit/00fb61b5f306aebde767cc21a498a8ca0742d0be
[c:01f6872]: https://github.com/zeyla/serenity/commit/01f687204dd9d5564ec4bdc860f11bfd5e01454f
[c:04cfaa9]: https://github.com/zeyla/serenity/commit/04cfaa9a69dc1638e9cd1904a9b8e94c1a97f832
[c:0708ccf]: https://github.com/zeyla/serenity/commit/0708ccf85bac347e59053133a2b8b6f2eabe99ba
[c:096b0f5]: https://github.com/zeyla/serenity/commit/096b0f57aae04a5e0ea28414f5016eeafc5b9e0a
[c:0a2f5ab]: https://github.com/zeyla/serenity/commit/0a2f5ab525022fbf0055649f2262573fb07cf18c
[c:0b9bf91]: https://github.com/zeyla/serenity/commit/0b9bf91f62eef85a4eca703902077f4c04b3b6d1
[c:0c9ec37]: https://github.com/zeyla/serenity/commit/0c9ec377aa7281fb3d4bc390c896b426660a5387
[c:0ec4dfb]: https://github.com/zeyla/serenity/commit/0ec4dfb785459c0d04c295f84a1c33e71c016eba
[c:0f41ffc]: https://github.com/zeyla/serenity/commit/0f41ffc811827fdd45e4e631884909e33fa8769e
[c:13de5c2]: https://github.com/zeyla/serenity/commit/13de5c2e50410c3a68435dc774537b490bb7115c
[c:143337a]: https://github.com/zeyla/serenity/commit/143337ae717773f59562d67f85d0e9c44063a45b
[c:147cf01]: https://github.com/zeyla/serenity/commit/147cf01d4f13e3ee15eb03705ab2b7a006851cdd
[c:1561f9e]: https://github.com/zeyla/serenity/commit/1561f9e36384a215d2b866a752996f80d36a3ede
[c:1594961]: https://github.com/zeyla/serenity/commit/159496188b2c841a65829328cddafef620c517af
[c:16bd765]: https://github.com/zeyla/serenity/commit/16bd765112befd5d81818cab7b97ac59bd8a1b75
[c:16d1b3c]: https://github.com/zeyla/serenity/commit/16d1b3cad3982accd805f64ef93e51d921b3da55
[c:2416813]: https://github.com/zeyla/serenity/commit/24168137ff7b1ec44d3ecdec0f516455fd3785a7
[c:268f356]: https://github.com/zeyla/serenity/commit/268f356a25f27175a5d72458fff92b0f770d0a5a
[c:2844ae1]: https://github.com/zeyla/serenity/commit/2844ae158f3d8297b17a584ff9a75f1f51116f48
[c:2a743ce]: https://github.com/zeyla/serenity/commit/2a743cedaf08f7eb532e3c4b795cfc5f85bc96af
[c:2b237e7]: https://github.com/zeyla/serenity/commit/2b237e7de221beab9c516d6de29f83188ef63840
[c:2cb607d]: https://github.com/zeyla/serenity/commit/2cb607d72a39aa7ab3df866b23de4c9798e69a0f
[c:2eaa415]: https://github.com/zeyla/serenity/commit/2eaa4159955260e7c9ade66803d69865f1f76018
[c:31aae7d]: https://github.com/zeyla/serenity/commit/31aae7d12763f94a7a08ea9fd0102921e8402241
[c:31becb1]: https://github.com/zeyla/serenity/commit/31becb16f184cd7d396b383ad4abed8095451fcb
[c:3348178]: https://github.com/zeyla/serenity/commit/3348178f151d8e1d7aa0432984a2dd23fa7b9e89
[c:345e140]: https://github.com/zeyla/serenity/commit/345e1401142d21a0fdabb2accd1f33e3a07c02c8
[c:38a484d]: https://github.com/zeyla/serenity/commit/38a484d0fec91e290bc1633fc871131f9decd0ca
[c:38db32e]: https://github.com/zeyla/serenity/commit/38db32e2cbb9dc8504e0dfbc2366b17596836da0
[c:3c7c575]: https://github.com/zeyla/serenity/commit/3c7c575d988f4dc793678880560aee48456f4526
[c:3ca7ad9]: https://github.com/zeyla/serenity/commit/3ca7ad92507f056054d081485f437c08505bc7e5
[c:3f03f9a]: https://github.com/zeyla/serenity/commit/3f03f9adc97315bb61a5c71f52365306cb8e2d1a
[c:404a089]: https://github.com/zeyla/serenity/commit/404a089af267c5d5c33025a3d74826e02b6f8ca1
[c:4229034]: https://github.com/zeyla/serenity/commit/42290348bc05c876b7e70c570a485160e594e098
[c:43a5c5d]: https://github.com/zeyla/serenity/commit/43a5c5d7eb8bffb8c9ca450ab1bc377d602fb8c3
[c:49a6841]: https://github.com/zeyla/serenity/commit/49a684134df32427e9502192122c4fb22ef1a735
[c:4a14b92]: https://github.com/zeyla/serenity/commit/4a14b92ff58173acb98c7e0a135b4989a87a7529
[c:4cf8338]: https://github.com/zeyla/serenity/commit/4cf8338e364b0feefef26ece6649077e87962ff3
[c:4de39da]: https://github.com/zeyla/serenity/commit/4de39da887248e374b4d824472a6422c7e46dacc
[c:55ccaca]: https://github.com/zeyla/serenity/commit/55ccaca57051b3fbd47cf7fa288014d9c36f6952
[c:57c060f]: https://github.com/zeyla/serenity/commit/57c060fa2fccfbb3b3d4b2d18aad2faa5929deb3
[c:585af23]: https://github.com/zeyla/serenity/commit/585af231028e46788d689f94e14e110c072a578e
[c:5918d01]: https://github.com/zeyla/serenity/commit/5918d01ed69541e43aed0e62ee6eadbf5ebb20d2
[c:5b275fc]: https://github.com/zeyla/serenity/commit/5b275fc425d4ef1c1a9eaa9d915db1f873f9c11d
[c:5bf6c2d]: https://github.com/zeyla/serenity/commit/5bf6c2d2cf0491951eddb10ab2641d02d0e730a1
[c:5c40e85]: https://github.com/zeyla/serenity/commit/5c40e85001b9b2620a76fcc57d8f0cddfb6f9b34
[c:5ee5fef]: https://github.com/zeyla/serenity/commit/5ee5feff615565b6f661ee3598fe19bb98bd6a88
[c:5fe6a39]: https://github.com/zeyla/serenity/commit/5fe6a3956d39e9b5caef19df78e8b392898b6908
[c:626ffb2]: https://github.com/zeyla/serenity/commit/626ffb25af35f5b91a76fdccf6788382a1c39455
[c:62ed564]: https://github.com/zeyla/serenity/commit/62ed564e5f67f3e25d2307fbbf950d0489a28de8
[c:6355288]: https://github.com/zeyla/serenity/commit/635528875c59d34f0d7b2f2b0a3bd61d762f0e9c
[c:651c618]: https://github.com/zeyla/serenity/commit/651c618f17cb92d3ea9bbd1d5f5c92a015ff64e0
[c:66546d3]: https://github.com/zeyla/serenity/commit/66546d36749f6c78a4957a616524fab734d5c972
[c:68c473d]: https://github.com/zeyla/serenity/commit/68c473dd17a2098f97808b3d1f2a200621f67c9d
[c:69ec62a]: https://github.com/zeyla/serenity/commit/69ec62a42bcb143cdde056ad8ffce81922e88317
[c:6a887b2]: https://github.com/zeyla/serenity/commit/6a887b25f2712d70c65fc85b5cfbd8b6d4b41260
[c:6b0b9b2]: https://github.com/zeyla/serenity/commit/6b0b9b2491fa895bd7dd8e065f067470ea08639d
[c:6e11a10]: https://github.com/zeyla/serenity/commit/6e11a103f7a6a4ab43b1aa511aad9e04b1fd8c5a
[c:6f33a35]: https://github.com/zeyla/serenity/commit/6f33a35c4f85a06c45c4ce9e118db203c4951475
[c:70bf22a]: https://github.com/zeyla/serenity/commit/70bf22a00cd19651a0d994cc43e8d8c4bd8947fc
[c:70d4e75]: https://github.com/zeyla/serenity/commit/70d4e7538cefc21dd0e06d5451888b82f53acf38
[c:760a47a]: https://github.com/zeyla/serenity/commit/760a47aa4d34160f44048e775afeb30f08891c99
[c:76f9095]: https://github.com/zeyla/serenity/commit/76f9095c012a8769c7bd27aca6540b7018574c28
[c:7914274]: https://github.com/zeyla/serenity/commit/79142745cb571ba2d4284fd1dcbe53c14a0ed623
[c:7b45f16]: https://github.com/zeyla/serenity/commit/7b45f16f063a47dc8a302dce5b016cf43a3edcc1
[c:7b4b154]: https://github.com/zeyla/serenity/commit/7b4b1544603a70dd634b51593ea5173b4515889a
[c:7e254c5]: https://github.com/zeyla/serenity/commit/7e254c5c6098bb1a47bac26c9895098a46cdc53f
[c:7f04179]: https://github.com/zeyla/serenity/commit/7f041791aa95e38a0cacd2ab64f0423524c60052
[c:83b1d96]: https://github.com/zeyla/serenity/commit/83b1d967f4cc2040f94d67dd987302347f227d6a
[c:83b29d5]: https://github.com/zeyla/serenity/commit/83b29d5f839cd2ea6cd150aa7b8ccbbc677c1fad
[c:86a4e00]: https://github.com/zeyla/serenity/commit/86a4e008ca7acf23d920e344463df801a774d5ce
[c:86cd00f]: https://github.com/zeyla/serenity/commit/86cd00f20d6f218e524deed040d3c209f0361a86
[c:8c5ee70]: https://github.com/zeyla/serenity/commit/8c5ee70b28b42ac92f899932ab2ddafeb9c6f913
[c:8e2c052]: https://github.com/zeyla/serenity/commit/8e2c052a55e5e08c6e7ed643b399f1a7f69a2b25
[c:8effc91]: https://github.com/zeyla/serenity/commit/8effc918cc3d269b0d4cf34ef4f2053cecad2606
[c:8f24aa3]: https://github.com/zeyla/serenity/commit/8f24aa391f6b8a9103a9c105138c7610288acb05
[c:9114963]: https://github.com/zeyla/serenity/commit/9114963daf708cfaeaf54d8c788206ccfbae5df8
[c:92309b2]: https://github.com/zeyla/serenity/commit/92309b2fb8ffd96292fd2edaa7c223a2ba774a56
[c:9268f9c]: https://github.com/zeyla/serenity/commit/9268f9c10ef47ffeaeb3d5040e65b1093e04b866
[c:933ee89]: https://github.com/zeyla/serenity/commit/933ee8914509e52c5119ced9f5d9d8f9644cfa63
[c:9392f61]: https://github.com/zeyla/serenity/commit/9392f61f8857b6ab2a04781c2d9c92a582a1577b
[c:93f3c60]: https://github.com/zeyla/serenity/commit/93f3c60b23cb8ffd16666bdc01b3502ca7ba5f47
[c:97f9bd1]: https://github.com/zeyla/serenity/commit/97f9bd10c16eb24d54a0ab00c52f19eb51a88675
[c:9aa357f]: https://github.com/zeyla/serenity/commit/9aa357f0c8f504b53b49824cc20561c8501d2dda
[c:9c04a19]: https://github.com/zeyla/serenity/commit/9c04a19015cf579d343d81a7fa50e6f4b18b4a5b
[c:9c1ed6c]: https://github.com/zeyla/serenity/commit/9c1ed6ca933f81bc0254d9d52159b9190b50a3ea
[c:9ec05e7]: https://github.com/zeyla/serenity/commit/9ec05e701bdbadad39847f0dcc18d5156ecdde02
[c:9ef5522]: https://github.com/zeyla/serenity/commit/9ef55224757dff6dec8576bd1ad11db24a10891e
[c:a0bb306]: https://github.com/zeyla/serenity/commit/a0bb30686c1a9431aef23c2e8594791f64035194
[c:a2cbeb6]: https://github.com/zeyla/serenity/commit/a2cbeb6ece9ef56e2082b6eabbabe5fe536ab3ec
[c:a39647d]: https://github.com/zeyla/serenity/commit/a39647d3ba1650a4dd4c92bd40001959828000c7
[c:a8acd61]: https://github.com/zeyla/serenity/commit/a8acd6138741a6e5268141ac4ce902561931d353
[c:ab778f8]: https://github.com/zeyla/serenity/commit/ab778f8a9cf47c4e27fe688a61effb0caa4f8a6e
[c:abd22d2]: https://github.com/zeyla/serenity/commit/abd22d289599530cbd1bc9cf1b739420f0d22372
[c:ada07fa]: https://github.com/zeyla/serenity/commit/ada07fae09f3521f44d81613f26839d69c1fc7ef
[c:ae352ea]: https://github.com/zeyla/serenity/commit/ae352ea3df86eb2d853d5b1af048a95409aafc38
[c:b001234]: https://github.com/zeyla/serenity/commit/b0012349cca2a5c7c62bb6d2c99106d245b6c55a
[c:b7cbf75]: https://github.com/zeyla/serenity/commit/b7cbf75103939b0b7834c808050b19ba4fbc4b17
[c:b96f85c]: https://github.com/zeyla/serenity/commit/b96f85c224b9c0478b7f1b5c5b76761e23ff7edf
[c:bad9ac3]: https://github.com/zeyla/serenity/commit/bad9ac3d28bb0417dedcdddf10cf764c08d1d6ae
[c:bb97211]: https://github.com/zeyla/serenity/commit/bb97211b2b107943dd6fabb7a0a344d4fe236780
[c:bcb70e8]: https://github.com/zeyla/serenity/commit/bcb70e85384a16b2440788a73241f507aaeba4dc
[c:bceb049]: https://github.com/zeyla/serenity/commit/bceb049bb2b804dac975567bb7eac6afcfc28574
[c:c01f238]: https://github.com/zeyla/serenity/commit/c01f238a34ad846f8732c8bf97fbbd96fbf6a7ae
[c:c032fbe]: https://github.com/zeyla/serenity/commit/c032fbe7a5c65fb6824a5eb36daf327134b854cf
[c:c050c59]: https://github.com/zeyla/serenity/commit/c050c59da25b9093a75bda22baa81be3b267c688
[c:c2e8b69]: https://github.com/zeyla/serenity/commit/c2e8b69702cf81a1cf149c420aec999124f398e2
[c:c36841d]: https://github.com/zeyla/serenity/commit/c36841dd1c3f80141251ba01130333f43ff363d7
[c:c74cc15]: https://github.com/zeyla/serenity/commit/c74cc15f8969c8db68119d07a4f273a0d3fc44f4
[c:c8536c1]: https://github.com/zeyla/serenity/commit/c8536c111117f26833fb1bceff734ac1abc55479
[c:c8c6b83]: https://github.com/zeyla/serenity/commit/c8c6b83ca685a3e503c853d4154a17761790954e
[c:cd914f5]: https://github.com/zeyla/serenity/commit/cd914f503c8f0ada7473b5b56e4ad7830370ea45
[c:d144136]: https://github.com/zeyla/serenity/commit/d1441363364970b749d57b8a4863b284239488d1
[c:d3389be]: https://github.com/zeyla/serenity/commit/d3389be3042fd7977350a08152d177ac6cdcd37f
[c:d4fc8b6]: https://github.com/zeyla/serenity/commit/d4fc8b6188627ae8d553cf282b1371e3de7b01f9
[c:d58c544]: https://github.com/zeyla/serenity/commit/d58c54425a18bbbdc8e66e8eebfb8191bad06901
[c:d9118c0]: https://github.com/zeyla/serenity/commit/d9118c081742d6654dc0a4f60228a7a212ca436e
[c:daf92ed]: https://github.com/zeyla/serenity/commit/daf92eda815b8f539f6d759ab48cf7a70513915f
[c:e1079e9]: https://github.com/zeyla/serenity/commit/e1079e9a03473f9ec67414628d5b84e7ea1b5b38
[c:e2557ac]: https://github.com/zeyla/serenity/commit/e2557ac794068c1a6a5c4c674ed9f7b7a806068e
[c:e4b484f]: https://github.com/zeyla/serenity/commit/e4b484f1c823ccb0aa2be7c54e0def07e5a01806
[c:e5a83dd]: https://github.com/zeyla/serenity/commit/e5a83dd1873e5af2df18835d960fe19286c70f1e
[c:e6712c9]: https://github.com/zeyla/serenity/commit/e6712c9459c367cf9ba3e5d9bf1c0831357a20b5
[c:e7110ad]: https://github.com/zeyla/serenity/commit/e7110adb1e5659b7395588381c2e56c2aa06d1fa
[c:e85e901]: https://github.com/zeyla/serenity/commit/e85e901062e8b9ea717ec6c6253c9c7a300448d3
[c:e891ebe]: https://github.com/zeyla/serenity/commit/e891ebeba43eb87c985db4e031b8bf76dcaca67b
[c:e8a9086]: https://github.com/zeyla/serenity/commit/e8a90860d1e451e21d3bf728178957fe54cf106d
[c:e9282d3]: https://github.com/zeyla/serenity/commit/e9282d3373158b6e9792a5484ae3dfb9212eb6f7
[c:e9aae9c]: https://github.com/zeyla/serenity/commit/e9aae9c043b206b15bd5429126ded62259d6731b
[c:eb09f2d]: https://github.com/zeyla/serenity/commit/eb09f2d3389b135978e0671a0e7e4ed299014f94
[c:ef6eba3]: https://github.com/zeyla/serenity/commit/ef6eba37636a487c0d6f3b93b8e76c94f28abbab
[c:f00e165]: https://github.com/zeyla/serenity/commit/f00e1654e8549ec6582c6f3a8fc4af6aadd56015
[c:f0d1157]: https://github.com/zeyla/serenity/commit/f0d1157212397ae377e11d4205abfebc849ba9d8
[c:f3f74ce]: https://github.com/zeyla/serenity/commit/f3f74ce43f8429c4c9e38ab7b905fb5a24432fd4
[c:f53124e]: https://github.com/zeyla/serenity/commit/f53124ec952124f5b742f204cdf7e1dc00a168ab
[c:f57a187]: https://github.com/zeyla/serenity/commit/f57a187d564bdcd77f568e77a102d6d261832ee0
[c:f69512b]: https://github.com/zeyla/serenity/commit/f69512beaa157775accd4392295dba112adcf1df
[c:f6b27eb]: https://github.com/zeyla/serenity/commit/f6b27eb39c042e6779edc2d5d4b6e6c27d133eaf
[c:f847638]: https://github.com/zeyla/serenity/commit/f847638859423ffaaecfdb77ee5348a607ad3293
[c:f894cfd]: https://github.com/zeyla/serenity/commit/f894cfdc43a708f457273e1afb57ed1c6e8ebc58
[c:f96b6cc]: https://github.com/zeyla/serenity/commit/f96b6cc5e1e0383fd2de826c8ffd95565d5ca4fb
[c:fafa363]: https://github.com/zeyla/serenity/commit/fafa3637e760f0c72ae5793127bc2f70dcf2d0e2
[c:fb07751]: https://github.com/zeyla/serenity/commit/fb07751cfc1efb657cba7005c38ed5ec6b192b4f
[c:fb4d411]: https://github.com/zeyla/serenity/commit/fb4d411054fa44928b4fa052b19de19fce69d7cf
[c:ff4437a]: https://github.com/zeyla/serenity/commit/ff4437addb01e5c6c3ad8c5b1830db0d0a86396b
