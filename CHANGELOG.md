# Change Log
All notable changes to this project will be documented in this file.
This project mostly adheres to [Semantic Versioning][semver].

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

### Fixes

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

[c:00fb61b]: https://github.com/zeyla/serenity/commit/00fb61b5f306aebde767cc21a498a8ca0742d0be
[c:0708ccf]: https://github.com/zeyla/serenity/commit/0708ccf85bac347e59053133a2b8b6f2eabe99ba
[c:096b0f5]: https://github.com/zeyla/serenity/commit/096b0f57aae04a5e0ea28414f5016eeafc5b9e0a
[c:0a2f5ab]: https://github.com/zeyla/serenity/commit/0a2f5ab525022fbf0055649f2262573fb07cf18c
[c:147cf01]: https://github.com/zeyla/serenity/commit/147cf01d4f13e3ee15eb03705ab2b7a006851cdd
[c:1594961]: https://github.com/zeyla/serenity/commit/159496188b2c841a65829328cddafef620c517af
[c:2416813]: https://github.com/zeyla/serenity/commit/24168137ff7b1ec44d3ecdec0f516455fd3785a7
[c:2a743ce]: https://github.com/zeyla/serenity/commit/2a743cedaf08f7eb532e3c4b795cfc5f85bc96af
[c:2b237e7]: https://github.com/zeyla/serenity/commit/2b237e7de221beab9c516d6de29f83188ef63840
[c:2cb607d]: https://github.com/zeyla/serenity/commit/2cb607d72a39aa7ab3df866b23de4c9798e69a0f
[c:3348178]: https://github.com/zeyla/serenity/commit/3348178f151d8e1d7aa0432984a2dd23fa7b9e89
[c:345e140]: https://github.com/zeyla/serenity/commit/345e1401142d21a0fdabb2accd1f33e3a07c02c8
[c:38a484d]: https://github.com/zeyla/serenity/commit/38a484d0fec91e290bc1633fc871131f9decd0ca
[c:38db32e]: https://github.com/zeyla/serenity/commit/38db32e2cbb9dc8504e0dfbc2366b17596836da0
[c:3ca7ad9]: https://github.com/zeyla/serenity/commit/3ca7ad92507f056054d081485f437c08505bc7e5
[c:4229034]: https://github.com/zeyla/serenity/commit/42290348bc05c876b7e70c570a485160e594e098
[c:5918d01]: https://github.com/zeyla/serenity/commit/5918d01ed69541e43aed0e62ee6eadbf5ebb20d2
[c:5b275fc]: https://github.com/zeyla/serenity/commit/5b275fc425d4ef1c1a9eaa9d915db1f873f9c11d
[c:5c40e85]: https://github.com/zeyla/serenity/commit/5c40e85001b9b2620a76fcc57d8f0cddfb6f9b34
[c:5fe6a39]: https://github.com/zeyla/serenity/commit/5fe6a3956d39e9b5caef19df78e8b392898b6908
[c:6a887b2]: https://github.com/zeyla/serenity/commit/6a887b25f2712d70c65fc85b5cfbd8b6d4b41260
[c:6355288]: https://github.com/zeyla/serenity/commit/635528875c59d34f0d7b2f2b0a3bd61d762f0e9c
[c:651c618]: https://github.com/zeyla/serenity/commit/651c618f17cb92d3ea9bbd1d5f5c92a015ff64e0
[c:66546d3]: https://github.com/zeyla/serenity/commit/66546d36749f6c78a4957a616524fab734d5c972
[c:68c473d]: https://github.com/zeyla/serenity/commit/68c473dd17a2098f97808b3d1f2a200621f67c9d
[c:69ec62a]: https://github.com/zeyla/serenity/commit/69ec62a42bcb143cdde056ad8ffce81922e88317
[c:70bf22a]: https://github.com/zeyla/serenity/commit/70bf22a00cd19651a0d994cc43e8d8c4bd8947fc
[c:760a47a]: https://github.com/zeyla/serenity/commit/760a47aa4d34160f44048e775afeb30f08891c99
[c:76f9095]: https://github.com/zeyla/serenity/commit/76f9095c012a8769c7bd27aca6540b7018574c28
[c:7b45f16]: https://github.com/zeyla/serenity/commit/7b45f16f063a47dc8a302dce5b016cf43a3edcc1
[c:83b29d5]: https://github.com/zeyla/serenity/commit/83b29d5f839cd2ea6cd150aa7b8ccbbc677c1fad
[c:86cd00f]: https://github.com/zeyla/serenity/commit/86cd00f20d6f218e524deed040d3c209f0361a86
[c:8c5ee70]: https://github.com/zeyla/serenity/commit/8c5ee70b28b42ac92f899932ab2ddafeb9c6f913
[c:8e2c052]: https://github.com/zeyla/serenity/commit/8e2c052a55e5e08c6e7ed643b399f1a7f69a2b25
[c:92309b2]: https://github.com/zeyla/serenity/commit/92309b2fb8ffd96292fd2edaa7c223a2ba774a56
[c:933ee89]: https://github.com/zeyla/serenity/commit/933ee8914509e52c5119ced9f5d9d8f9644cfa63
[c:93f3c60]: https://github.com/zeyla/serenity/commit/93f3c60b23cb8ffd16666bdc01b3502ca7ba5f47
[c:a2cbeb6]: https://github.com/zeyla/serenity/commit/a2cbeb6ece9ef56e2082b6eabbabe5fe536ab3ec
[c:a8acd61]: https://github.com/zeyla/serenity/commit/a8acd6138741a6e5268141ac4ce902561931d353
[c:ab778f8]: https://github.com/zeyla/serenity/commit/ab778f8a9cf47c4e27fe688a61effb0caa4f8a6e
[c:ada07fa]: https://github.com/zeyla/serenity/commit/ada07fae09f3521f44d81613f26839d69c1fc7ef
[c:abd22d2]: https://github.com/zeyla/serenity/commit/abd22d289599530cbd1bc9cf1b739420f0d22372
[c:b001234]: https://github.com/zeyla/serenity/commit/b0012349cca2a5c7c62bb6d2c99106d245b6c55a
[c:bcb70e8]: https://github.com/zeyla/serenity/commit/bcb70e85384a16b2440788a73241f507aaeba4dc
[c:c01f238]: https://github.com/zeyla/serenity/commit/c01f238a34ad846f8732c8bf97fbbd96fbf6a7ae
[c:c050c59]: https://github.com/zeyla/serenity/commit/c050c59da25b9093a75bda22baa81be3b267c688
[c:c2e8b69]: https://github.com/zeyla/serenity/commit/c2e8b69702cf81a1cf149c420aec999124f398e2
[c:c36841d]: https://github.com/zeyla/serenity/commit/c36841dd1c3f80141251ba01130333f43ff363d7
[c:d3389be]: https://github.com/zeyla/serenity/commit/d3389be3042fd7977350a08152d177ac6cdcd37f
[c:d58c544]: https://github.com/zeyla/serenity/commit/d58c54425a18bbbdc8e66e8eebfb8191bad06901
[c:e5a83dd]: https://github.com/zeyla/serenity/commit/e5a83dd1873e5af2df18835d960fe19286c70f1e
[c:e85e901]: https://github.com/zeyla/serenity/commit/e85e901062e8b9ea717ec6c6253c9c7a300448d3
[c:e8a9086]: https://github.com/zeyla/serenity/commit/e8a90860d1e451e21d3bf728178957fe54cf106d
[c:e9aae9c]: https://github.com/zeyla/serenity/commit/e9aae9c043b206b15bd5429126ded62259d6731b
[c:f3f74ce]: https://github.com/zeyla/serenity/commit/f3f74ce43f8429c4c9e38ab7b905fb5a24432fd4
[c:f57a187]: https://github.com/zeyla/serenity/commit/f57a187d564bdcd77f568e77a102d6d261832ee0
[c:f894cfd]: https://github.com/zeyla/serenity/commit/f894cfdc43a708f457273e1afb57ed1c6e8ebc58
[c:f96b6cc]: https://github.com/zeyla/serenity/commit/f96b6cc5e1e0383fd2de826c8ffd95565d5ca4fb
[c:fafa363]: https://github.com/zeyla/serenity/commit/fafa3637e760f0c72ae5793127bc2f70dcf2d0e2
[c:fb07751]: https://github.com/zeyla/serenity/commit/fb07751cfc1efb657cba7005c38ed5ec6b192b4f
[c:fb4d411]: https://github.com/zeyla/serenity/commit/fb4d411054fa44928b4fa052b19de19fce69d7cf

