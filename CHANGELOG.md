# Change Log

All notable changes to this project will be documented in this file.
This project mostly adheres to [Semantic Versioning][semver].

## [0.5.3] - 2018-05-01

Thanks to the following for their contributions:

- [@acdenisSK]
- [@FelixMcFelix]
- [@Lakelezz]
- [@zeyla]

### Added

- [http] Take `Date` header into account when ratelimiting ([@zeyla])
  [c:40db3c0]
- [general] Add new join messages ([@zeyla]) [c:36d7a54]

### Fixed

- [voice] Send silence frames upon connection ([@FelixMcFelix]) [c:83a0c85]
- [general] Remove spurious import warning ([@acdenisSK]) [c:64dcced]
- [docs] Fix dead link ([@Lakelezz]) [c:42063a2]
- [model] Fix "Guild Member Chunk" deserializations ([@zeyla]) [c:fd77a91]
- [voice] Fix voice hang ([@FelixMcFelix]) [c:e546fa2]
- [client] Fix panics on some guild member updates in certain situations
  ([@zeyla]) [c:526c366]

### Misc.

- [gateway] Clarify shard sequence-off log ([@zeyla]) [c:7f9c01e]
- [client] Log more information about failed deserializations ([@zeyla])
- [framework] Reword command macro docs ([@acdenisSK]) [c:a481df6]

## [0.5.2] - 2018-04-14

This release contains the usual bugfixes and helper methods.

Thanks to the following for their contributions:

- [@acdenisSK]
- [@FelixMcFelix]
- [@ForsakenHarmony]
- [@jkcclemens]
- [@Lakelezz]
- [@megumisonoda]
- [@Roughsketch]
- [@Scetch]
- [@xentec]
- [@zeyla]

### Added

- [builder] Generalize `CreateEmbed` method parameters ([@acdenisSK])
  [c:f115c17]
- [http] Add 'Get Guild Vanity Url' endpoint ([@zeyla]) [c:dbfc06e]
- [framework] Add `unrecognized_command` method ([@Lakelezz]) [c:2937792]
- [client] Add documentation to `EventHandler` ([@acdenisSK]) [c:80dfcb0]
- [http] Support sending files with an embed ([@Scetch]) [c:7e0d908]

### Fixed

- [voice] Add `Drop` impl for ffmpeg container ([@FelixMcFelix]) [c:3d67a4e]
- [model] Pad user discrims in `content_safe` ([@megumisonoda]) [c:2ab714f]
- [framework] Properly check if `Args` input is empty ([@acdenisSK])
  [c:beebff5]
- [voice] Backport [c:7d162b9] (voice fixes) ([@FelixMcFelix]) [c:9baf167]
- [framework] Fix no-cache StandardFramework compilations ([@Lakelezz])
  [c:02dc506]
- [builder] Make `CreateEmbed` and `CreateMessage` consistent ([@acdenisSK])
  [c:77c399b]
- [framework] Fix `help` command precedence ([@acdenisSK]) [c:c6a5fe4]
- [gateway] Fix heartbeat checking ([@zeyla]) [c:21fe999]
- [framework] Fix `Args::is_empty` behaviour ([@acdenisSK]) [c:e5bcee7]
- [framework] Add `Args::full_quotes` ([@acdenisSK]) [c:24d2233]
- [http] Do not include Optional params if None for audit logs ([@jkcclemens])
  [c:bd195de]
- [model] Handle deserializing `AuditLogEntry::target_id` ([@acdenisSK])
  [c:0d779ba]
- [model] Fix `AuditLogOptions` to be correct types ([@acdenisSK], [@jkcclemens]) [c:217e1c6], [c:2791ed7]

### Misc.

- [builder] DRY in `CreateEmbed` builder methods
  ([@xentec], [@acdenisSK], [@zeyla]) [c:2e1eb4c] [c:d8c9d89], [c:a4cc582],
  [c:ffc5ea1]
- [builder] Inline some CreateEmbed builder methods ([@acdenisSK]) [c:e814e9a]
- [framework] Add tests for empty messages ([@Lakelezz]) [c:d0ae9bb]
- [general] Remove useless clones ([@Roughsketch]) [c:b71d99f]
- [framework] Add `no_run` to doctests that instantiate a Client
  ([@Roughsketch]) [c:003dc2e]
- [general] Don't create enums and IDs via macros ([@ForsakenHarmony])
  [c:fdcf44e]
- [framework] Short-circuit on errors ([@acdenisSK]) [c:82e21a6]
- [model, utils] Fix nsfw related docs ([@Lakelezz]) [c:7f09642]
- [framework] Improve docs for `Args` ([@acdenisSK]) [c:b9fa745]
- [general] Fix some documentatoin typos ([@Lakelezz]) [c:e506e9f]

## [0.5.1] - 2018-01-31

This release contains a number of fixes, a few more model helper methods, and
additional framework features.

Thanks to the following for their contributions:

- [@acdenisSK]
- [@ConcurrentMarxistGC]
- [@FelixMcFelix]
- [@indiv0]
- [@Lakelezz]
- [@perryprog]
- [@zeyla]

### Added

- [framework] Add way to register middleware functions directly on
  `CreateCommand` ([@acdenisSK]) [c:d193975]
- [model] Add `Message::member` ([@zeyla]) [c:ce2952a]
- [http, model] Add functions to reorder a guild's channels ([@zeyla])
  [c:ab1f11a]
- [voice] Add multiple audio stream playback, volume control, and pausing
  ([@FelixMcfelix]) [c:324a288]

### Fixed

- [framework] Fix incorrect skipping for some prefixes ([@ConcurrentMarxistGC])
  [c:76bcf7d]
- [framework] Trim content after prefix mentions ([@Lakelezz]) [c:27c83e8]
- [voice] Strip RTP header extensions if present ([@indiv0]) [c:e4612ac]
- [voice] Fix voice websocket loop termination ([@indiv0]) [c:55fa37a]
- [model] Account for guild owners in member hierarchy check ([@zeyla])
  [c:03a7e3e]
- [model] Check message ID count in `delete_messages` ([@zeyla]) [c:92c91b8]
- [model] Correctly set newly created roles' positions on new roles ([@zeyla])
  [c:5a0b8a6]
- [voice] Fix an odd-to-use `Into<Option<Box<T>>>` bound ([@zeyla]) [c:eee3168]
- [framework] Fix case insensitivity for aliases ([@Lakelezz]) [c:d240074]
- [docs] Fix broken docs links caused by model module changes ([@zeyla])
  [c:8578d5f]

### Changed

### Misc.

- [general] Reduce number of clones in the library ([@zeyla]) [c:13b0de1]
- [example] Add voice receive example (example 10) ([@zeyla]) [c:b9a7e50]
- [examples, framework] Add docs for customised help functions ([@Lakelezz])
  [c:7912f23]
- [example] Add another message embed builder example ([@perryprog])
  [c:aba1ba6]

## [0.5.0] - 2018-01-20

This release is a rewrite of the client and gateway internals with a minimal
amount of breaking changes for userland code. These changes are mainly to
prepare for Tokio and to reduce the number of atomic operations per received
event, reducing the number of atomic operations by roughly 85%. The framework
has also seen a rewrite, and is now centered around a trait-based design.

Thanks to the following for their contributions:

- [@acdenisSK]
- [@Caemor]
- [@ConcurrentMarxistGC]
- [@drklee3]
- [@fenhl]
- [@Flat]
- [@ftriquet]
- [@hsiW]
- [@indiv0]
- [@jhelwig]
- [@jkcclemens]
- [@Lakelezz]
- [@MOZGIII]
- [@nabijaczleweli]
- [@Roughsketch]
- [@tahahawa]
- [@thelearnerofcode]
- [@timotree3]
- [@zeyla]

### Upgrade Path

Per [c:91c8ec4], the `Guild::default_channel` and
`Guild::default_channel_guarenteed` methods now return
`Option<Arc<Mutex<GuildChannel>>>` instead of `Option<GuildChannel>`. This
avoids a clone. To access the channel, you just have to retrieve a read or write
lock by doing `guild.default_channel()?.read()` or
`guild.default_channel()?.write()`.

Per [c:14b9222], there is a new `Member::default_channel()` function that
returns the default channel for the user. This no longer returns the channel
with the same ID as the guild itself, as this behaviour was changed by Discord.
A member's "default channel" is now the top-most channel that it has permission
to view. Accordingly, `Guild::default_channel` matches this behaviour.

Per [c:93e0a42], the library now uses the `parking_lot` crate's `Mutex` and
`RwLock` implementations over the stdlib's. `parking_lot`s implementations are
more efficient, do not poison due to lock drops on unwinding, and implement
eventual fairness.

To account for this, change all `Mutex` lock retrievals and `RwLock` read and
write lock retrievals to not unwrap. `parking_lot`'s `Mutex::lock`,
`RwLock::read`, and `RwLock::write` don't return Results, unlike the `stdlib`'s.

Per [c:78c6df9], the `Guild::features` structfield is no longer a
`Vec<Feature>`. Discord adds guild features over time, which can cause guilds
with those new features to fail in deserialization. Instead, we're
future-proofing by making this a `Vec<String>`.

Per [c:65e3279], the `CreateEmbed` builder's `field` and `fields` functions no
longer take a builder as the argument, and instead take 3 arguments. For
example, code like this:

```rust
channel.send_message(|m| m
    .embed(|e| e
        .title("This is an embed")
        .field(|f| f
            .name("Test field")
            .value("Test value")
            .inline(true))));
```

Would now be this:

```rust
channel.send_message(|m| m
    .embed(|e| e
        .title("This is an embed")
        .field("Test field", "Test value", true)))
```

Per [c:ad0dcb3], shards can no longer have their `afk` property set, as this was
a leftover from user account support. This removes the `afk` parameter of the
`Context::set_presence` function, removal of the parameter from the
`Shard::set_presence` function, and the `Shard::set_afk` function.

Per [c:b328b3e], the `client::EventHandler` no longer prefixes all trait methods
with `on_`. An implementation that looks like this:

```rust
use serenity::client::{Context, EventHandler};
use serenity::model::Message;

struct Handler;

impl EventHandler for Handler {
    fn on_message(&self, _: Context, msg: Message) {
        // ...
    }
}
```

Now looks like this:

```rust
use serenity::client::{Context, EventHandler};
use serenity::model::channel::Message;

struct Handler;

impl EventHandler for Handler {
    fn message(&self, _: Context, msg: Message) {
        // ...
    }
}
```

(a note on the `serenity::model::channel::Message` import later.)

Per [c:b19b031], `Client::new` returns a `Result`, as it now creates some
essential information on instantiation instead of deferring it to when a
connection is started. You can probably just unwrap this Result.

Per [c:b8efeaf], [c:d5a9aa8], and [c:65233ad], the client and gateway internals
have been rebuilt to significantly reduce the number of atomic operations
(upwards of ~85%). This means that retrieval of shard information (like the
shard latency to the Discord gateway or the current connection status) are
retrieved via the encompassing [`ShardManager`][0.5.0:ShardManager] located on
the client. This can be inserted into the client's `data` structfield if you
need to access that information in event or framework command handlers. See
[this example][0.5.0:example-09] for more information. Additionally,
`Context::quit` to shutdown the shard no longer exists; go through the
`ShardManager` instead.

Per [c:aad4744], the framework's `Args::list` function has been renamed to
`Args::multiple` for consistency.

Per [c:f10b9d7], [c:1fd652b], [c:0aa55a2], the framework has been reworked to
be trait-based; thus as per [c:f61816c], [c:4e20277], allowed more useful functionality to commands.

