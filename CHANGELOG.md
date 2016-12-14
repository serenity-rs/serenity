# Change Log
All notable changes to this project will be documented in this file.
This project mostly adheres to [Semantic Versioning][semver].

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

[0.1.2]: https://github.com/zeyla/serenity.rs/compare/v0.1.1...v0.1.2
[0.1.1]: https://github.com/zeyla/serenity.rs/compare/v0.1.0...v0.1.1
[0.1.0]: https://github.com/zeyla/serenity.rs/tree/403d65d5e98bdfa9f0c018610000c4a0b0c7d8d5
[crates.io listing]: https://crates.io/crates/serenity
[semver]: http://semver.org
[v0.1.2:example 06]: https://github.com/zeyla/serenity.rs/tree/5a3665a9951c023e3e6ea688844558b7f8b5fb6e/examples/06_command_framework
[v0.1.1:example 06]: https://github.com/zeyla/serenity.rs/tree/ccb9d16e5dbe965e5a604e1cb402cd3bc7de0df5/examples/06_command_framework

[@abalabahaha]: https://github.com/abalabahaha
[@acdenisSK]: https://github.com/acdenisSK
[@Flat]: https://github.com/Flat
[@fwrs]: https://github.com/fwrs
[@GetRektByMe]: https://github.com/GetRektByMe
[@iCrawl]: https://github.com/iCrawl
[@indiv0]: https://github.com/indiv0
[@khazhyk]: https://github.com/khazhyk
[@SunDwarf]: https://github.com/SunDwarf