## [0.1.3] - 2016-12-14

This is a hotfix for applying a PR and fixing a major bug in the plain help
command.

### Added

- Blocking individual users and guilds in commands
- Disabling commands
- Configuring "owners" of the bot, which command checks won't apply to

### Fixes

- The plain help command now properly sends a message when requesting
  information about a command
- Groups are now on their own lines in the plain help command

## [0.1.2] - 2016-12-14

v0.1.2 focuses on revamping the framework, adding a large amount of
configuration and overall features. v0.1.3 will be focused on performance
optimizations and code cleanup.

Thanks to the following for contributions this release:

- [@acdenisSK]
- [@fwrs]

v0.1.2 can now be retrieved from the [crates.io listing].

### Added

`UserId::find()` has been added to find the User from cache ([@fwrs]).

`utils::{parse_channel, parse_emoji, parse_role, parse_username}` added to parse
each item from a string; useful for command arguments.

The `CreateEmbed` builder now implements `From<Embed>`, so you can use an
  already-built embed from something like a `Message`.

`Context::get_current_user` to retrieve the current user from the cache; prefer
using `CACHE.read().unwrap().user` to avoid a clone.

Implemented `Emoji::url()` and `EmojiIdentifier::url()` to generate URLs for the
emoji's image.



The framework has been revamped:
Structs can now be used as framework command arguments. FromStr is implemented
for:
- `User`
- `UserId`
- `Role`
- `RoleId`
- `EmojiIdentifier`
- `ChannelId`
- `Channel`
You can implement this yourself for your own structs ([@fwrs]).