Per [c:05f6ed4], the [client's close handle] has been removed, in favour of
doing so through the `ShardManager`.

Per [c:8c9baa7], the `Guild::default_message_notifications`, `Guild::mfa_level`,
`PartialGuild::default_message_notifications`, and `PartialGuild::mfa_level`
structfields are now enums to represent a stronger type, instead of `u64`s.

Per [c:bcd16dd], the `model` module has been broken up: instead of a giant root
module full of imports, types have been moved to where they fit. For example,
the `Message`, `Reaction`, and `Channel` structs are now in the `model::channel`
module. The `RichInvite`, `Member`, `Role`, and `MfaLevel` types are now in
`model::guild`. Refer to the commit message or the
[`model` module docs][0.5.0:model] for more information.

Per [c:be43836], the `http::HttpError` enum's `InvalidRequest` variant no longer
gives just the HTTP status code of the response. It now includes the full
Response instance.

Per [c:2edba81], the `builder` re-export in the `utils` module no longer exists
after being there in deprecation for a long time. Please import it like so:

```rust
// old
use serenity::utils::builder;

// new
use serenity::builder;
```

### Added

- [framework] Make the framework error's internal String public ([@acdenisSK])
[c:3b9f0f8]
- [client, gateway] Improve shard and shard runner logging ([@zeyla])
[c:f0ee805]
- [gateway] Have `ConnectionStage` derive `Copy` ([@acdenisSK]) [c:551f166]
- [builder, framework, http, model] Replace `Vec<T>` parameter with more generic
`IntoIterator<Item=T>` ([@ftriquet]) [c:b146501], [c:934eb3a]
- [builder, gateway, model, voice] Make more parameters generic with trait
bounds of `AsRef` ([@acdenisSK]) [c:e0e7617], [c:b62dfd4]
- [framework, model] Add help command filtering, member prefix searching
([@Lakelezz]) [c:ee207b3]
- [model] Add guild member filtering functions ([@Lakelezz]) [c:f26dad8]
- [model] `impl BanOptions for &str` ([@acdenisSK]) [c:7c911d5]
- [model] Derive `Default` on IDs and `CurrentUser` ([@acdenisSK]) [c:0881e18]
- [client] Add a threadpool for event dispatches ([@zeyla]) [c:1fa83f7],
[c:3e14067], [c:f2c21ef]
- [model] Fall back to `str::parse` if `parse_username` fails ([@acdenisSK])
[c:8c85664]
- [model] Add a parsing fallback for `RoleId` ([@acdenisSK]) [c:5d4301b]
- [http, model] Expand Audit Log support ([@acdenisSK]) [c:f491809]
- [framework] Make `Command::aliases` public ([@acdenisSK]) [c:8c83866]
- [model] `impl FromStr for ReactionType` ([@acdenisSK]) [c:2032a40],
[c:84706f1]
- [builder] Make trait bounds more generic, from `Into<String>` to `Display`
([@acdenisSK]) [c:05dad71]
- [framework, internal, model, utils] Derive `Debug` on more public types
([@thelearnerofcode]) [c:e5a6f3a]
- [model] Change `PrivateChannel::say` to accept a more generic argument
([@fenhl]) [c:a359f77]
- [model] `impl From<EmojiId, EmojiIdentifier> for ReactionType` ([@fenhl])
[c:68156c9]
- [http] `impl From<&Path> for AttachmentType` ([@zeyla]) [c:7a5aa3c]
- [model] Add `GameType::Listening` ([@hsiW], [@zeyla]) [c:40c5c12], [c:a17fea7]
- [framework] Add `cmd` function to `CreateCommand` and `CreateGroup`
([@acdenisSK]) [c:e748d1f]
- [model] Add `Reaction::message` function ([@Roughsketch]) [c:fd19446]
- [model] Add `Reaction::channel` function ([@zeyla]) [c:e02a842]
- [model] Add `Reaction::user` function ([@zeyla]) [c:82b87f1]
- [model] Implement `Deserialize` for `{,Gateway,Voice}Event` ([@zeyla])
[c:c3aa63f]
- [framework] Add `help()` to `CreateGroup` ([@Lakelezz]) [c:39a1435]
- [framework] Add a way to execute code when a command is registered
([@acdenisSK]) [c:f61816c]
- [framework] Add `before`/`after` middleware to `Command` ([@acdenisSK])
[c:4e20277]
- [general] Switch from `try_opt!` macro to using `?` operator ([@hsiW])
[c:2d23d8b]
- [framework] Make help commands customizable ([@Lakelezz]) [c:031fc92]
- [model] Add `VIEW_AUDIT_LOG` permission ([@Lakelezz]) [c:612e973]
- [model] Fallback to `str::parse` on `ChannelId` `FromStr` impl ([@acdenisSK])
[c:0525ede]
- [model] Add missing fields to `Guild` ([@zeyla]) [c:3d24033], [c:99d17d2],
[c:2abeea5]
- [framework] Add `Args::len` ([@acdenisSK]) [c:2c9b682], [c:b60d037],
[c:143fddd]
- [model] Add variant adapters to `Channel` ([@timotree3]) [c:f0a56f4]
- [model] Add `animated` field to `Emoji` and `ReactionType` ([@zeyla])
[c:f2fa349]
- [framework] Better support for multiple delimiters on `Args` ([@Lakelezz])
[c:62647f5]
- [model] Update `Region` to include new voice regions ([@Flat]) [c:d264cc3]
- [framework] Add `Args::iter_quoted` ([@acdenisSK]) [c:032c5a7]
- [model] Add missing `num` implementations on models ([@zeyla]) [c:0b1f684]
- [client] Add an event for shard connection changes ([@zeyla]) [c:7e46d8f]
- [model] Implement or derive `serde::Serialize` on all models ([@zeyla])
[c:25dddb6]
- [model] Further generic-ify `reaction_users`' `after` parameter ([@zeyla])
[c:85d7d5f]
- [model] Add `Member::highest_role` ([@zeyla]) [c:b7542f4]
- [model] Add `Guild::greater_member_hierarchy` ([@zeyla]) [c:84ff27b]
- [model] Allow channels to be moved in and out of a category ([@jkcclemens])
[c:6587655]
- [cache, model] Create partial member instances for users without a Member
instance ([@zeyla]) [c:d1113c0]

### Fixed

- [gateway] Improve shard reconnection logic ([@zeyla]) [c:45c1f27]
- [gateway] Reset shard heartbeat state on resume ([@zeyla]) [c:ae50886]
- [http] Make `webhook_id` a majour parameter in ratelimiting ([@zeyla])
[c:1735e57]
- [gateway] Resume on resumable session invalidations ([@zeyla]) [c:eb9e8df]
- [client] Fix setting of framework ([@zeyla]) [c:12317b9]
- [framework] Fix help commands to list all eligible commands in DMs
([@Lakelezz]) [c:114e43a]
- [framework] Fix command parsing behaviour when prefix has spaces
([@ConcurrentMarxistGC]) [c:10c56a9]
- [client] Attempt to restart failed shard boots ([@zeyla]) [c:8d68503]
- [client, gateway] Fix shards attempting to re-identify on their own ([@zeyla])
[c:e678883]
- [framework] Fix multiple char delimiters ([@zeyla]) [c:08febb0]
- [framework] Fix `multiple_quoted` ([@Lakelezz]) [c:9aad1aa]
- [model] Fix `#` finding in `Guild::member_named` ([@tahahawa]) [c:a7b67df]
- [builder] Convert embed footers for `impl Form<Embed> for CreateEmbed`
([@drklee3]) [c:9aaa555]
- [framework] Fix plain help command ([@Lakelezz]) [c:4bd223a]
- [model] Correctly iterate over channel permission overwrites in permission
building ([@zeyla]) [c:7566f32]
- [model] Compare instants in `Shard::latency`, avoiding panics ([@zeyla])
[c:08db9fa]
- [model] Add some role hierarchy position checks ([@zeyla]) [c:222382c]
- [framework] Add missing `correct roles` checks in help commands ([@Lakelezz])
[c:470f366]
- [framework] Fix multibyte character-based prefixes ([@ConcurrentMarxistGC])
[c:e611776]

### Changed

- [framework] Change the way users' command handlers are stored ([@acdenisSK])
[c:d90b90c]
- [model] `Guild::{default_channel, default_channel_guarenteed}` now return
an `Arc<Mutex<GuildChannel>>` instead of a clone of the channel ([@acdenisSK])
[c:91c8ec4]
- [framework] Don't default command argument delimiter to `" "` ([@jhelwig])
[c:3a4cb18]
- [model] Change behaviour of `default_channel` to match Discord's new
behaviour ([@hsiW]) [c:14b9222]
- [utils] Disallow Message Builder `I` from being user-implemented
([@acdenisSK]) [c:7cf1e52]
- [general] Switch to `parking_lot::{Mutex, RwLock}` ([@zeyla]) [c:93e0a42]
- [model] Make `{Guild, PartialGuild}::features` a `Vec<String>` ([@zeyla])
[c:78c6df9]
- [builder] Slightly change performance of builders by using `&'static str`s and
a `VecMap` ([@acdenisSK], [@zeyla]) [c:9908999], [c:3a0c890], [c:26fe139]
- [builder] Change `CreateEmbed::field{,s}` to not take builders ([@zeyla])
[c:65e3279]
- [client, gateway] Remove setting of a shard's `afk` field ([@zeyla])
[c:ad0dcb3]
- [client] Remove `on_` prefix to `EventHandler` tymethods ([@zeyla])
[c:b328b3e]
- [client] Make the Client return a result [c:b19b031]
- [client, gateway] Redo client+gateway internals to reduce atomic operations
([@zeyla]) [c:b8efeaf], [c:d5a9aa8], [c:65233ad]
- [framework] Rename `Args::list` -> `Args::multiple` ([@acdenisSK]) [c:aad4744]
- [framework] Make framework use trait-based commands ([@acdenisSK])
[c:f10b9d7], [c:1fd652b], [c:0aa55a2]
- [client] Remove client close handle ([@zeyla]) [c:05f6ed4]
- [model] Change types of some `Guild` and `PartialGuild` structfields
([@zeyla]) [c:8c9baa7]
- [model] Break up the model module ([@zeyla]) [c:bcd16dd]
- [http] Give the full hyper Response in HTTP errors ([@zeyla]) [c:be43836]
- [utils] Remove `builder` module re-export ([@zeyla]) [c:2edba81]
- [framework] Remove `is_bot` state boolean ([@zeyla]) [c:524b8f8]
- [client, framework, gateway, voice] Use an encompassing `InterMessage` enum to
communicate over the gateway ([@zeyla]) [c:9232b8f]

### Misc.

- [general] Simplify `Error`'s `Display` impl ([@zeyla]) [c:ee2bbca]
- [framework] Document that application owners bypass checks ([@fenhl])
[c:b215457]
- [general] Compile all features for docs.rs ([@zeyla]) [c:a96be90]
- [model] Document that `Reaction::{message, users}` methods hit the API
([@zeyla]) [c:141bbfc]
- [builder] Use `ToString` blanket impl for `Display`s ([@acdenisSK])
[c:3ca7e15]
- [framework] Avoid an unwrap in `Args::parse_quotes` ([@zeyla]) [c:60613ef]
- [client] Trim token given in `Client::new` ([@zeyla]) [c:25d79ac]
- [model] Fix a doc typo on `User` ([@Lakelezz]) [c:9da642a]
- [model] Fix docs for `User::has_role` ([@zeyla]) [c:b52eb9f]

[client's close handle]: https://docs.rs/serenity/0.4.7/serenity/client/struct.CloseHandle.html
[0.5.0:ShardManager]: https://docs.rs/serenity/0.5.0/serenity/client/bridge/gateway/struct.ShardManager.html
[0.5.0:example-09]: https://github.com/zeyla/serenity/blob/91cf5cd401d09a3bca7c2573b88f2e3beb9c0948/examples/09_shard_manager/src/main.rs
[0.5.0:model]: https://docs.rs/serenity/0.5.0/serenity/model/index.html

## [0.4.5] - 2017-12-09

This release contains a hotfix for the hotfix release, as well as a slight
behaviour change to the `EditRole` builder.

The last release contained a deserialization implementation fix which seemed to
work after running tests, but it turns out that not all deserialization issues
were fixed.

The `EditRole` builder's Default implementation no longer sets a value for each
field, as this causes problems with stateless editing of roles.

### Fixed

- [model] Fix remaining deserializers [c:52403a5]

### Changed

- [builder] Remove `EditRole::default` implementation [c:795eaa1]

## [0.4.4] - 2017-12-09

This release contains a hotfix for snowflake deserialization on `serde_json`
v1.0.8. Primary development is continuing on the v0.5.x branch and the
[library organization].

### Fixed

- [model] Fix snowflake deserializer [c:77f462e]

## [0.4.3] - 2017-11-01

This release contains bugfixes and marks the final release of the v0.4.x branch.
Future development will continue on the v0.5.x branch.

Thanks to the following for their contributions:

- [@acdenisSK]
- [@ThatsNoMoon]
- [@zeyla]

### Added

- [model] Add `Guild::member_permissions` ([@zeyla]) [c:2ba4d03]

### Changed

- [model] Rename `Guild::permissions_for` to `Guild::permissions_in`, keep an
  alias ([@zeyla]) [c:dcac271]

### Fixed

- [model] Make `Member::permissions` return guild-level permissions ([@zeyla])
  [c:d3eddc6]

### Misc.

- [model] Add some docs to `BanOptions` ([@acdenisSK]) [c:c99091d]
- [model] Have `Guild::has_perms` use `Guild::member_permissions` ([@zeyla])
  [c:1b7101f]
- [http] Slightly clarify ratelimiting documentation ([@zeyla]) [c:3be6e2e]
- [docs] Fix ping bot example ([@ThatsNoMoon]) [c:800e58f]
- [docs] Use consistent token names in examples ([@zeyla]) [c:e219a6a]

## [0.4.2] - 2017-10-29

This release contains the regular bugfixes, new features and slight behaviour
changes.

Thanks to the following people for their contributions:

- [@acdenisSK]
- [@efyang]
- [@Caemor]
- [@Flat]
- [@hsiW]
- [@Lakelezz]
- [@ConcurrentMarxistGC]
- [@zeyla]

### Added

- [general] Add a way to change a role's position ([@Flat]) [c:f47a0c8]
- [general] Add logging and dotenv to example 07 ([@zeyla]) [c:d50b129]
- [general] Add owner + quit function to example 07 ([@zeyla]) [c:41f26b3]
- [framework] Add `PartialEq` impls and doc-tests to `Args` ([@acdenisSK]) [c:f9e5e76]
- [framework] Add "zero-copy" parsing to `Args` ([@acdenisSK]) [c:9428787]
- [framework] Add a debug impl to `DispatchError` ([@acdenisSK]) [c:a58de97]

### Fixed

- [general] Fix clippy warnings ([@hsiW]) [c:fbd6258]
- [model] Fall back to `str::parse` if `utils::parse_username` fails ([@acdenisSK]) [c:292ceda]
- [model] Fix `User::has_role` ([@zeyla]) [c:d3015a0ff]
- [gateway] Fix shard connection ([@zeyla]) [c:585ac6e]
- [gateway] Fix shard shutdown via `Context` ([@zeyla]) [c:3616585]
- [framework] Fix `allow_whitespace` ([@ConcurrentMarxistGC]) [c:e694766]
- [framework, gateway, cache] Properly update emojis in the cache, fix shard re-tries and do some cleanup to `help_commands.rs` ([@Lakelezz]) [c:e02d5fb]

### Changed

- [model] Do equality and hashing on just the user's id ([@acdenisSK]) [c:b7cdf15]
- [model] defer to `delete_message` if there's just one message to delete ([@acdenisSK]) [c:c7aa27d]
- [model] Use the underlaying integer value of `ChannelType` ([@acdenisSK]) [c:e57b510]

### Misc.

- [general] Update dependencies ([@zeyla]) [c:2219bb3]
- [general] Re-export parking_lot's `Mutex` and `RwLock` from the prelude ([@zeyla]) [c:74ec713]
- [general] Update the version in `Cargo.toml` to actually be `v0.4.2` ([@Caemor]) [c:5829c67]
- [general] Cleanup gitignore to have comments ([@hsiW]) [c:ce4f8c2]
- [gateway] Use update syntax for `Shard` ([@efyang]) [c:fcc4e2c]
- [model] Deprecate some methods on `Channel` ([@zeyla]) [c:23ff6f]

## [0.4.1] - 2017-10-14

This release contains bugfixes and some newly added or newly exposed
functionality.

Thanks to the following for their contributions this release:

- [@acdenisSK]
- [@ftriquet]
- [@hsiW]
- [@Lakelezz]
- [@lolzballs]
- [@Roughsketch]
- [@zeyla]

### Added

- [general] Replace Vec parameters by `IntoIterator` ([@ftriquet])[c:55167c3]
- [general] Replace slice parameters by `IntoIterator` ([@ftriquet]) [c:022e35d]
- [model] Add `Guild::members_starting_with` ([@Lakelezz]) [c:b3aa441]
- [model] Add `Guild::members_containing` ([@Lakelezz]) [c:1b167b5]
- [model] `impl<'a> BanOptions for &'a str` ([@acdenisSK]) [c:cf40386]
- [model] Derive `Default` on `CurrentUser` and IDs ([@acdenisSK]) [c:09a8a44]
- [client] Add a configurable, shard-shared threadpool ([@zeyla]) [c:d7621aa],
  [c:8109619]
- [model] Add `Guild::members_username_containing, members_nick_containing`
  ([@Lakelezz]) [c:002ce3a]
- [framework] Add an iterator for `Args` ([@acdenisSK]) [c:0ed1972]
- [framework] Make `has_all_requirements` public ([@Lakelezz]) [c:08d390c]
- [framework] Make default help messages list help for aliases ([@Lakelezz])
  [c:0d1c0f1]

### Fixed

- [model] Use `request_client!` for attachment downloading ([@lolzballs])
  [c:71f709d]
- [client] Fix client no-framework compilation ([@zeyla]) [c:1d4ecb2]
- [client] Fix client shards not filling ([@zeyla]) [c:86d8bdd]
- [model] Fix `User::tag` and `CurrentUser::tag` discrim output ([@zeyla])
  [c:6b9dcf5]
- [framework] Modify `initialized` method purpose ([@acdenisSK]) [c:05f158f]
- [framework] Make command Error string public ([@acdenisSK]) [c:917dd30]
- [client, gateway] Improve shard logic ([@acdenisSK], [@zeyla]) [c:683691f],
  [c:7befcd5]
- [gateway] Reset shard heartbeat state on resume ([@zeyla]) [c:c98cae4]
- [general] Fix font-height and soften the logo ([@Lakelezz]) [c:3b2c246]

### Misc.

- [client, gateway] Improve shard and shard runner logging ([@zeyla])
  [c:21e194b]
- `to_owned` -> `to_string` ([@acdenisSK]) [c:1bf4d9c]
- [general] Fix most clippy warnings ([@Roughsketch]) [c:7945094]
- [framework] Add some docs to `Args` ([@acdenisSK]) [c:8572943]
- [examples] Add `env_logger` bot example [c:0df77b9]
- [general] Fix clippy lints ([@zeyla]) [c:483b069]
- [model] Optimize `Member::roles` ([@hsiW]) [c:8565fa2]
- [general] Internally use a `try_opt!` macro ([@hsiW]) [c:9b0c053]
- [general] Feature-flag extern crates behind their name ([@zeyla]) [c:11b85ca]

## [0.4.0] - 2017-09-25

This release contains a lot of added functionality, minor-scale rewrites,
bugfixes, documentation work, and the beginning of a rewrite to use the tokio
ecosystem.

The release was delayed due to a [fairly majour bug][rust-websocket:issue:137]
in rust-websocket that we have forked over to temporarily fix.

This release was lead in development by [@acdenisSK].

Thanks to the following for their contributions this release:

- [@acdenisSK]
- [@Arcterus]
- [@Bond-009]
- [@blaenk]
- [@hsiW]
- [@imnotbad]
- [@joek13]
- [@Lakelezz]
- [@Roughsketch]
- [@xentec]
- [@zeyla]

### Upgrade Path

Per commits [c:af1061b], [c:cdedf36], and [c:aa307b1], Direct Messaging other
bot users is now disallowed by the API. To fix this, simply don't do it.

Per commit [c:ebc4e51], deprecated functions were finally removed. The following
can simply have their usage renamed:

- `Cache::get_channel` --> `Cache::channel`
- `Cache::get_guild` --> `Cache::guild`
- `Cache::get_guild_channel` --> `Cache::guild_channel`
- `Cache::get_member` --> `Cache::member`
- `Cache::get_private_channel` --> `Cache::private_channel`
- `Cache::get_role` --> `Cache::role`
- `Cache::get_user` --> `Cache::user`
- `ChannelId::get_invites` --> `ChannelId::invites`
- `ChannelId::get_message` --> `ChannelId::message`
- `ChannelId::get_messages` --> `ChannelId::messages`
- `ChannelId::get_reaction_users` --> `ChannelId::get_reaction_users`
- `ChannelId::get_webhooks` --> `ChannelId::webhooks`
- `Channel::get_message` --> `Channel::message`
- `Channel::get_messages` --> `Channel::messages`
- `Channel::get_reaction_users` --> `Channel::reaction_users`
- `Client::login_bot` --> `Client::new`
- `Client::login` --> `Client::new`
- `Colour::get_b` --> `Colour::b`
- `Colour::get_g` --> `Colour::g`
- `Colour::get_r` --> `Colour::r`
- `Colour::get_tuple` --> `Colour::tuple`
- `CurrentUser::distinct` --> `CurrentUser::tag`
- `Group::get_message` --> `Group::message`
- `Group::get_messages` --> `Group::messages`
- `Group::get_reaction_users` --> `Group::reaction_users`
- `Guild::get_bans` --> `Guild::bans`
- `Guild::get_channels` --> `Guild::channels`
- `Guild::get_emoji` --> `Guild::emoji`
- `Guild::get_emojis` --> `Guild::emojis`
- `Guild::get_integrations` --> `Guild::integrations`
- `Guild::get_invites` --> `Guild::invites`
- `Guild::get_member` --> `Guild::member`
- `Guild::get_members` --> `Guild::members`
- `Guild::get_member_named` --> `Guild::member_named`
- `Guild::get_prune_count` --> `Guild::prune_count`
- `Guild::get_webhooks` --> `Guild::webhooks`
- `GuildId::get_bans` --> `GuildId::bans`
- `GuildId::get_channels` --> `GuildId::channels`
- `GuildId::get_emoji` --> `GuildId::emoji`
- `GuildId::get_emojis` --> `GuildId::emojis`
- `GuildId::get_integrations` --> `GuildId::integrations`
- `GuildId::get_invites` --> `GuildId::invites`
- `GuildId::get_member` --> `GuildId::member`
- `GuildId::get_members` --> `GuildId::members`
- `GuildId::get_prune_count` --> `GuildId::prune_count`
- `GuildId::get_webhooks` --> `GuildId::webhooks`
- `Message::get_reaction_users` --> `Message::reaction_users`
- `PartialGuild::get_bans` --> `PartialGuild::bans`
- `PartialGuild::get_channels` --> `PartialGuild::channels`
- `PartialGuild::get_emoji` --> `PartialGuild::emoji`
- `PartialGuild::get_emojis` --> `PartialGuild::emojis`
- `PartialGuild::get_integrations` --> `PartialGuild::integrations`
- `PartialGuild::get_invites` --> `PartialGuild::invites`
- `PartialGuild::get_member` --> `PartialGuild::member`
- `PartialGuild::get_members` --> `PartialGuild::members`
- `PartialGuild::get_prune_count` --> `PartialGuild::prune_count`
- `PartialGuild::get_webhooks` --> `PartialGuild::webhooks`
- `PrivateChannel::get_message` --> `PrivateChannel::message`
- `PrivateChannel::get_messages` --> `PrivateChannel::messages`
- `PrivateChannel::get_reaction_users` --> `PrivateChannel::reaction_users`
- `Role::edit_role` --> `Role::edit`
- `User::distinct` --> `User::tag`

`http::send_file` has been replaced by `http::send_files`. Instead of using `http::send_file` like so:

```rust
use serde_json::Map;
use serenity::http;
use serenity::model::ChannelId;
use std::fs::File;

let channel_id = ChannelId(253635665344987136);
let filename = "mr-sakamoto.png";
let file = File::open(&format!("./img/{}", filename))?;
let map = Map::<String, Value>::new();

http::send_file(channel_id, file, filename, map)?;
```

Instead send an attachment of files, such as:

```rust
use serde_json::Map;
use serenity::http;
use serenity::model::ChannelId;
use std::fs::File;

let channel_id = ChannelId(253635665344987136);
let files = vec![
    (File::open(&format!("./img/{}", filename))?, filename),
];
let map = Map::<String, Value>::new();

http::send_files(channel_id, files, map)?;
```

Similar logic can be applied to shortcut methods which have been removed,
namely:

- `Channel::send_file` (instead use `Channel::send_files`)
- `ChannelId::send_file` (instead use `ChannelId::send_files`)
- `Group::send_file` (instead use `Group::send_files`)
- `GuildChannel::send_file` (instead use `GuildChannel::send_files`)
- `PrivateChannel::send_file` (instead use `PrivateChannel::send_files`)

Instead of using the now-removed `Channel::delete_messages` and
`Channel::delete_permission`, use the inner channel's method:

```rust
use serenity::model::{Channel, ChannelId};

let channel = ChannelId(253635665344987136).get()?;
let message_ids = vec![
    MessageId(359845483356749825),
    MessageId(359854838403694592),
];

if let Channel::Guild(c) = channel {
    c.delete_messages(&message_ids)?;
}
```

Similar logic can be applied to `Channel::delete_permission`.

`Member::find_guild` ended up being only a shortcut to the `Member::guild_id`
structfield. Instead of calling the `find_guild` method like
`member.find_guild()`, instead access the structfield directly via
`member.guild_id`.

The `model::permissions::{general, text, voice}` methods have been removed, as
they ended up being shortcuts to the `model::permissions::PRESET_GENERAL`,
`model::permissions::PRESET_TEXT`, and `model::permissions::PRESET_VOICE`
constants, respectively.

Per commit [c:ea432af], event handling is now done via implementing a trait.
Instead of passing functions to the client directly like:

```rust
use serenity::Client;
use std::env;

let mut client = Client::new(env::var("DISCORD_TOKEN")?);

client.on_message(|ctx, msg| {
    // code
});
```

Instead implement the new EventHandler trait:

```rust
use serenity::client::{Client, Context, EventHandler};
use serenity::model::Message;

struct Handler;

impl EventHandler for Handler {
    fn on_message(&self, ctx: Context, msg: Message) {
        // code
    }
}

let client = Client::new(env::var("DISCORD_TOKEN")?);
```

Per commit [c:4f2e47f], the deprecated `ext` module (which has recently only
been a series of re-exports for the `cache`, `framework`, and `voice` modules)
was removed. Instead of using `serenity::ext::cache` for example, use
`serenity::cache`.

Per commit [c:878684f], due to the concept of default channels being changed,
`GuildId::as_channel_id` has been deprecated due to the fact that the ID of the
default channel of a guild will no longer necessarily be the same as the guild's
ID.

If you require this _same exact functionality_ (the `GuildId` as a `ChannelId`),
rewrite your code from:

```rust
use serenity::model::GuildId;

let channel_id = GuildId(81384788765712384).as_channel_id();
```

to:

```rust
use serenity::model::{ChannelId, GuildId};

let guild_id = GuildId(81384788765712384);
let channel_id = ChannelId(guild_id.0);
```

Per commits [c:2b053ea], [c:8cc2300], [c:8e29694], and [c:948b27c], custom
frameworks can now be implemented, meaning that a built implementation is now
passed instead of a base framework being provided and mutated. To use the old
framework, modify code from:

```rust
use serenity::Client;
use std::env;

let mut client = Client::new(&env::var("DISCORD_TOKEN")?);

client.with_framework(|f| f
    // method calls to mutate framework here
);
```

to the new style:

```rust
use serenity::client::{Client, EventHandler};
use serenity::framework::standard::StandardFramework;
use std::env;

struct Handler;

impl EventHandler for Handler { }

let mut client = Client::new(&env::var("DISCORD_TOKEN")?, Handler);

client.with_framework(StandardFramework::new()
    // method calls here to mutate framework here
);
```

Per commit [c:fc9eba3d], if you were pattern matching on the
`serenity::framework::DispatchError::CheckFailed` variant, instead either use or
ignore the matched data by rewriting code from:

```rust
use serenity::framework::DispatchError;

// Code to begin dispatch error handling here.

match dispatch_error {
    DispatchError::CheckFailed => {
        // Handle operation here.
    },
    // Other variants.
}
```

to:

```rust
// The standard implementation is now in a "standard" framework module, but
// that's unrelated.
use serenity::framework::standard::DispatchError;

match dispatch_error {
    DispatchError::CheckFailed(_) => {
        // Handle operation here.
    },
    // Other variants.
}
```

Per commits [c:45d72ef], [c:03b6d78], and [c:d35d719], the framework's
`command!` macro no longer parses arguments' types for you. You are now given an
`Args` struct that you can retrieve arguments from and parse from to a requested
type that implements `FromStr`.

For example, a simple sum function that looked like:

```rust
#[macro_use] extern crate serenity;

command!(sum(_ctx, msg, _args, x: i64, y: i64) {
    let _ = msg.reply(&format!("Result: {}", x + y));
});
```

Now looks like:

```rust
use serenity::client::Context;
use serenity::framework::standard::Args;
use serenity::model::Message;

fn sum(_: &mut Context, msg: &Message, args: Args) -> Result<(), String> {
    let x = match args.single::<i64>() {
        Ok(x) => x,
        Err(_) => return Ok(()),
    };
    let y = match args.single::<i64>() {
        Ok(y) => y,
        Err(_) => return Ok(()),
    };

    let _ = msg.reply(&format!("Result: {}", x + y));
}
```

Per commit [c:562ce49], `serenity::model::User`'s `FromStr` implementation can
now hit the REST API. No code changes required, but do note the possibility.

Per commit [c:40031d9], the following routes have been removed for being userbot
routes, which are leftovers from when serenity supported them and had them
removed:

- `http::get_application_info`
- `http::get_applications`
- `http::get_emoji`
- `http::get_emojis`
- `model::Guild::emoji`
- `model::Guild::emojis`
- `model::GuildId::emoji`
- `model::GuildId::emojis`
- `model::PartialGuild::emoji`
- `model::PartialGuild::emojis`

Per commit [c:092f288], bitflags has been upgraded, which introduces a minor
change in how to use permissions.

Update code from:

```rust
use serenity::model::permissions::{ADD_REACTIONS, MANAGE_MESSAGES};

foo(vec![ADD_REACTIONS, MANAGE_MESSAGES]);
```

to:

```rust
use serenity::model::Permissions;

foo(vec![Permissions::ADD_REACTIONS, Permissions::MANAGE_MESSAGES]);
```

### Added

- [framework] Make `CommandOrAlias` and `CommandGroup.commands`
  public ([@joek13]) [c:3db42c9]
- [builder] Add support for sending attachments in embeds ([@acdenisSK])
  [c:c68d4d5]
- [client] Add an `on_cached` event ([@acdenisSK]) [c:6d6063f]
- [framework] Add reaction actions
- [client] Add shard shutdown shortcut to the context ([@acdenisSK]) [c:561b0e3]
- [client] Add `is_new` paramenter to the `guild_create` handler ([@acdenisSK])
  [c:3017f6d]
- [http, model] Add ban reasons ([@acdenisSK]) [c:420f9bd], [c:8a33329],
  [c:710fa02], [c:421c709]
- [model] Add `Guild::members_with_status` ([@acdenisSK]) [c:a7a0945],
  [c:29ee627]
- [model] Make `Ban` and `User` impl `Eq`, `Hash`, and `PartialEq`
  ([@acdenisSK]) [c:64bfc54]
- [model] Return error if user exceeds reason limit ([@acdenisSK]) [c:60c33db],
  [c:25d4931]
- [builder] Add method to add multiple embed fields ([@acdenisSK]) [c:dbd6727]
- [model] Make `BanOptions` take and return an `&str` ([@acdenisSK]) [c:1ab8b31]
- [framework] Provide the command to checks ([@acdenisSK]) [c:eb47559]
- [model] Add `{ChannelId, GuildChannel, PrivateChannel}::name` functions
  ([@acdenisSK]) [c:ca0f113]
- [client] Switch to tokio for events ([@Arcterus]) [c:88765d0]
- [client] Add method to close all shards explicitly ([@acdenisSK]) [c:4d4e9dc],
  [c:c2cf691], [c:c7b8ab8], [c:9900b20], [c:d8027d7], [c:051d23d]
- [framework] Implement adding checks to buckets ([@acdenisSK]) [c:dc3a4df]
- [client] Handle the closing of shards ([@blaenk]) [c:5fd3509]
- [client] Make `CloseHandle` derive `Copy` ([@blaenk]) [c:b249c82]
- [model] Add `nsfw` property to channels ([@acdenisSK], [@Bond-009])
  [c:b602805], [c:fd89d09], [c:fd47b86]
- [http, model] Add audit log support ([@acdenisSK]) [c:6a101c4], [c:4532e4a],
  [c:9ccf388], [c:1fad3dd], [c:e2053dd]
- [model] Add `Message::is_own` ([@acdenisSK], [@zeyla]) [c:5a96724],
  [c:fdbfbe0], [c:6572580]
- [utils] Implement `From<(u8, u8, u8)> for Colour` ([@acdenisSK]) [c:6f147e1]
- [builder, model] Make some functions accept a `Display` bound instead of
  `&str` ([@acdenisSK]) [c:7e913b6], [c:05162aa], [c:0810ab7]
- [model] Add simulated default channel methods ([@acdenisSK]) [c:878684f]
- [framework] Add support for custom delimiters ([@acdenisSK]) [c:125c1b8],
  [c:fdfb184]
- [framework] Provide Args to checks ([@acdenisSK], [@Roughsketch]) [c:005437f],
  [c:68c5be8], [c:26919cf], [c:25e91da], [c:ab67c1d], [c:caf69d6]
- [model] Use cache when possible in `UserId::get` ([@Roughsketch]) [c:bfdb57c]
- [utils] Add `with_config{,_mut}` ([@acdenisSK]) [c:1a08904]
- [voice] Add ability to play DCA and Opus ([@Roughsketch]) [c:3e0b103],
  [c:e1a8fe3]
- [model] Add `{Guild,PartialGuild}::role_by_name ([@Lakelezz]) [c:f6fcf32]
- [framework] Add `CreateCommand::num_args` ([@Roughsketch]) [c:aace5fd]
- [framework] Add case insensitive command name support ([@acdenisSK])
  [c:deee38d]
- [framework] Allow commands to be limited to roles ([@Lakelezz]) [c:d925f92]
- [client] Add a way for users to get shards ([@zeyla]) [c:619a91d]
- [cache, client, model] Add channel category support ([@acdenisSK], [@zeyla])
  [c:4be6b9d], [c:870a2a5], [c:192ac8a], [c:485ad29], [c:52b8e29]
- [client] Add `Context::handle` ([@acdenisSK]) [c:97e84fe]
- [framework] Copy some functionality from Command to Group ([@Roughsketch])
  [c:8e1435f]

### Fixed

- [client] Return websocket pings with a pong ([@acdenisSK]) [c:824f8cb],
  [c:e218ce0], [c:e72e25c], [c:bd05bda]
- [utils] Fix `MessageBuilder::push_mono_safe`
- [framework] Fix args when `use_quotes` is active ([@acdenisSK]) [c:e7a5ba3]
- [model] Make `Reaction::name` optional ([@acdenisSK]) [c:8f37f78]
- [gateway] Fix presence updates due to API change ([@Roughsketch]) [c:16a5828]
- [model] Fix `permissions::PRESET_GENERAL` bits ([@zeyla]) [c:9f02720]
- [http] Update deprecated bulk delete endpoint ([@zeyla]) [c:dbcb351]
- [client] Fix subtraction overflow on guild cached dispatch ([@Roughsketch])
  [c:f830f31]
- [framework] Fix admin permission check ([@Lakelezz]) [c:2fb12e2]
- [general] Fix compiles of a variety of feature combinations ([@zeyla])
  [c:8e3b4d6]
- [client] Fix spawning of multiple events (non-v0.3 bug) ([@zeyla]) [c:7c4b052]
- [framework] Add Send/Sync to framework items (non-v0.3 bug) ([@zeyla])
  [c:50d7f00]

### Changed

- [model] Prevent Direct Messaging other bot users ([@zeyla]) [c:af1061b],
  [c:266411c]
- [cache, client] Apply API changes for bot DMs ([@acdenisSK]) [c:cdedf36],
  [c:aa307b1]
- [client] Switch to a trait-based event handler ([@acdenisSK]) [c:ea432af]
- [cache, client, http, model, utils] Remove deprecated functions ([@acdenisSK])
  [c:ebc4e51]
- [framework] Allow custom framework implementations ([@acdenisSK], [@zeyla])
  [c:2b053ea], [c:8cc2300], [c:8e29694], [c:948b27c]
- [general] Remove the BC-purposed `ext` module ([@acdenisSK]) [c:4f2e47f]
- [model] Deprecate `GuildId::as_channel_id` ([@acdenisSK]) [c:878684f]
- [utils] Remove `I` bound for MessageBuilder language params ([@acdenisSK])
  [c:f16af97]
- [cache] Split event handling to a trait ([@acdenisSK]) [c:eee857a],
  [c:32de2cb], [c:bc3491c]
- [framework] Provide command to `DispatchError::CheckFailed` ([@Lakelezz])
  [c:fc9eba3]
- [framework] Provide arguments as an iterable struct
  ([@acdenisSK], [@Roughsketch]) [c:106a4d5], [c:428cbb9], [c:45d72ef],
  [c:03b6d78], [c:d35d719]
- [model] Provide useful user/role/channel id `FromStr` parsing errors
  ([@acdenisSK]) [c:8bf77fa], [c:8d51ead]
- [model] Allow `User`'s `FromStr` impl to hit REST ([@Roughsketch]) [c:562ce49]
- [http] Remove remaining userbot endpoints ([@zeyla]) [c:40031d9]
- [general] Update bitflags, other dependencies ([@zeyla]) [c:092f288]

### Misc.

- [model] Fix a `ModelError` doctest ([@zeyla]) [c:bd9fcf7]
- [docs] Various docs fixes ([@hsiW]) [c:f05efce]
- [docs] Update links to docs ([@zeyla]) [c:78e7b1b]
- [general] Fix clippy warnings ([@imnotbad]) [c:e1912c2]
- [docs] Update to add `EventHandler` ([@acdenisSK]) [c:fdfd5bc]
- [examples] Update examples ([@acdenisSK], [@Roughsketch]) [c:3582691],
  [c:4e360cf]
- [docs] Fix doctests from `EventHandler` changes ([@acdenisSK]) [c:511ec87]
- [docs] Update readme to use correct docs link ([@acdenisSK]) [c:0240717]
- [client] Add a macro for reaction dispatching ([@acdenisSK]) [c:4efe1d1]
- [framework] Simplify an iterator usage ([@acdenisSK]) [c:fbc1ac7]
- [general] Fix clippy warnings ([@imnotbad]) [c:b6af867]
- [docs] Fix the doc on `PrivateChannel::name` ([@acdenisSK]) [c:14fd41b]
- [model, voice] Use stabilized loop-with-break-value ([@acdenisSK]) [c:f5a97d4]
- [model] Change a `match` to an `and_then` ([@acdenisSK]) [c:5e5f161]
- [framework] Make bucket checks less cache dependent ([@acdenisSK]) [c:ea1eba8]
- [framework] Remove unnecessary `Send + Sync` bounds ([@acdenisSK]) [c:3c2716b]
- [client, framework, http, utils] Remove some clones ([@acdenisSK]) [c:0d6965f]
- [cache] Remove an unnecessary map ([@acdenisSK]) [c:924c447]
- [general] Make Travis test on osx ([@Arcterus]) [c:fb2a1a9]
- [cache] Ignore private channels on create if already cached ([@acdenisSK],
  [@Lakelezz]) [c:7e8da0c], [c:e5889ed], [c:069df4f]
- [examples] Document example 05 more heavily ([@Lakelezz]) [c:0186754]
- [examples] Fix listed feature requirements in examples ([@zeyla]) [c:078947e]
- [http] Document and un-hide `http::set_token` ([@zeyla]) [c:cb18d42]
- [model] Refactor Display impl for Ids ([@acdenisSK]) [c:47ea8f7]
- [client] Add a sharding manager base ([@zeyla]) [c:6c43fed]

## [0.3.0] - 2017-06-24

This release contains a number of added methods, fixes, deprecations, and
documentation improvements. It brings a module restructure and an upgrade to
rust-websocket v0.20, hyper v0.10, and switching to `native-tls`, meaning
using an up-to-date rust-openssl v0.9 on Linux, schannel on Windows, and
Secure Transport on Mac. The long-standing issue [#56][issue:56] was closed.

Thanks to the following for their contributions this release:

- [@acdenisSK]
- [@barzamin]
- [@eLunate]
- [@Flat]
- [@fwrs]
- [@hsiW]
- [@Roughsketch]

### Upgrade Path

Invite retrieval functions now accept a `stats` argument. If you don't need
stats, just pass `false`.

`ChannelId::create_permission` and `GuildChannel::create_permission` now accept
a reference, as they do not need to own the overwrite.

The deprecated `GuildChannel` methods (`get_invites`, `get_message`,
`get_messages`, `get_reaction_users`, `get_webhooks`) have been removed. Use
their equivalents without the `get_` prefix.

The `send_file` functions have been deprecated. Use `send_files` instead by
passing a Vec.

`CurrentUser::distinct` and `User::distinct` have been deprecated. Instead use
`CurrentUser::tag` and `User::tag`.

`User::get` has been deprecated. Instead, use `UserId::get`.

`Role::edit_role` has been deprecated, renaming it to `Role::edit`.

`time` has been removed as a direct dependency, moving to `chrono`.
Public-facing fields that return `time::Timespec` or were a String in ISO-3339
format are now `chrono::DateTime<UTC>`s. Instead use its methods for what was
being done with the `Timespec`s or strings.

`User::direct_message` and `User::dm` now accept a builder to allow for more
complete, yet simple use out of the methods. Instead of passing a `&str`, use
the provided builder:

```rust
// old
user.dm("hello")?;

// new
user.dm(|m| m.content("hello"))?;
```

`Client::login` has been deprecated. Instead use `Client::new`:

```rust
use serenity::Client;
use std::env;

// old
let client = Client::login(&env::var("DISCORD_TOKEN")?);

// new
let client = Client::new(&env::var("DISCORD_TOKEN")?);
```

`Member::guild_id` is now no longer an `Option<GuildId>` -- just a `GuildId`.
Since this is now always present, `Member::find_guild` has been deprecated since
the cache now never searches the cache for the guild ID.

The deprecated `GuildChannel` methods `get_invites`, `get_message`,
`get_messages`, `get_reaction_users`, and `get_webhooks` have been removed. Use
their alternatives, such as `GuildChannel::invites`, instead.

### Added

- Add support for retrieving invites with counts ([@hsiW]) [c:302d771]
- Handle message type 7 ([@fwrs]) [c:8f88c6b]
- Add `GuildChannel::permissions_for` [c:6502ded]
- Add `Invite::url()`, `RichInvite::url()` [c:3062981]
- Reasonable derive Debug on all items [c:9dae9e6]
- Add more examples and improve others [c:8c0aeac]
- Support adding reactions when creating message ([@acdenisSK]) [c:77b5b48]
- Add `VerificationLevel::Higher` [c:7dbae6b]
- Add `CurrentUser::invite_url` ([@Roughsketch], [@Flat]) [c:e033ff3],
  [c:0b95db9]
- `impl From<char> for ReactionType` [c:2afab7c]
- Implement multiple attachments ([@Flat]) [c:46b79dd]
- Add `_line` + `_line_safe` methods to `MessageBuilder` ([@Roughsketch])
  [c:543b604]
- Add docs for `CurrentUser` ([@Roughsketch]) [c:921f7f4]
- Add cache docs ([@Roughsketch]) [c:d367a70]
- Add docs and tests for framework ([@Roughsketch]) [c:4267bdb]
- Add `Content` for `MessageBuilder` ([@eLunate]) [c:060b06e]
- Include more info on ratelimiting debugs [c:d37461b]
- Add `User::refresh` [c:8c04d31]
- Add some model docs ([@Roughsketch]) [c:c00f349]
- Add `Message::channel()` [c:063a52f]
- Add `CurrentUser::default_avatar_url` [c:2d09152]
- Add `CurrentUser::face()`, `User::face()` [c:d033909]
- Deserialize embed footers [c:e92b667]
- Add `Member::permissions` [c:39a28d3] ([@acdenisSK])
- Add `wait` parameter to `http::execute_webhook` [c:dc73d1a]

### Fixed

- Don't skip `@everyone` role when checking channel overwrites ([@Roughsketch])
  [c:b468cbf]
- Allow `unreachable_code` lint in `command!` macro ([@Flat]) [c:eb43b9c]
- Fix incorrect attempted `send_file` deserialization [c:0102706]
- Fix ratelimits causing 429s in certain situations [c:f695174]
- Check last heartbeat acknowledged in heartbeater [c:ec9b1c7]
- Make client join shards and return [c:175d3a3]
- Make client starts return an error [c:858bbf2]
- Ws read/write timeout after 90s to avoid infinite blocking [c:1700a4a]
- Fix negative nonces failing to deserialize [c:d0b64cd]
- Use HTTPS Connector with remaining HTTP functions [c:0d218e0] ([@Roughsketch])

### Changed

- Restructure modules [c:9969be6]
- Change `create_permission` to take a reference [c:aea9885]
- Remove deprecated `GuildChannel` methods [c:ab7f113]
- `Guild::create_channel` doesn't require mutability [c:494cc50]
- Deprecate `*User::distinct`, add `*User::tag` [c:6579b1f]
- Deprecate `User::get` [c:afc571f]
- Deprecate `Role::edit_role`, add `Role::edit` [c:c00f349]
- Switch to chrono [c:990e611]
- Make `User::direct_message`/`User::dm` accept a builder [c:11a02db]
- Deprecate `Client::login`, add `Client::new` [c:7990381]
- Make `Member::guild_id` non-optional [c:b4bd771]
- Remove `Context::channel_id` and `Context::queue` [c:8b504ad]
- Make the framework's `dynamic_prefix` accept an `&Message` [c:2845681]
- Deprecate `Channel::delete_messages`, `Channel::delete_permission` [c:7fc49d8]
- Make `Message::nonce` a `serde_json::Value` [c:c832009]

### Misc.

- Remove deprecated `login_bot` usage from docs ([@hsiW]) [c:ae395f4]
- Fix call to `VoiceManager::join` in example 06 ([@barzamin]) [c:6853daf]
- Sort default help by group/command names ([@Roughsketch]) [c:93416cd]
- Move `CreateGroup` docs to the struct [c:71f3dbb]
- Don't create group in help if no commands to show ([@Roughsketch]) [c:4f5fbb5]
- Move user avatar method logic out [c:8360f32]
- Upgrade rust-websocket and hyper, switch to native-tls [c:8f8a059]
- Fix broken links in README [c:51c15d0]
- Remove unused `cookie` dependency [c:92f4ec2]
- Switch from `#[doc(hidden)]` to `pub(crate)` [c:32e07e4] ([@acdenisSK])
- Re-export all errors from the prelude [c:db0f025]
- Rework shard logic and shard handling [c:601704a]

## [0.2.0] - 2017-05-13

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

[0.5.3]: https://github.com/zeyla/serenity/compare/v0.5.2...v0.5.3
[0.5.2]: https://github.com/zeyla/serenity/compare/v0.5.1...v0.5.2
[0.5.1]: https://github.com/zeyla/serenity/compare/v0.5.0...v0.5.1
[0.5.0]: https://github.com/zeyla/serenity/compare/v0.4.7...v0.5.0
[0.4.5]: https://github.com/zeyla/serenity/compare/v0.4.4...v0.4.5
[0.4.4]: https://github.com/zeyla/serenity/compare/v0.4.3...v0.4.4
[0.4.3]: https://github.com/zeyla/serenity/compare/v0.4.2...v0.4.3
[0.4.2]: https://github.com/zeyla/serenity/compare/v0.4.1...v0.4.2
[0.4.1]: https://github.com/zeyla/serenity/compare/v0.4.0...v0.4.1
[0.4.0]: https://github.com/zeyla/serenity/compare/v0.3.0...v0.4.0
[0.3.0]: https://github.com/zeyla/serenity/compare/v0.2.0...v0.3.0
[0.2.0]: https://github.com/zeyla/serenity/compare/v0.1.5...v0.2.0
[0.1.5]: https://github.com/zeyla/serenity/compare/v0.1.4...v0.1.5
[0.1.4]: https://github.com/zeyla/serenity/compare/v0.1.3...v0.1.4
[0.1.3]: https://github.com/zeyla/serenity/compare/v0.1.2...v0.1.3
[0.1.2]: https://github.com/zeyla/serenity/compare/v0.1.1...v0.1.2
[0.1.1]: https://github.com/zeyla/serenity/compare/v0.1.0...v0.1.1
[0.1.0]: https://github.com/zeyla/serenity/tree/403d65d5e98bdfa9f0c018610000c4a0b0c7d8d5
[crates.io listing]: https://crates.io/crates/serenity
[library organization]: https://github.com/serenity-rs
[semver]: http://semver.org

[issue:56]: https://github.com/zeyla/serenity/issues/56
[rust-websocket:issue:137]: https://github.com/cyderize/rust-websocket/issues/137

[@Arcterus]: https://github.com/Arcterus
[@abalabahaha]: https://github.com/abalabahaha
[@acdenisSK]: https://github.com/acdenisSK
[@Bond-009]: https://github.com/Bond-009
[@barzamin]: https://github.com/barzamin
[@bippum]: https://github.com/bippum
[@blaenk]: https://github.com/blaenk
[@Caemor]: https://github.com/Caemor
[@DeltaEvo]: https://github.com/DeltaEvo
[@drklee3]: https://github.com/drklee3
[@eLunate]: https://github.com/eLunate
[@emoticon]: https://github.com/emoticon
[@efyang]: https://github.com/efyang
[@fenhl]: https://github.com/fenhl
[@FelixMcFelix]: https://github.com/FelixMcFelix
[@Flat]: https://github.com/Flat
[@ForsakenHarmony]: https://github.com/ForsakenHarmony
[@foxbot]: https://github.com/foxbot
[@ftriquet]: https://github.com/ftriquet
[@fwrs]: https://github.com/fwrs
[@GetRektByMe]: https://github.com/GetRektByMe
[@hsiW]: https://github.com/hsiW
[@iCrawl]: https://github.com/iCrawl
[@imnotbad]: https://github.com/imnotbad
[@indiv0]: https://github.com/indiv0
[@jhelwig]: https://github.com/jhelwig
[@jkcclemens]: https://github.com/jkcclemens
[@joek13]: https://github.com/joek13
[@Lakelezz]: https://github.com/Lakelezz
[@lolzballs]: https://github.com/lolzballs
[@khazhyk]: https://github.com/khazhyk
[@megumisonoda]: https://github.com/megumisonoda
[@MOZGIII]: https://github.com/MOZGIII
[@nabijaczleweli]: https://github.com/nabijaczleweli
[@perryprog]: https://github.com/perryprog
[@Roughsketch]: https://github.com/Roughsketch
[@Scetch]: https://github.com/Scetch
[@sschroe]: https://github.com/sschroe
[@SunDwarf]: https://github.com/SunDwarf
[@tahahawa]: https://github.com/tahahawa
[@ThatsNoMoon]: https://github.com/ThatsNoMoon
[@thelearnerofcode]: https://github.com/thelearnerofcode
[@timotree3]: https://github.com/timotree3
[@ConcurrentMarxistGC]: https://github.com/ConcurrentMarxistGC
[@xentec]: https://github.com/xentec
[@zeyla]: https://github.com/zeyla

[c:36d7a54]: https://github.com/zeyla/serenity/commit/36d7a541ec53051007fee74f621919bea721c8f2
[c:40db3c0]: https://github.com/zeyla/serenity/commit/40db3c024dd5e13c5a02e10ab4f7a7e9c31a6876
[c:42063a2]: https://github.com/zeyla/serenity/commit/42063a240682f0cfa93b7dce9fcb79b2dfe7ef99
[c:526c366]: https://github.com/zeyla/serenity/commit/526c366eb355ff2cdfd5769621448d35926fe123
[c:64dcced]: https://github.com/zeyla/serenity/commit/64dccedc999b6f2cdf4b60830d4c5fb1126bd37c
[c:7b6b601]: https://github.com/zeyla/serenity/commit/7b6b6016078c3492d2873d3eed0ddb39323079ab
[c:7f9c01e]: https://github.com/zeyla/serenity/commit/7f9c01e4e4d413979e2e66eea1d3cdf9157c4dc5
[c:83a0c85]: https://github.com/zeyla/serenity/commit/83a0c85b0bf87cb4272b5d6e189d139fc31a6d23
[c:a481df6]: https://github.com/zeyla/serenity/commit/a481df6e67d83216617a40d07991ba8ea04d0075
[c:e546fa2]: https://github.com/zeyla/serenity/commit/e546fa2a32a05a9bbc351b9aa789233ee71e88f0
[c:fd77a91]: https://github.com/zeyla/serenity/commit/fd77a91f2ba7c782f3e0e67ecee19df17bb31e26

[c:003dc2e]: https://github.com/zeyla/serenity/commit/003dc2eed0f09cd214373f1581b7794d1483a689
[c:02dc506]: https://github.com/zeyla/serenity/commit/02dc5064d9402f73ef514c9b8ffa318f5d4235ff
[c:0d779ba]: https://github.com/zeyla/serenity/commit/0d779ba85ad43eb7e95be3844ad3fcd74335b47c
[c:21fe999]: https://github.com/zeyla/serenity/commit/21fe999d23cb0e4e76812537b48edadeab5a1540
[c:217e1c6]: https://github.com/zeyla/serenity/commit/217e1c6ebd9577fa5a538b4ecd3c1303ee034462
[c:24d2233]: https://github.com/zeyla/serenity/commit/24d2233ba583221b35eca02cf321e7a4a1adf76d
[c:2791ed7]: https://github.com/zeyla/serenity/commit/2791ed78df18f2721264352611b1ba26b3077196
[c:2937792]: https://github.com/zeyla/serenity/commit/29377922d3d79848efcb8d3bd0fbd52c21e81c5d
[c:2ab714f]: https://github.com/zeyla/serenity/commit/2ab714f9c3683db83eba1f6fe7bb3887a47d4f3f
[c:2e1eb4c]: https://github.com/zeyla/serenity/commit/2e1eb4c35b488ac68a33e76502d0c8f56a72c4b6
[c:3d67a4e]: https://github.com/zeyla/serenity/commit/3d67a4e2cd33b17747c7499e07d0a0e05fe73253
[c:5166a0a]: https://github.com/zeyla/serenity/commit/5166a0a81bac494b0e337d00f946a85d65e4bbbd
[c:77c399b]: https://github.com/zeyla/serenity/commit/77c399ba7b3bff0cf3180df5edad5d6ff6dcb10d
[c:7d162b9]: https://github.com/zeyla/serenity/commit/7d162b96686a56eed052984c18f22f54ad76f780
[c:7e0d908]: https://github.com/zeyla/serenity/commit/7e0d9082fe0262c7b4c4668ca1e1208dffa4796f
[c:7f09642]: https://github.com/zeyla/serenity/commit/7f09642fa66517fc983f63bb2e414638382a6512
[c:80dfcb0]: https://github.com/zeyla/serenity/commit/80dfcb0539c88e9434dc4875d5af55df1aafa725
[c:82e21a6]: https://github.com/zeyla/serenity/commit/82e21a61ae17d8466834f486108d1ace2791efc4
[c:9baf167]: https://github.com/zeyla/serenity/commit/9baf1675b0d1837fe010cfbadac8e80fd9d53898
[c:a4cc582]: https://github.com/zeyla/serenity/commit/a4cc5821c0ca194d321141985cfaf30241f54acf
[c:b71d99f]: https://github.com/zeyla/serenity/commit/b71d99fde84135fa66f73c4817d340ffbe8bddae
[c:b9fa745]: https://github.com/zeyla/serenity/commit/b9fa7457c2011130b28f5eca063f88bdf72be544
[c:bd195de]: https://github.com/zeyla/serenity/commit/bd195dea4422f516b727868209116ff868e3941b
[c:beebff5]: https://github.com/zeyla/serenity/commit/beebff50eb4f447f41162299fde81cb6fa9b336d
[c:c6a5fe4]: https://github.com/zeyla/serenity/commit/c6a5fe4ef9dc07b9b14f742440a35ba6ea02058b
[c:d0ae9bb]: https://github.com/zeyla/serenity/commit/d0ae9bba89d5e66129da7bed7faf39abfd4fb17d
[c:d8c9d89]: https://github.com/zeyla/serenity/commit/d8c9d89c80ffb0ae95b91c4dcc593fd70fa976d8
[c:dbfc06e]: https://github.com/zeyla/serenity/commit/dbfc06e9c6df506839fb178eaeb9db704aefd357
[c:e506e9f]: https://github.com/zeyla/serenity/commit/e506e9f30584d0fd67bf28a33b1033c4f1e5cd8b
[c:e5bcee7]: https://github.com/zeyla/serenity/commit/e5bcee7b2c2d2ff873982c6a3bf39ab16ec4e1e3
[c:e814e9a]: https://github.com/zeyla/serenity/commit/e814e9aa9b6117defa4f885ef0027e61706112d5
[c:f115c17]: https://github.com/zeyla/serenity/commit/f115c177def1a35fc532c896713a187bb468088e
[c:fdcf44e]: https://github.com/zeyla/serenity/commit/fdcf44e1463e708cd8b612c183e302db9af0febd
[c:ffc5ea1]: https://github.com/zeyla/serenity/commit/ffc5ea1da38cb7d9c302fa8d5614253c1f361311

[c:03a7e3e]: https://github.com/zeyla/serenity/commit/03a7e3e1d82ca667ca065367d2cf21b847f984ac
[c:13b0de1]: https://github.com/zeyla/serenity/commit/13b0de121eda30e59849fd442c8a0926a63df2b8
[c:27c83e8]: https://github.com/zeyla/serenity/commit/27c83e8ef0def0a62e8a5ce5bfd4849892749c83
[c:324a288]: https://github.com/zeyla/serenity/commit/324a288fbb0dd7d135aa9aab876cf39dabb6a02e
[c:5a0b8a6]: https://github.com/zeyla/serenity/commit/5a0b8a68c133c3093260a5aeb08b02eb3595c18d
[c:55fa37a]: https://github.com/zeyla/serenity/commit/55fa37ade187aa68ef3eec519d22767920aae4ab
[c:76bcf7d]: https://github.com/zeyla/serenity/commit/76bcf7dcef91fd2658fb3348acf6df0ecc33fcdf
[c:7912f23]: https://github.com/zeyla/serenity/commit/7912f23bed7ddc540c46aee0ecd64c6b60daa0f4
[c:8578d5f]: https://github.com/zeyla/serenity/commit/8578d5fe6e3bdc2842cda9417c242169f93b1a99
[c:92c91b8]: https://github.com/zeyla/serenity/commit/92c91b81490b621de4519e0d87830dbce53dd689
[c:ab1f11a]: https://github.com/zeyla/serenity/commit/ab1f11a37d64166c08f833042d7b3bcde2ea586d
[c:aba1ba6]: https://github.com/zeyla/serenity/commit/aba1ba67dc78a0c14e5de3c8ac650829e436e96f
[c:b9a7e50]: https://github.com/zeyla/serenity/commit/b9a7e50579718a20e60a19f0c0d410661ee3e77a
[c:ce2952a]: https://github.com/zeyla/serenity/commit/ce2952ad0d783b4a256171e48602c6caf1125c61
[c:d193975]: https://github.com/zeyla/serenity/commit/d1939756f6bf4b4bb3c60fbb81a397c218492d62
[c:d240074]: https://github.com/zeyla/serenity/commit/d2400742f657d9f8c432a440810d49e63339f5aa
[c:e4612ac]: https://github.com/zeyla/serenity/commit/e4612acf58dc42fdc32094426c14274bd61203dd
[c:eee3168]: https://github.com/zeyla/serenity/commit/eee3168b4ed266001571abe4e1a6ae4ef06b93e0

[c:031fc92]: https://github.com/zeyla/serenity/commit/031fc92e02c314cce9fc8febcc7900fa2d885941
[c:032c5a7]: https://github.com/zeyla/serenity/commit/032c5a75620c3ff185a749113d93fb3051b38acb
[c:0525ede]: https://github.com/zeyla/serenity/commit/0525ede086ccffa5781c9a1876a368ac3531813e
[c:05f6ed4]: https://github.com/zeyla/serenity/commit/05f6ed429aeac1920307ea692fef122bbd2dffff
[c:0881e18]: https://github.com/zeyla/serenity/commit/0881e18c07113cc7b2f6cec38cadcb1ea03dda12
[c:08db9fa]: https://github.com/zeyla/serenity/commit/08db9fa2adef141743ab9681c46dd91489278063
[c:08febb0]: https://github.com/zeyla/serenity/commit/08febb0d8d95bbbae9861130a756e842eae40eef
[c:0aa55a2]: https://github.com/zeyla/serenity/commit/0aa55a2b9b757321d5b8bb9e512813aa9d0a62ca
[c:0b1f684]: https://github.com/zeyla/serenity/commit/0b1f684106031657d6bf581206c06e5443d06da9
[c:10c56a9]: https://github.com/zeyla/serenity/commit/10c56a9385c0d410241d33525f8f49242daced2d
[c:114e43a]: https://github.com/zeyla/serenity/commit/114e43a3d0079072593588ad7b2de9f8588e041d
[c:12317b9]: https://github.com/zeyla/serenity/commit/12317b98a0cc145e717e9da3cdbe8bc1ff8fc4f1
[c:141bbfc]: https://github.com/zeyla/serenity/commit/141bbfcb1e4843eaeb55bf07e10e2c0aa4bbe1e4
[c:143fddd]: https://github.com/zeyla/serenity/commit/143fddd83f1fc93c070e36bf31906d2631c68f97
[c:14b9222]: https://github.com/zeyla/serenity/commit/14b92221184fcaca0f4a60a3b31d5a9470b14b1f
[c:1735e57]: https://github.com/zeyla/serenity/commit/1735e57ea57bcd4d75b73ac9398e13bee5198c5b
[c:1fa83f7]: https://github.com/zeyla/serenity/commit/1fa83f73577d926664518d849bc26e46087611b4
[c:1fd652b]: https://github.com/zeyla/serenity/commit/1fd652be41f4de96d26efaf20055cf7a80e42bf1
[c:2032a40]: https://github.com/zeyla/serenity/commit/2032a402c387b1310f2ae62621f3e07c86b76aef
[c:222382c]: https://github.com/zeyla/serenity/commit/222382ca48cb9786aaf5d0b5fc16958e482e7c5f
[c:25d79ac]: https://github.com/zeyla/serenity/commit/25d79ac7e07654dbf77166d46db33d186faf9885
[c:25dddb6]: https://github.com/zeyla/serenity/commit/25dddb6695109eeead9e19593cb85a22096c2c7a
[c:26fe139]: https://github.com/zeyla/serenity/commit/26fe139363a847542bbe609fe4d15accbf4fef14
[c:2abeea5]: https://github.com/zeyla/serenity/commit/2abeea53745b5ddfce33d9e1160c682888850344
[c:2c9b682]: https://github.com/zeyla/serenity/commit/2c9b6824a7bf6231a08d5c5465c1db5417ea5d8a
[c:2d23d8b]: https://github.com/zeyla/serenity/commit/2d23d8b50386e38fece6987286bd0b3d56d1cada
[c:2edba81]: https://github.com/zeyla/serenity/commit/2edba816f6901db46e7fc0f6664058033a56d5e7
[c:39a1435]: https://github.com/zeyla/serenity/commit/39a1435be57335e99039ddea731032221eb6d96e
[c:3a0c890]: https://github.com/zeyla/serenity/commit/3a0c8908ce837f6fe64f865a1a7a9de63cbd237c
[c:3a4cb18]: https://github.com/zeyla/serenity/commit/3a4cb18be8ca33d507cbfc88fec79b2a6c5d8bfc
[c:3b9f0f8]: https://github.com/zeyla/serenity/commit/3b9f0f8501f7581d710e3f7ebbfcd3176d14a9b1
[c:3ca7e15]: https://github.com/zeyla/serenity/commit/3ca7e15e55de640200edb3898a33b838946a506c
[c:3d24033]: https://github.com/zeyla/serenity/commit/3d24033f550623f78ad71a37f6ec847e7d0a2c78
[c:3e14067]: https://github.com/zeyla/serenity/commit/3e1406764cf655694fef0e04e43324d58499bba3
[c:40c5c12]: https://github.com/zeyla/serenity/commit/40c5c12373e2a2c7acd3501f43c79f9bf3e7c685
[c:45c1f27]: https://github.com/zeyla/serenity/commit/45c1f27edbeedcb30aa3e9daa78eba44817f7260
[c:470f366]: https://github.com/zeyla/serenity/commit/470f366000b3d3f8080e02b185f0f7fef592a736
[c:4bd223a]: https://github.com/zeyla/serenity/commit/4bd223a88cfacc335814ef3ddc0c1aaa88fc05f7
[c:4e20277]: https://github.com/zeyla/serenity/commit/4e20277de4f164705074ba41199e4530332056b3
[c:524b8f8]: https://github.com/zeyla/serenity/commit/524b8f8ab5153e20ad86be2df7fba6bbed159b7c
[c:551f166]: https://github.com/zeyla/serenity/commit/551f16673fe775a80a1da788fd7e1db20f6eae29
[c:5d4301b]: https://github.com/zeyla/serenity/commit/5d4301bbd2aaa4abe47fbbc2a7a2853ba9b728f2
[c:60613ef]: https://github.com/zeyla/serenity/commit/60613ef696b093dbbac3a4e9e033c226c5d358ea
[c:612e973]: https://github.com/zeyla/serenity/commit/612e973f286ba6b711824333551b07b88df6740c
[c:62647f5]: https://github.com/zeyla/serenity/commit/62647f53fd01a670cf5ad01c7d0a68cb69bf92cf
[c:65233ad]: https://github.com/zeyla/serenity/commit/65233ad6f3d002f72942aaf811514fa9d29ad068
[c:65e3279]: https://github.com/zeyla/serenity/commit/65e3279ce7b3c4807e8b1310551e9493d3868b94
[c:68156c9]: https://github.com/zeyla/serenity/commit/68156c9ce93edc86a70f50cf10986615cfb9f93a
[c:7566f32]: https://github.com/zeyla/serenity/commit/7566f32c194bc4e62e89adc57bfb6104cc99458e
[c:78c6df9]: https://github.com/zeyla/serenity/commit/78c6df9ed3640c097ef088519ec99a6a01796bfc
[c:7a5aa3c]: https://github.com/zeyla/serenity/commit/7a5aa3c57951ee5c7267fabf38f2729b06629b34
[c:7c911d5]: https://github.com/zeyla/serenity/commit/7c911d57eb3db3ac51cfc51cf9b3a5884e0f4ea3
[c:7cf1e52]: https://github.com/zeyla/serenity/commit/7cf1e523f0c0bee1b7ec11ff6e6565c68f9d173e
[c:7e46d8f]: https://github.com/zeyla/serenity/commit/7e46d8f3ac5a968df9a05f8f0006522ad14891ef
[c:82b87f1]: https://github.com/zeyla/serenity/commit/82b87f196425ff8553bc9dcb84ddac9764b971e4
[c:84706f1]: https://github.com/zeyla/serenity/commit/84706f1fc0a934a851d57f524697da5b177b9be8
[c:84ff27b]: https://github.com/zeyla/serenity/commit/84ff27be8455d9ec885b190150a2b592cffdf2a2
[c:85d7d5f]: https://github.com/zeyla/serenity/commit/85d7d5f6a6df9841659bc7ad8e392f31c1aae46c
[c:8c83866]: https://github.com/zeyla/serenity/commit/8c83866748bf7bf339df9a234c3297c8008ffa46
[c:8c85664]: https://github.com/zeyla/serenity/commit/8c85664a94f7439ab4bc3a132f313a9e26d94fe7
[c:8c9baa7]: https://github.com/zeyla/serenity/commit/8c9baa74c2716d62c405d909bb453ffea636c94d
[c:8d68503]: https://github.com/zeyla/serenity/commit/8d685039d89fd2130e88c9a9e0421492a3e99da6
[c:91c8ec4]: https://github.com/zeyla/serenity/commit/91c8ec4ae7540956a714ce9584074538b45467cc
[c:9232b8f]: https://github.com/zeyla/serenity/commit/9232b8f065deb4637a74e7f85ab617bb527c51be
[c:934eb3a]: https://github.com/zeyla/serenity/commit/934eb3aa0b1f9c0aaad003627bd65932114654c1
[c:93e0a42]: https://github.com/zeyla/serenity/commit/93e0a4215c915b98cf433ac6d0bcfbc60f0168ec
[c:9908999]: https://github.com/zeyla/serenity/commit/9908999a6bae1585bb70b7814f13b49bf99b6c32
[c:99d17d2]: https://github.com/zeyla/serenity/commit/99d17d2975143b0d588c969f7ae6f8d11e62a9e1
[c:9aaa555]: https://github.com/zeyla/serenity/commit/9aaa55542d6bee1e953a080612ee6af765b8a5a5
[c:9aad1aa]: https://github.com/zeyla/serenity/commit/9aad1aa375168d6131cb6f68d6998b2af6fb00a3
[c:9da642a]: https://github.com/zeyla/serenity/commit/9da642a5bea8b4ac2d291058ad22e4cbe27b1f94
[c:a17fea7]: https://github.com/zeyla/serenity/commit/a17fea783cd91b2adcd1330b7038cf3ca2d79a85
[c:a359f77]: https://github.com/zeyla/serenity/commit/a359f77d1fd03def94fc08367132a616ec2ea599
[c:a7b67df]: https://github.com/zeyla/serenity/commit/a7b67df6d77f5acacf83710807b231866397d551
[c:a96be90]: https://github.com/zeyla/serenity/commit/a96be90385b58a9098b918e0fd17288d89229752
[c:aad4744]: https://github.com/zeyla/serenity/commit/aad4744fb751e3e1147f085781323172755d4ef2
[c:ad0dcb3]: https://github.com/zeyla/serenity/commit/ad0dcb305d959a2bb273a63dd2dd1b5594f5c49d
[c:ae50886]: https://github.com/zeyla/serenity/commit/ae50886a1a8f69c114d9e99a0913c878aaaaabe2
[c:b146501]: https://github.com/zeyla/serenity/commit/b14650193342297746f985f8794e4b93ceeac52b
[c:b19b031]: https://github.com/zeyla/serenity/commit/b19b031a5052a268f323a116403ea66ca71ea575
[c:b215457]: https://github.com/zeyla/serenity/commit/b215457ab46c9d10bf47300d6525f9a2641d3b17
[c:b328b3e]: https://github.com/zeyla/serenity/commit/b328b3e09b0095abb54530dc4d50db6b4e3e1779
[c:b52eb9f]: https://github.com/zeyla/serenity/commit/b52eb9f108fb7af182e0cf29259cd4d522ed7f89
[c:b60d037]: https://github.com/zeyla/serenity/commit/b60d0378548a53ffefda17aab403c073d3438cf6
[c:b62dfd4]: https://github.com/zeyla/serenity/commit/b62dfd431668b4bdb6021d21120da05d17ab77d5
[c:b7542f4]: https://github.com/zeyla/serenity/commit/b7542f44306fedb7f79f7b8cd5c8d6afd6ccb7ad
[c:b8efeaf]: https://github.com/zeyla/serenity/commit/b8efeaf5e920cbfc775cdee70f23aa41ab7b9dd5
[c:bcd16dd]: https://github.com/zeyla/serenity/commit/bcd16dddb8cc3086a13524c79676f3a8bebbc524
[c:be43836]: https://github.com/zeyla/serenity/commit/be43836839a31714f58e3ffe81dd4b0aeab2ab59
[c:c3aa63f]: https://github.com/zeyla/serenity/commit/c3aa63faee8b3ae6d5126aa27a74876766c61e4c
[c:d1113c0]: https://github.com/zeyla/serenity/commit/d1113c07f061149b5d090c1f15c3c03806f8b0cf
[c:d264cc3]: https://github.com/zeyla/serenity/commit/d264cc3496f56d2757cf9c1735d5d8a68577c2f5
[c:d5a9aa8]: https://github.com/zeyla/serenity/commit/d5a9aa8b1e0a94094ef5bda98a76dd259a6e7a3a
[c:d90b90c]: https://github.com/zeyla/serenity/commit/d90b90c7f3d8a368acbab46150f199866562229a
[c:e02a842]: https://github.com/zeyla/serenity/commit/e02a842fb76b1e591287395ac223cc1c04913820
[c:e0e7617]: https://github.com/zeyla/serenity/commit/e0e76173f63b6071b9df3ff8f53371b4b6c4ee1e
[c:e5a6f3a]: https://github.com/zeyla/serenity/commit/e5a6f3a8ed367bd3d780fd23a0a27f8a80567879
[c:e611776]: https://github.com/zeyla/serenity/commit/e6117760e3c020ed41d643a8b34d7bfeb62d3bfa
[c:e678883]: https://github.com/zeyla/serenity/commit/e6788838556d13d4a4f19253ce297ca2f72168ee
[c:e748d1f]: https://github.com/zeyla/serenity/commit/e748d1ff80dbbeb82b23f8ac9fec9cf8c7e4a69e
[c:eb9e8df]: https://github.com/zeyla/serenity/commit/eb9e8dfbc9d778de405d7369579d90c49a2bf90c
[c:ee207b3]: https://github.com/zeyla/serenity/commit/ee207b331d571d5afb5c35c8f119937d0196663a
[c:ee2bbca]: https://github.com/zeyla/serenity/commit/ee2bbcaa0b62c09a6c0df352bfddcbf99d06e61d
[c:f0a56f4]: https://github.com/zeyla/serenity/commit/f0a56f46ce7ef6c2a65d64d8953ac43e0b7b5b4d
[c:f0ee805]: https://github.com/zeyla/serenity/commit/f0ee805a8ee20b6180b9f54d5096a8a9b73b4be2
[c:f10b9d7]: https://github.com/zeyla/serenity/commit/f10b9d77f0b94864fa20688e3c99de6cec7ca6f9
[c:f26dad8]: https://github.com/zeyla/serenity/commit/f26dad86aea82070aab9cc081f50d0144ee4c778
[c:f2c21ef]: https://github.com/zeyla/serenity/commit/f2c21ef5b15ef1f345cdc30f4b793e55905f15f4
[c:f2fa349]: https://github.com/zeyla/serenity/commit/f2fa349d831c1db59993341284049ffbd56dde3b
[c:f61816c]: https://github.com/zeyla/serenity/commit/f61816ca141add5024e36e073764b7c824872ca4
[c:fd19446]: https://github.com/zeyla/serenity/commit/fd19446fcc4c7ad2c9f634c97fa1c056440a6abd

[c:52403a5]: https://github.com/zeyla/serenity/commit/52403a5084ed7f0589bde3351844907a92de2d62
[c:795eaa1]: https://github.com/zeyla/serenity/commit/795eaa15bca61116fbde9c2482c765f2d47a7696

[c:77f462e]: https://github.com/zeyla/serenity/commit/77f462ea2044ef7d2d12fd1289ea75a6a33cb5dd

[c:1b7101f]: https://github.com/zeyla/serenity/commit/1b7101fe71335c0e18bf855c0703acc23d87e427
[c:2ba4d03]: https://github.com/zeyla/serenity/commit/2ba4d03f15d57d9f0fb1cc4d4f4355ebbc483d0a
[c:3be6e2e]: https://github.com/zeyla/serenity/commit/3be6e2e28b0c3e9baaef19f405c463e3a41fed25
[c:800e58f]: https://github.com/zeyla/serenity/commit/800e58f4603ce99ab69569b30cbec756301a6a63
[c:c99091d]: https://github.com/zeyla/serenity/commit/c99091d241f240c6b76ac969655a8ec4423aaf80
[c:d3eddc6]: https://github.com/zeyla/serenity/commit/d3eddc68e07bbc31e2043577cbf48741f0547ed3
[c:dcac271]: https://github.com/zeyla/serenity/commit/dcac27168915b4f22745950ec0ef0c0af696774e
[c:e219a6a]: https://github.com/zeyla/serenity/commit/e219a6a9d6a890b008fc390a909ae504a0c1a329

[c:002ce3a]: https://github.com/zeyla/serenity/commit/002ce3aa272fa51b84e820f12db39cb87a461a83
[c:022e35d]: https://github.com/zeyla/serenity/commit/022e35d5b12322bd77bbe74a1a3b2ad319977390
[c:05f158f]: https://github.com/zeyla/serenity/commit/05f158fc89f2adc82e31cf4b93706dc7d25e11d8
[c:08d390c]: https://github.com/zeyla/serenity/commit/08d390c19f187986fd2856fe5cbb9035a0877e0f
[c:09a8a44]: https://github.com/zeyla/serenity/commit/09a8a444f5bcefaee8b83dc129a3cea2de8792f9
[c:0d1c0f1]: https://github.com/zeyla/serenity/commit/0d1c0f1356fd3a891232498c2230d0bb4d2ed4ff
[c:0df77b9]: https://github.com/zeyla/serenity/commit/0df77b933ff5e98725252116069afad2dec9f89b
[c:0ed1972]: https://github.com/zeyla/serenity/commit/0ed19727debf28a8aa0818b44713090e97dd6eee
[c:11b85ca]: https://github.com/zeyla/serenity/commit/11b85ca6799b9984481119851f983d8e3c84cdc0
[c:1b167b5]: https://github.com/zeyla/serenity/commit/1b167b5496ce816cbcacb0e4f6e63399dffaa25c
[c:1bf4d9c]: https://github.com/zeyla/serenity/commit/1bf4d9cb9823dca8c4bb77147c66eac2d53f609f
[c:1d4ecb2]: https://github.com/zeyla/serenity/commit/1d4ecb2f13258d286378c44d59c2ee4b1c68349d
[c:21e194b]: https://github.com/zeyla/serenity/commit/21e194bffc37f396f007d390170f5b60e22f5d02
[c:3b2c246]: https://github.com/zeyla/serenity/commit/3b2c2462cb34b5ae5190ebc4a9e04968dc8d5335
[c:483b069]: https://github.com/zeyla/serenity/commit/483b069cc0c821ec673ac475b168809e3a41525a
[c:55167c3]: https://github.com/zeyla/serenity/commit/55167c300598536a852b3596fcf1c420aeb96c3a
[c:683691f]: https://github.com/zeyla/serenity/commit/683691f762bbf58e3abf3bc67381e18112f5c8ad
[c:6b9dcf5]: https://github.com/zeyla/serenity/commit/6b9dcf5272458499c1caef544cb82d5a8624258b
[c:71f709d]: https://github.com/zeyla/serenity/commit/71f709d0aceedb6d3091d0c28c9535e281270f71
[c:7945094]: https://github.com/zeyla/serenity/commit/794509421f21bee528e582a7b109d6a99284510a
[c:7befcd5]: https://github.com/zeyla/serenity/commit/7befcd5caa9ccdf44d90ecc12014c335b1bd2be7
[c:8109619]: https://github.com/zeyla/serenity/commit/8109619184867fc843a1e73d18d37726a34f7fbf
[c:8565fa2]: https://github.com/zeyla/serenity/commit/8565fa2cb356cf8cbccfeb09828c9d136ad3d614
[c:8572943]: https://github.com/zeyla/serenity/commit/857294358d5f3029850dc79c174b831c0b0c161c
[c:86d8bdd]: https://github.com/zeyla/serenity/commit/86d8bddff3e3242186d0c2607b34771e5422ba5b
[c:917dd30]: https://github.com/zeyla/serenity/commit/917dd3071dc8a145b9c379cb3a8a84731c690340
[c:9b0c053]: https://github.com/zeyla/serenity/commit/9b0c053725e04c60eb7ddcfeb847be4189b3dbf6
[c:b3aa441]: https://github.com/zeyla/serenity/commit/b3aa441c2d61ba324396deaf70f2c5818fd3f528
[c:c98cae4]: https://github.com/zeyla/serenity/commit/c98cae4e838147eaa077bbc68ffebf8834ff7b6b
[c:cf40386]: https://github.com/zeyla/serenity/commit/cf403867400110f446720fc20fad6781cf8c6b13
[c:d7621aa]: https://github.com/zeyla/serenity/commit/d7621aa4dfb2a3dea22e7848eb97e2b4cc1ade14

[c:005437f]: https://github.com/zeyla/serenity/commit/005437f56869e846ff677b6516605def0c4de7bc
[c:0186754]: https://github.com/zeyla/serenity/commit/01867549709ef73ee09ed442e1d5ea938fd7f74d
[c:0240717]: https://github.com/zeyla/serenity/commit/02407175e463b2b75295364d6b0e182fe34966ed
[c:03b6d78]: https://github.com/zeyla/serenity/commit/03b6d78885b3a59ffa781ded3682c2dd24e65aa7
[c:05162aa]: https://github.com/zeyla/serenity/commit/05162aa18aa737c05fbc13917fed1c8c218064d5
[c:051d23d]: https://github.com/zeyla/serenity/commit/051d23d60d4898d331d046861035165bf2e6cd23
[c:069df4f]: https://github.com/zeyla/serenity/commit/069df4f85d8c462df58c1fce00595462f2825337
[c:078947e]: https://github.com/zeyla/serenity/commit/078947edc2b7036b2a0b49afc3cc54b12a39af18
[c:0810ab7]: https://github.com/zeyla/serenity/commit/0810ab7a6aa37ca684b10c22dde8f0e03d3f8ea2
[c:092f288]: https://github.com/zeyla/serenity/commit/092f288fdd22ae39b019e61a6f12420b6ca3b67c
[c:0d6965f]: https://github.com/zeyla/serenity/commit/0d6965f647396c84b2570e92b63244c3afaea863
[c:106a4d5]: https://github.com/zeyla/serenity/commit/106a4d5f8ff22a829a9486ce88fa8326184828fa
[c:125c1b8]: https://github.com/zeyla/serenity/commit/125c1b8feff65ed86136ca0c3b75cdfa073aefc3
[c:14fd41b]: https://github.com/zeyla/serenity/commit/14fd41b0d62ab441b6600028792641d813f09cd8
[c:16a5828]: https://github.com/zeyla/serenity/commit/16a5828394c21baf799366136f5d48e20447a49e
[c:192ac8a]: https://github.com/zeyla/serenity/commit/192ac8aec0afb33055352ed6e6838c506cbbbf8c
[c:1a08904]: https://github.com/zeyla/serenity/commit/1a089048138e85607bd298ebc07e30f57fb4ac53
[c:1ab8b31]: https://github.com/zeyla/serenity/commit/1ab8b31a19c6782b867b518c01bad9fbbdd06241
[c:1fad3dd]: https://github.com/zeyla/serenity/commit/1fad3dd60a0a9a0959f6e7e55896bef151bf3e9d
[c:25d4931]: https://github.com/zeyla/serenity/commit/25d49316133e2a8b7c4b26d3b6a44efdf5ad8834
[c:25e91da]: https://github.com/zeyla/serenity/commit/25e91dabd2380bd8fd98acbb7cb220dd90d238bd
[c:266411c]: https://github.com/zeyla/serenity/commit/266411cd6fc9ee96310da52c68264f303bcf5938
[c:26919cf]: https://github.com/zeyla/serenity/commit/26919cf9aad1d7bc5f0f8042b4caf6bfcddbd7d8
[c:29ee627]: https://github.com/zeyla/serenity/commit/29ee627207e0c2a0d3f5310ac00d90b232d910c0
[c:2b053ea]: https://github.com/zeyla/serenity/commit/2b053ea007d6ca9cc820cb910597e8b5dad89d70
[c:2fb12e2]: https://github.com/zeyla/serenity/commit/2fb12e2b3782fff211a41cb27cd316afc4320a7b
[c:3017f6d]: https://github.com/zeyla/serenity/commit/3017f6dbc02e6189c69491993e828e2a7595cbed
[c:32de2cb]: https://github.com/zeyla/serenity/commit/32de2cb941e8d4fdffde7b8b82599fcd78ab4c2f
[c:3582691]: https://github.com/zeyla/serenity/commit/35826915a174c7f3e5d82bbc320d3238ae308d8c
[c:3c2716b]: https://github.com/zeyla/serenity/commit/3c2716bbaeb71eca8cb2c7fca0dfd0b00cd34ba5
[c:3db42c9]: https://github.com/zeyla/serenity/commit/3db42c96c98fdd6d332347767cb1c276858da98b
[c:3e0b103]: https://github.com/zeyla/serenity/commit/3e0b1032d80a1847558a752e8316d97f9ae58f04
[c:40031d9]: https://github.com/zeyla/serenity/commit/40031d9ec55b1a4dd6e350a7566ea230751a54ed
[c:420f9bd]: https://github.com/zeyla/serenity/commit/420f9bdaa5a5022ff1d769f1d44a689a6fea12a4
[c:421c709]: https://github.com/zeyla/serenity/commit/421c709bbd706d4f04453baacf0ec6a88759f8cd
[c:428cbb9]: https://github.com/zeyla/serenity/commit/428cbb94de239e87d3258891591e1464cb9d2e06
[c:4532e4a]: https://github.com/zeyla/serenity/commit/4532e4a1e87d7b4f09446b1f10db178931eb314a
[c:45d72ef]: https://github.com/zeyla/serenity/commit/45d72eff173d87b1353d8b5d001775cc49129dab
[c:47ea8f7]: https://github.com/zeyla/serenity/commit/47ea8f79b4e980e38fb337b2f3cefc5c7d92fb33
[c:485ad29]: https://github.com/zeyla/serenity/commit/485ad299fec218ed3fd354f7207ce6160d803b06
[c:4be6b9d]: https://github.com/zeyla/serenity/commit/4be6b9d5008ff8bb3d1fdddff5647a6bb307513c
[c:4d4e9dc]: https://github.com/zeyla/serenity/commit/4d4e9dcf4b559423dd5b169ecef46efe6a0d1fca
[c:4e360cf]: https://github.com/zeyla/serenity/commit/4e360cf86a74051e2d4f98758c65ae29b97b7b8b
[c:4efe1d1]: https://github.com/zeyla/serenity/commit/4efe1d1271515e9ffecd318e368f127becfe273f
[c:4f2e47f]: https://github.com/zeyla/serenity/commit/4f2e47f399a10b281a1638fd7fcd3b945154d52c
[c:50d7f00]: https://github.com/zeyla/serenity/commit/50d7f00f1b01f4e0d9c86dbdd05a4d4f7b41f8b1
[c:511ec87]: https://github.com/zeyla/serenity/commit/511ec87280e8ddec6589f48fec8260bf2e598bdb
[c:52b8e29]: https://github.com/zeyla/serenity/commit/52b8e29193801aa254ac7ab105331fb6b0e8eec1
[c:561b0e3]: https://github.com/zeyla/serenity/commit/561b0e38b4cda6661425f76c8d707d58d0f12d09
[c:562ce49]: https://github.com/zeyla/serenity/commit/562ce49698a39d5da68d3ac58a3d8cf401aa9e42
[c:5a96724]: https://github.com/zeyla/serenity/commit/5a967241efabd49116a6d6d5a6eeb95d3281d93b
[c:5e5f161]: https://github.com/zeyla/serenity/commit/5e5f161f83b48367bc65d83f8d3cb7f4b1b61f0a
[c:5fd3509]: https://github.com/zeyla/serenity/commit/5fd3509c8cfe25370ca4fa66a8468bd2a9679ef5
[c:60c33db]: https://github.com/zeyla/serenity/commit/60c33db56bb3754bb0d2196d5f48fee63adf7730
[c:619a91d]: https://github.com/zeyla/serenity/commit/619a91de7a2d3e882cbcb8d8566ffeee3bc8192f
[c:64bfc54]: https://github.com/zeyla/serenity/commit/64bfc5471808cff59c9b4b5eef80a756f13ff5be
[c:6572580]: https://github.com/zeyla/serenity/commit/657258040376be45a8be0ef0e3bd762a23babb0a
[c:68c5be8]: https://github.com/zeyla/serenity/commit/68c5be8b6beec57618abea4d8b5bcca34489746e
[c:6a101c4]: https://github.com/zeyla/serenity/commit/6a101c4a409ae3abe4038f96dcd51f0788d4c0e4
[c:6c43fed]: https://github.com/zeyla/serenity/commit/6c43fed3702be3fdc1eafed26a2f6335acd71843
[c:6d6063f]: https://github.com/zeyla/serenity/commit/6d6063fc8334a4422465d30e938a045fd7a09d17
[c:6f147e1]: https://github.com/zeyla/serenity/commit/6f147e182b60817dd16e7868326b8cfa1f89ac88
[c:710fa02]: https://github.com/zeyla/serenity/commit/710fa02405d8d740c4ee952822d856af0e845aa8
[c:78e7b1b]: https://github.com/zeyla/serenity/commit/78e7b1b0624edce9bf69ff6d1d652f9cdfd3f841
[c:7c4b052]: https://github.com/zeyla/serenity/commit/7c4b052d7b5a50f234721249bd0221f037e48ea9
[c:7e8da0c]: https://github.com/zeyla/serenity/commit/7e8da0c6574ed051de5a9d51001ead0779dfb1de
[c:7e913b6]: https://github.com/zeyla/serenity/commit/7e913b6185468d2dd3740c711d418a300584b5bb
[c:824f8cb]: https://github.com/zeyla/serenity/commit/824f8cb63271ac3907a9c8223b08b7ee6ff0d746
[c:870a2a5]: https://github.com/zeyla/serenity/commit/870a2a5f821c9b0624cad03d873d04a8aad47082
[c:878684f]: https://github.com/zeyla/serenity/commit/878684f61fb48a25e117ed32548f78869cb027fc
[c:88765d0]: https://github.com/zeyla/serenity/commit/88765d0a978001ff88a1ee12798a725b7f5a90e9
[c:8a33329]: https://github.com/zeyla/serenity/commit/8a333290365f1304ad84a8e8f17c0d60728241c2
[c:8bf77fa]: https://github.com/zeyla/serenity/commit/8bf77fa431308451411670f20896e36f920997c5
[c:8cc2300]: https://github.com/zeyla/serenity/commit/8cc2300f7f2992ae858808033137440ee7e22cd8
[c:8d51ead]: https://github.com/zeyla/serenity/commit/8d51ead1747296eac5f2880332ae3e6de048ea4f
[c:8e1435f]: https://github.com/zeyla/serenity/commit/8e1435f29a2051f3f481131399fedf5528cb96e4
[c:8e29694]: https://github.com/zeyla/serenity/commit/8e296940b7e40879dcfbb185282b906804ba7e3d
[c:8e3b4d6]: https://github.com/zeyla/serenity/commit/8e3b4d601ffb78909db859640482f7e0bb10131f
[c:8f37f78]: https://github.com/zeyla/serenity/commit/8f37f78af0b9fda4cb0c4bf41e4c047958aa5a40
[c:924c447]: https://github.com/zeyla/serenity/commit/924c44759a79a8467cbf9f616a6aaa54c0e746cb
[c:948b27c]: https://github.com/zeyla/serenity/commit/948b27ce74e8dce458d427d8159f2a821d4d7cec
[c:97e84fe]: https://github.com/zeyla/serenity/commit/97e84fe136c5649ca3529c11790d9988dfe3bb92
[c:9900b20]: https://github.com/zeyla/serenity/commit/9900b20bf5cd4036cd8d8ba28bdcd852f2c89d2f
[c:9ccf388]: https://github.com/zeyla/serenity/commit/9ccf388e89b0cedddbf76a2236254d4d6ba0dd02
[c:9f02720]: https://github.com/zeyla/serenity/commit/9f02720d53ea117b1f6505a061b42fd7044219b9
[c:aa307b1]: https://github.com/zeyla/serenity/commit/aa307b160a263fb4d091d4aed06076b6c7f744b6
[c:aace5fd]: https://github.com/zeyla/serenity/commit/aace5fdb7f6eb71c143414c491005e378e299221
[c:ab67c1d]: https://github.com/zeyla/serenity/commit/ab67c1dd60b5f49541815b2527e8a3cb7712e182
[c:af1061b]: https://github.com/zeyla/serenity/commit/af1061b5e82ed1bf4e71ff3146cb98bc6cbb678c
[c:b249c82]: https://github.com/zeyla/serenity/commit/b249c8212ecd37cf3d52188fcc56f45268b3400e
[c:b602805]: https://github.com/zeyla/serenity/commit/b602805501df003d1925c2f0d0c80c2bac6d32a2
[c:b6af867]: https://github.com/zeyla/serenity/commit/b6af86779701110f7f21da26ae8712f4daf4ee3b
[c:bc3491c]: https://github.com/zeyla/serenity/commit/bc3491cf3a70a02ce5725e66887746567ae4660c
[c:bd05bda]: https://github.com/zeyla/serenity/commit/bd05bdad1765ad2038dcc4650e1ad4da8a2e020c
[c:bd9fcf7]: https://github.com/zeyla/serenity/commit/bd9fcf73a7912c900d194a0bebae586fb0d96d79
[c:bfdb57c]: https://github.com/zeyla/serenity/commit/bfdb57cdf35721f4953d436a819745ac5d44295e
[c:c2cf691]: https://github.com/zeyla/serenity/commit/c2cf6910b6a77c40d543d8950fca45c0d49b6073
[c:c68d4d5]: https://github.com/zeyla/serenity/commit/c68d4d5230e60ab48c5620f3d7daff666ded4a11
[c:c7b8ab8]: https://github.com/zeyla/serenity/commit/c7b8ab89c33c72b36b789dcc0648c164df523b1b
[c:ca0f113]: https://github.com/zeyla/serenity/commit/ca0f113324c1ed64a8646c42ed742dd8021fbccd
[c:caf69d6]: https://github.com/zeyla/serenity/commit/caf69d66893c2688f0856cc33f03702071d1314a
[c:cb18d42]: https://github.com/zeyla/serenity/commit/cb18d4207c3b9cf942bd561e76ae4059dd50979d
[c:cdedf36]: https://github.com/zeyla/serenity/commit/cdedf36330aa6da9e59d296164090f54b651b874
[c:d35d719]: https://github.com/zeyla/serenity/commit/d35d719518a48b1cf51c7ecb5ed9c717893784dc
[c:d8027d7]: https://github.com/zeyla/serenity/commit/d8027d7a3b9521565faa829f865c6248b3ba26c5
[c:d925f92]: https://github.com/zeyla/serenity/commit/d925f926c0f9f5b8010a998570441258417fc89a
[c:dbcb351]: https://github.com/zeyla/serenity/commit/dbcb3514f20409b3c4c4054fe51aaa2bd1792b96
[c:dbd6727]: https://github.com/zeyla/serenity/commit/dbd672783ef6f647664d3b1aa97957af9321d55c
[c:dc3a4df]: https://github.com/zeyla/serenity/commit/dc3a4dfafb1ee096b56c78d2506743e4012323f7
[c:deee38d]: https://github.com/zeyla/serenity/commit/deee38d87d71a918b6d8270dbfaffeb0a7234508
[c:e1912c2]: https://github.com/zeyla/serenity/commit/e1912c22fc806f97d9eb9025aa2432e785003f3b
[c:e1a8fe3]: https://github.com/zeyla/serenity/commit/e1a8fe3e9f619fbb94dd54993c8f5d25fd5dc375
[c:e2053dd]: https://github.com/zeyla/serenity/commit/e2053dd53f7c85175901ee57f7c028ba369487a9
[c:e218ce0]: https://github.com/zeyla/serenity/commit/e218ce0ec78b7b480e9a83628378dc9670e2cf4a
[c:e5889ed]: https://github.com/zeyla/serenity/commit/e5889ed1a62ddcb6bc11364800cd813329eb3ece
[c:e72e25c]: https://github.com/zeyla/serenity/commit/e72e25cf8b0160a3ec0de0be98dd8f1467d3b505
[c:e7a5ba3]: https://github.com/zeyla/serenity/commit/e7a5ba3e6c7e914c952408828f0cc71e15acea61
[c:ea1eba8]: https://github.com/zeyla/serenity/commit/ea1eba89087825e526e54fffdb27642fe72f9602
[c:ea432af]: https://github.com/zeyla/serenity/commit/ea432af97a87b8a3d673a1f40fe06cde4d84e146#diff-2e7fe478bd2e14b5b3306d2c679e4b5a
[c:eb47559]: https://github.com/zeyla/serenity/commit/eb47559fa00c13c8fdc8f40a8fe3d06690c0570c
[c:ebc4e51]: https://github.com/zeyla/serenity/commit/ebc4e51fe3b1e5bc61dc99da25a22d2e2277ffc6
[c:eee857a]: https://github.com/zeyla/serenity/commit/eee857a855831851599e5196750b27b26151eb16
[c:f05efce]: https://github.com/zeyla/serenity/commit/f05efce7af0cb7020e7da08c7ca58fa6f786d4ef
[c:f16af97]: https://github.com/zeyla/serenity/commit/f16af97707edfc36f52fa836791d07512e5d41ef
[c:f5a97d4]: https://github.com/zeyla/serenity/commit/f5a97d43b467130fd97af8c8a0dd1bbf0e7f5326
[c:f830f31]: https://github.com/zeyla/serenity/commit/f830f31f046b39124877a65fa1a95f789d125809
[c:fb2a1a9]: https://github.com/zeyla/serenity/commit/fb2a1a9262b481af62f9c0025a0f180626d19241
[c:fbc1ac7]: https://github.com/zeyla/serenity/commit/fbc1ac740e769e624637c490b6a959ed86ec3839
[c:fc9eba3]: https://github.com/zeyla/serenity/commit/fc9eba3d6d6a600f7d45a6f4e5918aae1191819d
[c:fd47b86]: https://github.com/zeyla/serenity/commit/fd47b865f3c32f5bbfce65162023898a6ecd29a1
[c:fd89d09]: https://github.com/zeyla/serenity/commit/fd89d09d3397eba21d1b454d3b6155ba9c3a829e
[c:fdbfbe0]: https://github.com/zeyla/serenity/commit/fdbfbe098c9d59000c234a0893496751744fd31e
[c:fdfb184]: https://github.com/zeyla/serenity/commit/fdfb1846083165629feca81b5169ceaf331289c5
[c:f6fcf32]: https://github.com/zeyla/serenity/commit/f6fcf32e7f62dfc207ac2f9f293f804446ea3423
[c:fdfd5bc]: https://github.com/zeyla/serenity/commit/fdfd5bcf708b6633b564fc58fb86935536310314

[c:00fb61b]: https://github.com/zeyla/serenity/commit/00fb61b5f306aebde767cc21a498a8ca0742d0be
[c:0102706]: https://github.com/zeyla/serenity/commit/0102706321a00cfb39b356bdf2cf8d523b93a8ec
[c:01f6872]: https://github.com/zeyla/serenity/commit/01f687204dd9d5564ec4bdc860f11bfd5e01454f
[c:04cfaa9]: https://github.com/zeyla/serenity/commit/04cfaa9a69dc1638e9cd1904a9b8e94c1a97f832
[c:060b06e]: https://github.com/zeyla/serenity/commit/060b06ec62b1f4e4cc2c11b877fd988b7dcfe627
[c:063a52f]: https://github.com/zeyla/serenity/commit/063a52f8c028c7432ee556380d2bd5c652d75d22
[c:0708ccf]: https://github.com/zeyla/serenity/commit/0708ccf85bac347e59053133a2b8b6f2eabe99ba
[c:096b0f5]: https://github.com/zeyla/serenity/commit/096b0f57aae04a5e0ea28414f5016eeafc5b9e0a
[c:0a2f5ab]: https://github.com/zeyla/serenity/commit/0a2f5ab525022fbf0055649f2262573fb07cf18c
[c:0b95db9]: https://github.com/zeyla/serenity/commit/0b95db916580b8b7eb8bf7e81e6051f849a9c0c8
[c:0b9bf91]: https://github.com/zeyla/serenity/commit/0b9bf91f62eef85a4eca703902077f4c04b3b6d1
[c:0c9ec37]: https://github.com/zeyla/serenity/commit/0c9ec377aa7281fb3d4bc390c896b426660a5387
[c:0d218e0]: https://github.com/zeyla/serenity/commit/0d218e02e043c043d7274c7169607b11c9897a5a
[c:0ec4dfb]: https://github.com/zeyla/serenity/commit/0ec4dfb785459c0d04c295f84a1c33e71c016eba
[c:0f41ffc]: https://github.com/zeyla/serenity/commit/0f41ffc811827fdd45e4e631884909e33fa8769e
[c:11a02db]: https://github.com/zeyla/serenity/commit/11a02db8e70c18a152bad9de6491817efc1d2f54
[c:13de5c2]: https://github.com/zeyla/serenity/commit/13de5c2e50410c3a68435dc774537b490bb7115c
[c:143337a]: https://github.com/zeyla/serenity/commit/143337ae717773f59562d67f85d0e9c44063a45b
[c:147cf01]: https://github.com/zeyla/serenity/commit/147cf01d4f13e3ee15eb03705ab2b7a006851cdd
[c:1561f9e]: https://github.com/zeyla/serenity/commit/1561f9e36384a215d2b866a752996f80d36a3ede
[c:1594961]: https://github.com/zeyla/serenity/commit/159496188b2c841a65829328cddafef620c517af
[c:16bd765]: https://github.com/zeyla/serenity/commit/16bd765112befd5d81818cab7b97ac59bd8a1b75
[c:16d1b3c]: https://github.com/zeyla/serenity/commit/16d1b3cad3982accd805f64ef93e51d921b3da55
[c:1700a4a]: https://github.com/zeyla/serenity/commit/1700a4a9090789d485c190c2a6ccd2c48986f5dd
[c:175d3a3]: https://github.com/zeyla/serenity/commit/175d3a3ef585f6fede959183138d507886192a4e
[c:2416813]: https://github.com/zeyla/serenity/commit/24168137ff7b1ec44d3ecdec0f516455fd3785a7
[c:268f356]: https://github.com/zeyla/serenity/commit/268f356a25f27175a5d72458fff92b0f770d0a5a
[c:2844ae1]: https://github.com/zeyla/serenity/commit/2844ae158f3d8297b17a584ff9a75f1f51116f48
[c:2845681]: https://github.com/zeyla/serenity/commit/28456813f6f05e9bdaf08e8cad641df1e3dfaff7
[c:2a743ce]: https://github.com/zeyla/serenity/commit/2a743cedaf08f7eb532e3c4b795cfc5f85bc96af
[c:2afab7c]: https://github.com/zeyla/serenity/commit/2afab7c6eb828e491721e15f11a76ae36e34796d
[c:2b237e7]: https://github.com/zeyla/serenity/commit/2b237e7de221beab9c516d6de29f83188ef63840
[c:2cb607d]: https://github.com/zeyla/serenity/commit/2cb607d72a39aa7ab3df866b23de4c9798e69a0f
[c:2d09152]: https://github.com/zeyla/serenity/commit/2d091528287b7f5dfd678e9bc77c25bf53b0f420
[c:2eaa415]: https://github.com/zeyla/serenity/commit/2eaa4159955260e7c9ade66803d69865f1f76018
[c:302d771]: https://github.com/zeyla/serenity/commit/302d771182308f907423ed73be9b736f268737fe
[c:3062981]: https://github.com/zeyla/serenity/commit/3062981bfc1412e93450b30fa9405e555624ce1e
[c:31aae7d]: https://github.com/zeyla/serenity/commit/31aae7d12763f94a7a08ea9fd0102921e8402241
[c:31becb1]: https://github.com/zeyla/serenity/commit/31becb16f184cd7d396b383ad4abed8095451fcb
[c:32e07e4]: https://github.com/zeyla/serenity/commit/32e07e4ac822d5cc1118f0db0fc92b549c1aaf81
[c:3348178]: https://github.com/zeyla/serenity/commit/3348178f151d8e1d7aa0432984a2dd23fa7b9e89
[c:345e140]: https://github.com/zeyla/serenity/commit/345e1401142d21a0fdabb2accd1f33e3a07c02c8
[c:38a484d]: https://github.com/zeyla/serenity/commit/38a484d0fec91e290bc1633fc871131f9decd0ca
[c:38db32e]: https://github.com/zeyla/serenity/commit/38db32e2cbb9dc8504e0dfbc2366b17596836da0
[c:39a28d3]: https://github.com/zeyla/serenity/commit/39a28d3bf5d7005c3549a09542d27c08660f49cb
[c:3c7c575]: https://github.com/zeyla/serenity/commit/3c7c575d988f4dc793678880560aee48456f4526
[c:3ca7ad9]: https://github.com/zeyla/serenity/commit/3ca7ad92507f056054d081485f437c08505bc7e5
[c:3f03f9a]: https://github.com/zeyla/serenity/commit/3f03f9adc97315bb61a5c71f52365306cb8e2d1a
[c:404a089]: https://github.com/zeyla/serenity/commit/404a089af267c5d5c33025a3d74826e02b6f8ca1
[c:4229034]: https://github.com/zeyla/serenity/commit/42290348bc05c876b7e70c570a485160e594e098
[c:4267bdb]: https://github.com/zeyla/serenity/commit/4267bdbae05d5516774ca72fe92789651cfa7230
[c:43a5c5d]: https://github.com/zeyla/serenity/commit/43a5c5d7eb8bffb8c9ca450ab1bc377d602fb8c3
[c:46b79dd]: https://github.com/zeyla/serenity/commit/46b79ddb45d03bfbe0eb10a9d5e1c53c9a15f55b
[c:494cc50]: https://github.com/zeyla/serenity/commit/494cc50ff3dcf8553a5588fa868754d27c237055
[c:49a6841]: https://github.com/zeyla/serenity/commit/49a684134df32427e9502192122c4fb22ef1a735
[c:4a14b92]: https://github.com/zeyla/serenity/commit/4a14b92ff58173acb98c7e0a135b4989a87a7529
[c:4cf8338]: https://github.com/zeyla/serenity/commit/4cf8338e364b0feefef26ece6649077e87962ff3
[c:4de39da]: https://github.com/zeyla/serenity/commit/4de39da887248e374b4d824472a6422c7e46dacc
[c:4f5fbb5]: https://github.com/zeyla/serenity/commit/4f5fbb54ae930dd56aa9a53878cf1b5e123de038
[c:51c15d0]: https://github.com/zeyla/serenity/commit/51c15d088054dd42c66fee10deed1431df931ec9
[c:543b604]: https://github.com/zeyla/serenity/commit/543b60421d1c6acd77e02cdd11c7dd2157399821
[c:55ccaca]: https://github.com/zeyla/serenity/commit/55ccaca57051b3fbd47cf7fa288014d9c36f6952
[c:57c060f]: https://github.com/zeyla/serenity/commit/57c060fa2fccfbb3b3d4b2d18aad2faa5929deb3
[c:585af23]: https://github.com/zeyla/serenity/commit/585af231028e46788d689f94e14e110c072a578e
[c:5918d01]: https://github.com/zeyla/serenity/commit/5918d01ed69541e43aed0e62ee6eadbf5ebb20d2
[c:5b275fc]: https://github.com/zeyla/serenity/commit/5b275fc425d4ef1c1a9eaa9d915db1f873f9c11d
[c:5bf6c2d]: https://github.com/zeyla/serenity/commit/5bf6c2d2cf0491951eddb10ab2641d02d0e730a1
[c:5c40e85]: https://github.com/zeyla/serenity/commit/5c40e85001b9b2620a76fcc57d8f0cddfb6f9b34
[c:5ee5fef]: https://github.com/zeyla/serenity/commit/5ee5feff615565b6f661ee3598fe19bb98bd6a88
[c:5fe6a39]: https://github.com/zeyla/serenity/commit/5fe6a3956d39e9b5caef19df78e8b392898b6908
[c:601704a]: https://github.com/zeyla/serenity/commit/601704acb94601a134ae43e795474afe8574b2ae
[c:626ffb2]: https://github.com/zeyla/serenity/commit/626ffb25af35f5b91a76fdccf6788382a1c39455
[c:62ed564]: https://github.com/zeyla/serenity/commit/62ed564e5f67f3e25d2307fbbf950d0489a28de8
[c:6355288]: https://github.com/zeyla/serenity/commit/635528875c59d34f0d7b2f2b0a3bd61d762f0e9c
[c:6502ded]: https://github.com/zeyla/serenity/commit/6502dedfcced471aaf17b7d459da827a1867807a
[c:651c618]: https://github.com/zeyla/serenity/commit/651c618f17cb92d3ea9bbd1d5f5c92a015ff64e0
[c:6579b1f]: https://github.com/zeyla/serenity/commit/6579b1fb0409410f303a4df5e7246c507a80f27b
[c:66546d3]: https://github.com/zeyla/serenity/commit/66546d36749f6c78a4957a616524fab734d5c972
[c:6853daf]: https://github.com/zeyla/serenity/commit/6853daf4d04719a9a8a081151bd85336e160a752
[c:68c473d]: https://github.com/zeyla/serenity/commit/68c473dd17a2098f97808b3d1f2a200621f67c9d
[c:69ec62a]: https://github.com/zeyla/serenity/commit/69ec62a42bcb143cdde056ad8ffce81922e88317
[c:6a887b2]: https://github.com/zeyla/serenity/commit/6a887b25f2712d70c65fc85b5cfbd8b6d4b41260
[c:6b0b9b2]: https://github.com/zeyla/serenity/commit/6b0b9b2491fa895bd7dd8e065f067470ea08639d
[c:6e11a10]: https://github.com/zeyla/serenity/commit/6e11a103f7a6a4ab43b1aa511aad9e04b1fd8c5a
[c:6f33a35]: https://github.com/zeyla/serenity/commit/6f33a35c4f85a06c45c4ce9e118db203c4951475
[c:70bf22a]: https://github.com/zeyla/serenity/commit/70bf22a00cd19651a0d994cc43e8d8c4bd8947fc
[c:70d4e75]: https://github.com/zeyla/serenity/commit/70d4e7538cefc21dd0e06d5451888b82f53acf38
[c:71f3dbb]: https://github.com/zeyla/serenity/commit/71f3dbb650f4b0d6434630137ae9eea502a1ebef
[c:760a47a]: https://github.com/zeyla/serenity/commit/760a47aa4d34160f44048e775afeb30f08891c99
[c:76f9095]: https://github.com/zeyla/serenity/commit/76f9095c012a8769c7bd27aca6540b7018574c28
[c:77b5b48]: https://github.com/zeyla/serenity/commit/77b5b480d67e747908f8f4fb9f910bab23b761b5
[c:7914274]: https://github.com/zeyla/serenity/commit/79142745cb571ba2d4284fd1dcbe53c14a0ed623
[c:7990381]: https://github.com/zeyla/serenity/commit/799038187d903a75d60f0c98d013ae87fb665d02
[c:7b45f16]: https://github.com/zeyla/serenity/commit/7b45f16f063a47dc8a302dce5b016cf43a3edcc1
[c:7b4b154]: https://github.com/zeyla/serenity/commit/7b4b1544603a70dd634b51593ea5173b4515889a
[c:7dbae6b]: https://github.com/zeyla/serenity/commit/7dbae6b5261b8f53200090c9eb1bf39a7498f07d
[c:7e254c5]: https://github.com/zeyla/serenity/commit/7e254c5c6098bb1a47bac26c9895098a46cdc53f
[c:7f04179]: https://github.com/zeyla/serenity/commit/7f041791aa95e38a0cacd2ab64f0423524c60052
[c:7fc49d8]: https://github.com/zeyla/serenity/commit/7fc49d8dd9e253b066ab1b82446d0344f800e2d7
[c:c832009]: https://github.com/zeyla/serenity/commit/c832009eae235881815186f740b716e0b7e63951
[c:8360f32]: https://github.com/zeyla/serenity/commit/8360f329eae1751a8a413a6f6838486f3a0bba01
[c:83b1d96]: https://github.com/zeyla/serenity/commit/83b1d967f4cc2040f94d67dd987302347f227d6a
[c:83b29d5]: https://github.com/zeyla/serenity/commit/83b29d5f839cd2ea6cd150aa7b8ccbbc677c1fad
[c:858bbf2]: https://github.com/zeyla/serenity/commit/858bbf298d08ada3ae6c5b24105bf751bc938d5e
[c:86a4e00]: https://github.com/zeyla/serenity/commit/86a4e008ca7acf23d920e344463df801a774d5ce
[c:86cd00f]: https://github.com/zeyla/serenity/commit/86cd00f20d6f218e524deed040d3c209f0361a86
[c:8b504ad]: https://github.com/zeyla/serenity/commit/8b504ad7f6e10fecb27583a949262eb61cfd266d
[c:8c04d31]: https://github.com/zeyla/serenity/commit/8c04d318e273e9bcb3af6ddd820ad067048e95c6
[c:8c0aeac]: https://github.com/zeyla/serenity/commit/8c0aeacadb93d3b56fb98beb882eaef1f79cd652
[c:8c5ee70]: https://github.com/zeyla/serenity/commit/8c5ee70b28b42ac92f899932ab2ddafeb9c6f913
[c:8e2c052]: https://github.com/zeyla/serenity/commit/8e2c052a55e5e08c6e7ed643b399f1a7f69a2b25
[c:8effc91]: https://github.com/zeyla/serenity/commit/8effc918cc3d269b0d4cf34ef4f2053cecad2606
[c:8f24aa3]: https://github.com/zeyla/serenity/commit/8f24aa391f6b8a9103a9c105138c7610288acb05
[c:8f88c6b]: https://github.com/zeyla/serenity/commit/8f88c6b0613199492ebca8cd9f2bf4dd5c97add7
[c:8f8a059]: https://github.com/zeyla/serenity/commit/8f8a05996c5b47ec9401aabb517d96ed2af5c36b
[c:9114963]: https://github.com/zeyla/serenity/commit/9114963daf708cfaeaf54d8c788206ccfbae5df8
[c:921f7f4]: https://github.com/zeyla/serenity/commit/921f7f42d87e7c727b5a87802d7738f8081b600a
[c:92309b2]: https://github.com/zeyla/serenity/commit/92309b2fb8ffd96292fd2edaa7c223a2ba774a56
[c:9268f9c]: https://github.com/zeyla/serenity/commit/9268f9c10ef47ffeaeb3d5040e65b1093e04b866
[c:92f4ec2]: https://github.com/zeyla/serenity/commit/92f4ec204d10a8d60af9ce3cc7433be8117a711d
[c:933ee89]: https://github.com/zeyla/serenity/commit/933ee8914509e52c5119ced9f5d9d8f9644cfa63
[c:93416cd]: https://github.com/zeyla/serenity/commit/93416cdebff12a3f85e694c8cb28350a5c14c50f
[c:9392f61]: https://github.com/zeyla/serenity/commit/9392f61f8857b6ab2a04781c2d9c92a582a1577b
[c:93f3c60]: https://github.com/zeyla/serenity/commit/93f3c60b23cb8ffd16666bdc01b3502ca7ba5f47
[c:9969be6]: https://github.com/zeyla/serenity/commit/9969be60cf320797c37b317da24d9a08fd5eafa5
[c:97f9bd1]: https://github.com/zeyla/serenity/commit/97f9bd10c16eb24d54a0ab00c52f19eb51a88675
[c:990e611]: https://github.com/zeyla/serenity/commit/990e611a56f37f64fbce74fbc487c7dcc4aa4e28
[c:9aa357f]: https://github.com/zeyla/serenity/commit/9aa357f0c8f504b53b49824cc20561c8501d2dda
[c:9c04a19]: https://github.com/zeyla/serenity/commit/9c04a19015cf579d343d81a7fa50e6f4b18b4a5b
[c:9c1ed6c]: https://github.com/zeyla/serenity/commit/9c1ed6ca933f81bc0254d9d52159b9190b50a3ea
[c:9dae9e6]: https://github.com/zeyla/serenity/commit/9dae9e67b992cea4c18f1c685f5185abd9428887
[c:9ec05e7]: https://github.com/zeyla/serenity/commit/9ec05e701bdbadad39847f0dcc18d5156ecdde02
[c:9ef5522]: https://github.com/zeyla/serenity/commit/9ef55224757dff6dec8576bd1ad11db24a10891e
[c:a0bb306]: https://github.com/zeyla/serenity/commit/a0bb30686c1a9431aef23c2e8594791f64035194
[c:a2cbeb6]: https://github.com/zeyla/serenity/commit/a2cbeb6ece9ef56e2082b6eabbabe5fe536ab3ec
[c:a39647d]: https://github.com/zeyla/serenity/commit/a39647d3ba1650a4dd4c92bd40001959828000c7
[c:a8acd61]: https://github.com/zeyla/serenity/commit/a8acd6138741a6e5268141ac4ce902561931d353
[c:ab778f8]: https://github.com/zeyla/serenity/commit/ab778f8a9cf47c4e27fe688a61effb0caa4f8a6e
[c:ab7f113]: https://github.com/zeyla/serenity/commit/ab7f113a9e3acd000dbf69b7c4bd8d2d766b39f1
[c:abd22d2]: https://github.com/zeyla/serenity/commit/abd22d289599530cbd1bc9cf1b739420f0d22372
[c:ada07fa]: https://github.com/zeyla/serenity/commit/ada07fae09f3521f44d81613f26839d69c1fc7ef
[c:ae352ea]: https://github.com/zeyla/serenity/commit/ae352ea3df86eb2d853d5b1af048a95409aafc38
[c:ae395f4]: https://github.com/zeyla/serenity/commit/ae395f44361a9a9b488b31d6ac0cb54e0ee9e7a1
[c:aea9885]: https://github.com/zeyla/serenity/commit/aea98851e86c0f36be231c0a3b763f769c76e061
[c:afc571f]: https://github.com/zeyla/serenity/commit/afc571fd67c294cc10682db5c579d10645aec437
[c:b001234]: https://github.com/zeyla/serenity/commit/b0012349cca2a5c7c62bb6d2c99106d245b6c55a
[c:b468cbf]: https://github.com/zeyla/serenity/commit/b468cbffa0db341987d1dc397582b3edd3944d09
[c:b4bd771]: https://github.com/zeyla/serenity/commit/b4bd7714a155381cc16ece51acb0c4dc6cde96a2
[c:b7cbf75]: https://github.com/zeyla/serenity/commit/b7cbf75103939b0b7834c808050b19ba4fbc4b17
[c:b96f85c]: https://github.com/zeyla/serenity/commit/b96f85c224b9c0478b7f1b5c5b76761e23ff7edf
[c:bad9ac3]: https://github.com/zeyla/serenity/commit/bad9ac3d28bb0417dedcdddf10cf764c08d1d6ae
[c:bb97211]: https://github.com/zeyla/serenity/commit/bb97211b2b107943dd6fabb7a0a344d4fe236780
[c:bcb70e8]: https://github.com/zeyla/serenity/commit/bcb70e85384a16b2440788a73241f507aaeba4dc
[c:bceb049]: https://github.com/zeyla/serenity/commit/bceb049bb2b804dac975567bb7eac6afcfc28574
[c:c00f349]: https://github.com/zeyla/serenity/commit/c00f3490f2fb0c045c2da72d850f70da8e2cdb95
[c:c01f238]: https://github.com/zeyla/serenity/commit/c01f238a34ad846f8732c8bf97fbbd96fbf6a7ae
[c:c032fbe]: https://github.com/zeyla/serenity/commit/c032fbe7a5c65fb6824a5eb36daf327134b854cf
[c:c050c59]: https://github.com/zeyla/serenity/commit/c050c59da25b9093a75bda22baa81be3b267c688
[c:c2e8b69]: https://github.com/zeyla/serenity/commit/c2e8b69702cf81a1cf149c420aec999124f398e2
[c:c36841d]: https://github.com/zeyla/serenity/commit/c36841dd1c3f80141251ba01130333f43ff363d7
[c:c74cc15]: https://github.com/zeyla/serenity/commit/c74cc15f8969c8db68119d07a4f273a0d3fc44f4
[c:c8536c1]: https://github.com/zeyla/serenity/commit/c8536c111117f26833fb1bceff734ac1abc55479
[c:c8c6b83]: https://github.com/zeyla/serenity/commit/c8c6b83ca685a3e503c853d4154a17761790954e
[c:cd914f5]: https://github.com/zeyla/serenity/commit/cd914f503c8f0ada7473b5b56e4ad7830370ea45
[c:d033909]: https://github.com/zeyla/serenity/commit/d03390968ec7a5e1e93dbcc508c3b8a5f44b792d
[c:d0b64cd]: https://github.com/zeyla/serenity/commit/d0b64cd64a18a6116267fa09a837d62c19cced42
[c:d144136]: https://github.com/zeyla/serenity/commit/d1441363364970b749d57b8a4863b284239488d1
[c:d3389be]: https://github.com/zeyla/serenity/commit/d3389be3042fd7977350a08152d177ac6cdcd37f
[c:d367a70]: https://github.com/zeyla/serenity/commit/d367a704985bbb127f410770125c160f90561937
[c:d37461b]: https://github.com/zeyla/serenity/commit/d37461b5b705e0cdf802925c59113898a71676df
[c:d4fc8b6]: https://github.com/zeyla/serenity/commit/d4fc8b6188627ae8d553cf282b1371e3de7b01f9
[c:d58c544]: https://github.com/zeyla/serenity/commit/d58c54425a18bbbdc8e66e8eebfb8191bad06901
[c:d9118c0]: https://github.com/zeyla/serenity/commit/d9118c081742d6654dc0a4f60228a7a212ca436e
[c:daf92ed]: https://github.com/zeyla/serenity/commit/daf92eda815b8f539f6d759ab48cf7a70513915f
[c:db0f025]: https://github.com/zeyla/serenity/commit/db0f025d154e4b6212dd9340c1b789b3c711a24a
[c:dc73d1a]: https://github.com/zeyla/serenity/commit/dc73d1a4bad07b453a9d60a6c8f8c187a7e42450
[c:e033ff3]: https://github.com/zeyla/serenity/commit/e033ff33b94e024fe5f55a8c93c65c3e885f821b
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
[c:e92b667]: https://github.com/zeyla/serenity/commit/e92b667058138ffd01587e28e9d8551cd59df160
[c:e9aae9c]: https://github.com/zeyla/serenity/commit/e9aae9c043b206b15bd5429126ded62259d6731b
[c:eb09f2d]: https://github.com/zeyla/serenity/commit/eb09f2d3389b135978e0671a0e7e4ed299014f94
[c:eb43b9c]: https://github.com/zeyla/serenity/commit/eb43b9c4a4e43a8e097ea71fdc7584c8108b52a3
[c:ec9b1c7]: https://github.com/zeyla/serenity/commit/ec9b1c79abeb2a4eff9f013ba8f0e430979dbc56
[c:ef6eba3]: https://github.com/zeyla/serenity/commit/ef6eba37636a487c0d6f3b93b8e76c94f28abbab
[c:f00e165]: https://github.com/zeyla/serenity/commit/f00e1654e8549ec6582c6f3a8fc4af6aadd56015
[c:f0d1157]: https://github.com/zeyla/serenity/commit/f0d1157212397ae377e11d4205abfebc849ba9d8
[c:f3f74ce]: https://github.com/zeyla/serenity/commit/f3f74ce43f8429c4c9e38ab7b905fb5a24432fd4
[c:f53124e]: https://github.com/zeyla/serenity/commit/f53124ec952124f5b742f204cdf7e1dc00a168ab
[c:f57a187]: https://github.com/zeyla/serenity/commit/f57a187d564bdcd77f568e77a102d6d261832ee0
[c:f69512b]: https://github.com/zeyla/serenity/commit/f69512beaa157775accd4392295dba112adcf1df
[c:f695174]: https://github.com/zeyla/serenity/commit/f695174287e3999cbcbabc691a86302fa8269900
[c:f6b27eb]: https://github.com/zeyla/serenity/commit/f6b27eb39c042e6779edc2d5d4b6e6c27d133eaf
[c:f847638]: https://github.com/zeyla/serenity/commit/f847638859423ffaaecfdb77ee5348a607ad3293
[c:f894cfd]: https://github.com/zeyla/serenity/commit/f894cfdc43a708f457273e1afb57ed1c6e8ebc58
[c:f96b6cc]: https://github.com/zeyla/serenity/commit/f96b6cc5e1e0383fd2de826c8ffd95565d5ca4fb
[c:fafa363]: https://github.com/zeyla/serenity/commit/fafa3637e760f0c72ae5793127bc2f70dcf2d0e2
[c:fb07751]: https://github.com/zeyla/serenity/commit/fb07751cfc1efb657cba7005c38ed5ec6b192b4f
[c:fb4d411]: https://github.com/zeyla/serenity/commit/fb4d411054fa44928b4fa052b19de19fce69d7cf
[c:ff4437a]: https://github.com/zeyla/serenity/commit/ff4437addb01e5c6c3ad8c5b1830db0d0a86396b

[c:f47a0c8]: https://github.com/zeyla/serenity/commit/f47a0c831efe5842ca38cb1067de361ae42f6edc
[c:d50b129]: https://github.com/zeyla/serenity/commit/d50b12931404946e219d3ff0878f0632445ef35f
[c:41f26b3]: https://github.com/zeyla/serenity/commit/41f26b3757c7a5fba1f09f34e3192e2fd9702a4a
[c:f9e5e76]: https://github.com/zeyla/serenity/commit/f9e5e76585a1f6317dadb67e440765b0070ca131
[c:9428787]: https://github.com/zeyla/serenity/commit/9428787abb6126ba05bfef96cd2b8d2a217fdf5d
[c:a58de97]: https://github.com/zeyla/serenity/commit/a58de97e6089aa98f04d2cdc7312ed38a9f72b22
[c:fbd6258]: https://github.com/zeyla/serenity/commit/fbd625839e6a2e01b16e6c3814cb9b9f31dc7caa
[c:292ceda]: https://github.com/zeyla/serenity/commit/292cedaa3462f7532efda98722354afa8e213b6a
[c:d3015a0ff]: https://github.com/zeyla/serenity/commit/d3015a0ff0c0c87888437f991945453b92296875
[c:585ac6e]: https://github.com/zeyla/serenity/commit/585ac6e6ca792facf29063776c83262fa849161b
[c:3616585]: https://github.com/zeyla/serenity/commit/361658510f3e2eb9aefbe66232b9b1f1a1ebb80f
[c:e694766]: https://github.com/zeyla/serenity/commit/e694766bb6c93d5f6a75ad9871cfdefbd0309a17
[c:e02d5fb]: https://github.com/zeyla/serenity/commit/e02d5fb8171b11214e1502c6754fef1972bbf1b9
[c:b7cdf15]: https://github.com/zeyla/serenity/commit/b7cdf1542cb9199c61c0b17bdd381d4f117f635e
[c:c7aa27d]: https://github.com/zeyla/serenity/commit/c7aa27dbb64e64d70c7f13725c79017c4bba1c95
[c:2219bb3]: https://github.com/zeyla/serenity/commit/2219bb37a80c4c2b4ff5a24d72b82737eb241195
[c:74ec713]: https://github.com/zeyla/serenity/commit/74ec713825b2b4c55382fb76fa57bd967e66b3aa
[c:5829c67]: https://github.com/zeyla/serenity/commit/5829c673c13655b86d317ab65d204067a2b1a7a4
[c:ce4f8c2]: https://github.com/zeyla/serenity/commit/ce4f8c2ac8dd2c472ab537a60bf92579d078073b
[c:fcc4e2c]: https://github.com/zeyla/serenity/commit/fcc4e2ce2e523248ed33c9f4853d3485cbc9b6e6
[c:23ff6f]: https://github.com/zeyla/serenity/commit/23ff6f21019bc94f8dc32355fa34691b881bfb69
[c:e57b510]: https://github.com/zeyla/serenity/commit/e57b510edd640abb243664337a1c163924313612
