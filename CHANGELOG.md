# Change Log
All notable changes to this project will be documented in this file.
This project mostly adheres to [Semantic Versioning][semver].

## [Unreleased]

### Added

### Changed

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
  [example 06] for an example

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

## Backwards Compatibility Breaks

None

[example 06]: https://github.com/zeyla/serenity.rs/tree/ccb9d16e5dbe965e5a604e1cb402cd3bc7de0df5/examples/06_command_framework

Clause: backwards compatibility breaks are ones that are _not_ due to a change
in Discord's API.


## [0.1.0] - 2016-11-30

Initial commit.

[Unreleased]: https://github.com/zeyla/serenity.rs/compare/v0.1.1...master
[0.1.1]: https://github.com/zeyla/serenity.rs/compare/v0.1.0...v0.1.1
[0.1.0]: https://github.com/zeyla/serenity.rs/tree/403d65d5e98bdfa9f0c018610000c4a0b0c7d8d5
[crates.io listing]: https://crates.io/crates/serenity
[semver]: http://semver.org

[@abalabahaha]: https://github.com/abalabahaha
[@Flat]: https://github.com/Flat
[@fwrs]: https://github.com/fwrs
[@GetRektByMe]: https://github.com/GetRektByMe
[@iCrawl]: https://github.com/iCrawl
[@indiv0]: https://github.com/indiv0
[@khazhyk]: https://github.com/khazhyk
[@SunDwarf]: https://github.com/SunDwarf