The framework global configuration now has additional configuration:

- account type to listen to or ignore (selfbot, ignore bots, or listen to
  everyone)
- dynamic prefix per context
- multiple prefixes
- error messages for various errors (not enough arguments, command cooldown,
  check fails, lack of permissions, etc.)

Commands can now be built with a large amount of configuration via a
`CreateCommand` builder; see example 06 on how to use this. The configuration
for commands includes:

- checks (whether the command should be executed)
- cooldown (ratelimit) bucket
- description (used for the help command)
- usage listing (used for the help command)
- argument quote parsing (parsing `a b c` vs. `a "b c"`)
- required minimum/maximum argument counts
- permissions required to use the command
- whether to display the command in the help message
- whether the command can only be used in DMs or Guilds

Additionally, groups can now be created via the `CreateGroup` builder. It allows
setting the group name (e.g. `"music"`), adding commands to the group, and
setting the group's prefix.

Two default help commands are provided for you if you don't wish to make your
own: one that creates an embed and one that is text-only.


Thanks to [@fwrs] for most work of the work on the framework.

See [example 06][v0.1.2:example 06] for examples on most of this.


### Fixes

- MessageBuilder channel/role/user methods now properly mention the given
  item ([@fwrs])
- No-cache+method compiles have been fixed
- `rest::edit_profile` no longer updated the internal token for bot users; this
  is a preliminary fix, as the endpoint will soon not return `"Bot "` in the
  token key for bot users

### Changed

- `model::Mention` has been removed, in favour of simply
  `model_name::mention()` (BC break) ([@fwrs])
- Ids now mention where possible on format; use Id.0 to format the inner
  snowflake directly (BC break) ([@fwrs])
- All internal `try!()`s have been converted to use rustc 1.13's `?`
  ([@acdenisSK])
- `CreateEmbedImage::{height, width}` and
  `CreateEmbedThumbnail::{height, width}` have been deprecated, as these do
  not do anything and there seems to not be any plans on Discord's side to make
  them do anything. These will be removed in v0.1.3 and the builders themselves
  will be replaced with methods on `CreateEmbed` (BC break)

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

### Added

- The following colours to the Colour struct:
  - "Kerbal" ([@indiv0])
  - "Blurple" ([@GetRektByMe])
  - "Blitz Blue" ([@iCrawl])
  - "Fabled Pink" ([@Flat])
  - "Fooyoo" ([@SunDwarf])
  - "Rosewater" ([@fwrs])
- `Message::guild_id` as a quick method for retrieving the Id of a message's guild
- `CurrentUser::guilds()` to get the current user's guilds. Meant for use with
  selfbots
- `CurrentUser::edit()` to edit the current user's profile settings
- `User::distinct` to format a string with the `username#discriminator`
  combination ([@fwrs])
- `Member::colour` to retrieve the member's colour ([@fwrs])
- Roles can now be directly compared (`role1 < role2`) for hierarchy
- Documentation:
  - `EditMember` and `EditProfile` ([@Kiseii])
  - Documentation for 19 model definitions ([@fwrs])
  - Context permission requirements
- A custom shared state (not the Cache) can now be accessed and mutated across
  commands/contexts, through the use of `Context.data`'s ShareMap. See
  [example 06][v0.1.1:example 06] for an example

### Fixes

- `rest::start_integration_sync`/`Context::start_integration_sync` now properly
  work ([@abalabahaha])
- Role positions can now be negative; fixes issues where a guild's @everyone
  role (and other roles) are negative
- `Context::move_member`'s signature is now correct

### Changed

- `Colour::dark_green` is now sorted alphabetically ([@khazhyk])
- The `command!` macro now publicly exports functions. This allows commands
  created via this macro to be separated into different modules or crates
- `rest::get_guilds` now supports pagination of guilds, as the output is now
  limited to 100

### Backwards Compatibility Breaks

None

Clause: backwards compatibility breaks are ones that are _not_ due to a change
in Discord's API.


## [0.1.0] - 2016-11-30

Initial commit.

[0.1.4]: https://github.com/zeyla/serenity/compare/v0.1.3...v0.1.4
[0.1.3]: https://github.com/zeyla/serenity/compare/v0.1.2...v0.1.3
[0.1.2]: https://github.com/zeyla/serenity/compare/v0.1.1...v0.1.2
[0.1.1]: https://github.com/zeyla/serenity/compare/v0.1.0...v0.1.1
[0.1.0]: https://github.com/zeyla/serenity/tree/403d65d5e98bdfa9f0c018610000c4a0b0c7d8d5
[crates.io listing]: https://crates.io/crates/serenity
[semver]: http://semver.org
[v0.1.2:example 06]: https://github.com/zeyla/serenity/tree/5a3665a9951c023e3e6ea688844558b7f8b5fb6e/examples/06_command_framework
[v0.1.1:example 06]: https://github.com/zeyla/serenity/tree/ccb9d16e5dbe965e5a604e1cb402cd3bc7de0df5/examples/06_command_framework

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
[@SunDwarf]: https://github.com/SunDwarf
