# Change Log

All notable changes to this project will be documented in this file.
This project mostly adheres to [Semantic Versioning][semver].

## [0.8.2] - 2020-04-11

This is a release for a critical bugfix.

In an attempt to fix one thing, other things broke. Any time Serenity made a request to a POST or PUT endpoint (such as creating reactions or banning users), Discord would return HTTP 400. This release corrects that.

Thanks to the following for their contributions:

- [@Lakelezz]

### Fixed

- [http] Always send the `Content-Length` header ([@Lakelezz]) [c:f5dd8bf]

## [0.8.1] - 2020-04-02

This is the last release for the 0.8.x series. 0.9.x will contain async/await support!

It is mostly comprised of bugfixes and quality of life changes.

### Note

While this is a minor release, a change that is technically breaking has been included. Some `MessageBuilder` methods were forgotten about when changing builders to mutably borrow in 0.8.0. This release fixes that.


Thanks to the following for their contributions:

- [@acdenisSK]
- [@bikeshedder]
- [@Elinvynia]
- [@KamranMackey]
- [@Lakelezz]
- [@MaxOhn]
- [@natsukagami]
- [@nitsuga5124]
- [@Noituri]
- [@NovusTheory]
- [@TitusEntertainment]
- [@vivianhellyer]

### Added

- [model] Add `guild_id` into the `Reaction` model struct ([@Elinvynia]) [c:820d50e]

- [model] Add missing `guild_id` to various events ([@NovusTheory]) [c:3ca41fd]

- [model] Add support for client statuses ([@KamranMackey]) [c:5f9a27a]

- [http] make error module public ([@vivianhellyer]) [c:d2b19a2]

- [builder] Reexport `Timestamp` from the builder's module. ([@acdenisSK]) [c:3a313c8]

- [model] Implement various `kick_with_reason()` methods ([@nitsuga5124]) [c:5b0e9f3]

### Changed

- [framework] If finding the argument fails, return to the original position ([@acdenisSK]) [c:e005ef1]

- [framework] Display groups without commands in help. ([@Lakelezz]) [c:d6b0038]

- [model] Make `Member::distinct()` show the discriminator as a 4-digit number ([@natsukagami]) [c:a23acc7]

- [http] Deserialize from slices in `fire` ([@Noituri]) [c:a44f16d]

- [utils] &mut self instead of mut self for MessageBuilder methods ([@MaxOhn]) [c:91f10dd]

- [utils] Implement `Color` type alias to `Colour` ([@Elinvynia]) [c:c3d5264]

- [http] Only set the content type header if there's a body ([@acdenisSK]) [c:d851fea]

- [framework] Store command names in lowercase when case-insensitivity is on ([@acdenisSK]) [c:8bba7b0]

### Fixed

- [misc] Fix release dates ([@bikeshedder]) [c:f27c7c1]

- [framework/docs] Fill in the missing attribute options ([@acdenisSK]) [c:683ff27]

- [http/docs] Fix link to the `fire` method ([@acdenisSK]) [c:1361b33]

- [framework] Fix strikethrough options refusing to accept `name = value` syntax ([@acdenisSK]) [c:581eb2f]

- [framework/docs] Fix a broken link in docs ([@Elinvynia]) [c:48c4b59]

- [misc] Fix a typo in the message builder example ([@TitusEntertainment]) [c:f2d0ad5]

- [framework] Fix `check_in_help` being unaccounted for ([@acdenisSK]) [c:a692bcd]

- [http] Support image URLs in `add_file` ([@Elinvynia]) [c:61bcfbc]

- [http] Add impls for borrowed `Arc`s to `CacheHttp` ([@acdenisSK]) [c:4b67d8e]

### Removed

- [all] Remove usage of deprecated `std::error::Error` functions ([@acdenisSK]) [c:ec306ee]

## [0.8.0] - 2020-01-12

The next major release of Serenity, coupled with improvements, bugfixes and some breaking changes.

### `group!` is now `#[group]`

To stay consistent with the `#[command]`, `#[help]` and `#[check]` macros,
the function-like `group!` procedural macro has also joined in to
the squad of the attribute procedural macros.

Thanks to the following for their contributions:

- [@acdenisSK]
- [@CarlGroth]
- [@Erk-]
- [@ikkerens]
- [@Lakelezz]
- [@LeSeulArtichaut]
- [@Mendess2526]
- [@nickelc]
- [@TheUnitedStatesOfAmerica]
- [@zeyla]

### Added

- [framework] Allow multiple examples in commands ([@Mendess2526]) [c:08d894e]
- [gateway] Add support for receiving custom statuses ([@Erk-]) [c:f897a8d]

### Changed

- [http] Rename 'raw' module to 'client' ([@zeyla]) [c:8326dc7]
- [http] Rework ratelimit structure ([@zeyla]) [c:5dbe078]
- [cache/http] Use `CacheRwLock` in `CacheAndHttp` ([@Lakelezz]) [c:28a91c6]
- [framework] Apply case-insensitivity on prefixes ([@acdenisSK]) [c:b2c951d]
- [framework] Turn off the `owner_privilege` option ([@acdenisSK]) [c:d4b45f4]
- [model] Ensure nullable Discord API values are optional in audit log fields ([@ikkerens]) [c:3a3f9b3]
- [framework] Turn the function-like group macro to an attribute macro ([@acdenisSK]) [c:5b01290]
- [http] Utilise Discord's new precision and reset-after headers ([@acdenisSK]) [c:6916bfc]
- [http] Change `AttachmentType` to use `Cow`s instead ([@ikkerens]) [c:b5deb39]
- [framework] Format the list of attribute options with tables ([@acdenisSK]) [c:3c2f9ad]
- [framework] Impose thread-safety requirements on the event handlers ([@acdenisSK]) [c:3a449ee]
- [framework] Deduplicate Client initialisation code ([@acdenisSK]) [c:e807288] [c:ab7f631]
- [client/gateway] Add an option to turn off guild subscriptions ([@acdenisSK]) [c:db5a09e]
- [framework] Interpret doc-comments as the description attribute ([@acdenisSK]) [c:cc2f918]
- [framework] Account for checks in the help command ([@acdenisSK]) [c:240d3e6]
- [framework] Add documentation to the `check` macro ([@acdenisSK]) [c:0b3ad00]
- [model] Mark the `Event` enum as untagged (serde) ([@CarlGroth]) [c:173f7fa]
- [model] Add new auditlog type enums ([@ikkerens]) [c:aed4b24]
- [framework] Abuse the compiler to do type checking for us ([@acdenisSK]) [c:d6699c2]
- [meta] Upgrade to reqwest v0.10.0 ([@nickelc]) [c:69f2dff]

### Fixed

- [http] Fix crash due to Bearer token not having the 'email' scope ([@LeSeulArtichaut]) [c:ae0acd0]
- [model] Fix `Guild::edit_role_position` example ([@LeSeulArtichaut]) [c:346a7fe]
- [utils] Fix compilation of the `utils` feature without the `model` feature ([@Erk-]) [c:487fa04]

### Removed

- [framework] Get rid of the `Arc` implementation for `Framework` ([@acdenisSK]) [c:05044b6]
- [client] Turn the function-like group macro to an attribute macro ([@acdenisSK]) [c:5b01290]
- [http] Remove april fools functions ([@TheUnitedStatesOfAmerica]) [c:caeeda1]

## [0.7.5] - 2020-01-13

An emergency release to fix build breakage due to violation of SemVer for the `command_attr` crate.

## [0.7.4] - 2019-12-13

Thanks to the following for their contributions:

- [@acdenisSK]
- [@LeSeulArtichaut]

### Added

- [framework] Enable pub restrictions ([@acdenisSK]) [c:e6ed1b5]
- [framework] Implement Default for CommandOptions and GroupOptions ([@LeSeulArtichaut]) [c:918273b]

### Fixed

- [framework] Fix regression of default option initialisation ([@acdenisSK]) [c:42937e9]


## [0.7.3] - 2019-11-19

Small release including fixes for Discord API changes. Please note with this version the minimum supported version of Rust is 1.37.

Thanks to the following for their contributions:

- [@acdenisSK]
- [@ikkerens]

### Fixed

- [framework] Properly `pub`licise the iterator ([@acdenisSK]) [c:1924946]
- [ci] Try to update repository information beforehand ([@acdenisSK]) [c:19b590a]
- [audit log] Cover all error cases for audit log deserialization ([@ikkerens]) [c:beb4d5a] [c:afc04e4]

## [0.7.2] - 2019-10-21

A tiny release for a fix to voice

Thanks to the following for their contributions:

- [@acdenisSK]
- [@MOZGIII]

### Fixed

- [voice] Fix `ClientVoiceManager::remove` to actually call `Manager::remove` ([@MOZGIII]) [c:2734e27]
- [voice] Use the correct ip for the UDP socket ([@acdenisSK]) [c:c4b1c60]

## [0.7.1] - 2019-9-29

## Departure of a lead developer

It seems Discord has a thing against library developers. [They disabled the account of a discord.js developer because they were allegedly "underage" (below 13 years old)](https://github.com/discordjs/discord.js/issues/3440). There were credit card transactions to defend their innocence, but Discord argued that they need a photo of their face to properly verify their age, a request the developer declined to comply. Consequently, they chose to no longer be on Discord.

Recently, they did the same thing to [@Lakelezz], a huge contributor to Serenity. However, this time they did not state their exact reason, simply saying "in violation of the ToS". Just like the JS developer, she decided to stop affiliating herself with the platform, if this is how it presents its "gratitude" towards her. But also, to cease development of the library.

And thus, on her behalf, I, the main lead developer [@acdenisSK], announce her retirement of the project.

Thanks to the following for their contributions:

- [@acdenisSK]
- [@Erk-]
- [@ikkerens]
- [@kyranet]
- [@Lakelezz]
- [@shnarazk]
- [@Zalaxx]

### Added

- [model] Add support for the `preferred_locale` field ([@Erk-]) [c:2d3e585]
- [meta] Add missing word `need`. ([@Lakelezz]) [c:65837f5]
- [model] Add new message fields ([@Erk-]) [c:e762ea9]
- [gateway/client] Implement WebSocket shutdown support ([@ikkerens]) [c:711882b]
- [utils] Add more formats and case insensitivity to `parse_invite` ([@ikkerens]) [c:0183714]
- [model] Add optional inviter field to Invite ([@ikkerens]) [c:21c95fd]

### Changed

- [meta] Set minimum Rust version to `1.37.0`.  ([@Lakelezz]) [c:de9e8a6]
- [meta] Update related project's hrefs ([@kyranet]) [c:445810f]

### Fixed

- [meta] Fix serenity version in the readme ([@Zalaxx]) [c:730c959]
- [framework] Fix incorrect label usage in plain help commands ([@acdenisSK]) [c:d427da4]
- [model]  Fix `has_role` returning an incorrect result if the member is not cached ([@ikkerens]) [c:96b49f9]

### Removed

- [meta] Remove the last mention of the global CACHE ([@shnarazk]) [c:ebdeb4e]

## [0.7.0] - 2019-8-29

An emergency release to fix a conflict in our [`ring`](https://github.com/briansmith/ring) dependency that prevents compilation if you pull in an older and newer version at the same time.

Thanks to the following for their contributions:
- [@Lakelezz]

### Changed

- [meta] Update all dependencies ([@Lakelezz]) [c:50d2a76]

## [0.6.4] - 2019-8-27

Thanks to the following for their contributions:
- [@Alch-Emi]
- [@AregevDev]
- [@acdenisSK]
- [@Erk-]
- [@Jerald]
- [@Lakelezz]
- [@leo-lb]
- [@Sreyas-Sreelal]

### Added

- [model] Add a method to create and iterable of `Member`s in a `Guild` ([@Alch-Emi]) [c:aa1070d]
- [utils] Add quoting functionality to `MessageBuilder`  ([@AregevDev]) [c:720d9ad]
- [model] Add support for new message types ([@Erk-]) [c:c45c1d4]
- [model] Add support for store channel ([@Erk-]) [c:8594c29]
- [model] Link to `ShardMessenger::chunk_guilds` in `Guild`'s `member` field ([@Alch-Emi]) [c:8e926f9]
- [framework]  Add group-related removal and non-consuming adding functions to `StandardFramework` ([@Jerald]) [c:3a4e2ed]
- [framework] Allow delimiters to be set on a per command basis ([@acdenisSK]) [c:6f7797e]
- [voice] Play a YouTube Search's first video ([@Sreyas-Sreelal]) [c:ccbba0a]
- [model] Add methods to get permissions of `Role`s in `GuildChannel`s ([@Lakelezz]) [c:09c1e01]
- [utils] Allow users to create their own messages ([@acdenisSK]) [c:e8da420]

### Changed

- [model] Fetch the guild id only if necessary ([@acdenisSK]) [c:85dd1a0]
- [meta] Move `webpki` and `webpki-roots` to the `rustls_backend`-feature ([@leo-lb]) [c:2439275]

### Fixed

- [model] Fix content of a message if there's an attachment ([@Erk-]) [c:6d06632]
- [meta/examples] Fix a typo in the examples ([@Sreyas-Sreelal]) [c:22f3d2a]
- [framework] Fix plain help suggestions ([@Lakelezz]) [c:ec687ad]

## [0.6.3] - 2019-7-24

Thanks to the following for their contributions:

- [@acdenisSK]
- [@Erk-]
- [@hyarsan]
- [@Lakelezz]
- [@Mendess2526]
- [@NieDzejkob]
- [@xacrimon]

### Added

- [cache] Implement `CacheHttp` for `Arc<Http>` ([@Lakelezz]) [c:1a209e8]

### Changed

- [framework] Update module-level example ([@acdenisSK]) [c:26192fa]
- [client/model/utils] Changed some `&str`-parametres to `impl AsRef<str>` ([@xacrimon]) [c:abd84c2]
- [framework] Try to always ignore bots and webhooks when configured by the user ([@acdenisSK]) [c:4cf4b21] [c:b7b3a85]
- [framework] Apply case-insensitivity to help if needed ([@acdenisSK]) [c:cd4ca1b]

### Fixed

- [voice] Fix mistake in `voice::Audio`'s documentation with `play_only` ([@Mendess2526]) [c:e6c5d41]
- [utils]  Make `MessageBuilder::push_spoiler_*` consistent with the other `push_` functions ([@hyarsan]) [c:d2df2b9]
- [misc.] Update to the actual minimum Rust version Serenity supports ([@acdenisSK]) [c:d280ed1]
- [misc.] Fix comment in the group prefixes example ([@NieDzejkob]) [c:81d5af1]:
- [framework] Fix `Reason`'s doc-link by using `enum` ([@Lakelezz]) [c:a8f0387]
- [client] Fix private channel deletions making serenity panic ([@Erk-]) [c:67f5e3d]
- [model] Fix `create_invite`'s doc-example ([@Lakelezz]) [c:45d44cb]
- [framework] Update `help_commands`'s module example ([@acdenisSK]) [c:8cdfd7c]
- [framework] Remove unnecessary ticks ([@Lakelezz]) [c:eddef7b]

## [0.6.2] - 2019-6-30

A small release to address a severe deserialization bug.

Thanks to the following for their contributions:
- [@benjaminrsherman]
- [@Lakelezz]
- [@zeyla]

### Changed

- [meta] Reduce versioning in examples to major.minor ([@Lakelezz]) [c:13595ff]
- [framework/command_attr] Escape tags and add newline in documentation ([@Lakelezz]) [c:b28716c]

### Fixed

- [framework/command_attr] Fix invalid documentation for the group macro ([@benjaminrsherman]) [c:17f1dc2]
- [model] Fix guild deserialisation regression ([@zeyla]) [c:e628614]

## [0.6.1] - 2019-6-29

Thanks to the following for their contributions:

- [@acdenisSK]
- [@hyarsan]
- [@Lakelezz]
- [@rsaihe]
- [@xacrimon]
- [@zeyla]

## Added

- [framework/command_attr] Add the option to override the display name of a group ([@acdenisSK]) [c:759a278]
- [framework] Add `remains`, an optional alternative to `rest` ([@hyarsan]) [c:3e15bb8]

## Changed

- [meta] Update the versions to be latest in the README. ([@xacrimon]) [c:335701e]
- [model] Change the generic of `members` to encompass the `Option` ([@zeyla]) [c:3a72058]
- [framework] Remove `set_remove` with hint to the `#[check]` ([@Lakelezz]) [c:1527838]

## Fixed

- [model] Revert `say` taking `self` to `&self` ([@zeyla]) [c:e5081db]
- [framework] Give the owner privilege if only both the group and its command give consent. ([@acdenisSK]) [c:030bb4d]
- [command_attr] Fix `command_attr` documentation using `#[sub]` instead of `#[sub_commands]` ([@rsaihe]) [c:7a0d169]

## [0.6.0] - 2019-6-21

🎉 It has finally come for the biggest release of Serenity yet! 🎉

Thanks to the following for their contributions:

- [@AregevDev]
- [@acdenisSK]
- [@andreasots]
- [@Celti]
- [@DarkKirb]
- [@Erk-]
- [@eatsfoobars]
- [@Flat]
- [@FelixMcFelix]
- [@hyarsan]
- [@icewind1991]
- [@Kroisse]
- [@Lakelezz]
- [@Mishio595]
- [@molenzwiebel]
- [@mattico]
- [@nycex]
- [@PvdBerg1998]
- [@Roughsketch]
- [@xacrimon]
- [@xSke]
- [@zeyla]


## Since rc-2.1:

### Added
- [framework/command_attr#docs] Add `#[bucket]` to the available attributes list. ([@acdenisSK]) [c:0daaac1]

### Changed

- [examples] Fix a typo of Serenity ([@Lakelezz]) [c:90b7829]
- [gateway] Remove tungstenite buffer limits ([@molenzwiebel]) [c:638b642]
- [framework/command_attr] Be more helpful when reporting errors on return types ([@acdenisSK]) [c:c8a8d4f]
- [model] Make all structs non-exhaustive ([@zeyla]) [c:dddd417]
- [http] No longer delay ratelimit sleeps by 500ms ([@acdenisSK]) [c:638bb1a]

### Fixed

- [framework/command_attr] Assign the new value to the correct colour ([@acdenisSK]) [c:d1addff]
- [model] Fix panic on deserialising `PartialGuild` with no Nitro boosters. ([@xSke]) [c:5e77718]

## rc-2

### Added

- [command_attr] Add some utility structs. ([@acdenisSK]) [c:9162929]
- [command_attr] Add docs for `lacking_ownership`. ([@acdenisSK]) [c:15e7fde]
- [example] Add new Example about Eventing and Timing. ([@Lakelezz]) [c:10b9cc2]
- [example] Add example of embedding a local image in an embed. ([@Erk-]) [c:709c9e4]
- [framework] Add back blocking guilds, channels, and users. ([@acdenisSK]) [c:33f8383]
- [framework] Output the `#[example]` text in help ([@Flat]) [c:7aea26c]
- [model] Add and use `AttachmentId`. ([@Lakelezz]) [c:c8a5f69]
- [model] Add a `channel_id_from_name`-method on `Guild`. ([@xacrimon]) [c:aae22a2]
- [model] Add `GuildChannel::members`. ([@Lakelezz]) [c:ddf7a3]
- [model] Add more guild fields from guild boosting. ([@AregevDev]) [c:4541935]
- [utils] Add `EmbedMessageBuilding`-trait. ([@zeyla]) [c:7c61f95]

### Fixed

- [builder] Return `&mut self` on `voice_channel`. ([@Lakelezz]) [c:0e55b73]
- [ci] Fix Azure Windows build. ([@Lakelezz]) [c:fc3a1f6]
- [client] Fix updates giving only new data. ([@zeyla]) [c:5f7231d]
- [clippy] Fix Clippy-lints. ([@Lakelezz]) [c:cd7d07e]
- [clippy] Implement suggestions from Clippy & remove Clippy arg max config. ([@Flat]) [c:6586830]
- [command_attr] Add missing `s`, `owner_only` became `owners_only`. ([@acdenisSK]) [c:3cf673e]
- [command_attr] Report errors from parsing group options, if any. ([@acdenisSK]) [c:8e01ff6]
- [doc] Small doc fixes for the command macro. ([@acdenisSK]) [c:186e914]
- [doc] Revise Guild's `voice_states` doc. ([@nycex]) [c:0a640a4]
- [example] Update the mentioned feature `methods` to `utils`. ([@Lakelezz]) [c:c970f44]
- [framework] Fix Help displaying Groups and their Commands. ([@Lakelezz]) [c:eca204a]
- [framework] Ensure to properly hide groups. ([@Lakelezz]) [c:5e66cd1]
- [framework] Add help for nested groups and their commands. ([@Lakelezz]) [c:6a37535]
- [framework] Get rid of a redundant feature gate. ([@acdenisSK]) [c:2ae3a48]
- [framework] Treat the actual name and aliases equally. ([@Lakelezz]) [c:82d97c2]
- [framework] Check if message author is in owners HashSet. ([@Flat]) [c:d91594b]
- [framework] Change `owners_privilege` to bypass all permission-checks. ([@Flat]) [c:98532da]
- [http] Fix setting role positions ([@icewind1991]) [c:c14ca32]
- [model] Fix lifetime issue with `send_message`. ([@acdenisSK]) [c:3902caf]
- [model] Fix no-default-features compilation. ([@zeyla]) [c:3de5378]
- [model] Fix `contains_case_insensitive` and `starts_with_case_insensitive`. ([@Flat]) [c:d27d391]
- [voice] Pipe youtube-dl to ffmpeg directly. ([@Flat]) [c:4793a84]

### Changed

- [builder] Use `ToString` on builder-arguments instead of `Display`. ([@acdenisSK]) [c:13fae29]
- [builder] Increase the capabilities for creating a channel. ([@acdenisSK]) [c:f2ff97a]
- [client] Improve `cached`'s name and documentation. ([@Lakelezz]) [c:7706475]
- [command_attr] Rectify command parsing. ([@acdenisSK]) [c:b1eff27]
- [command_attr] Use the function-name. ([@acdenisSK]) [c:05254c8]
- [command_attr] Change `only` to `only_in`. ([@acdenisSK]) [c:26b072f]
- [command_attr] Reinvent `group!` parsing. ([@acdenisSK]) [c:7f9c4e1]
- [command_attr] Stop appending `_HELP_COMMAND` to the generated instance from `#[help]` ([@acdenisSK]) [c:9783b35]
- [example] Update the framework example. ([@acdenisSK]) [c:0fcb43c]
- [example] Update to use shard manager. ([@zeyla]) [c:5375827]
- [framework] Take into regard prefixless groups ([@acdenisSK]) [c:ef15739]
- [framework] Ensure prefixes to be mandatory on help ([@Lakelezz]) [c:ab34f75]
- [model] Update `*Id::created_at()` to return a `DateTime<FixedOffset>` instead. ([@AregevDev]) [c:8d50840]
- [model] Replace `Context` as argument ([@Lakelezz]) [c:62e19a7] [c:58fa50c]
- [rustc] Set minimum Rust version to `1.35.0`. ([@Lakelezz]) [c:8c83fec]
- [voice] Add infinite retry arg to ytdl for rst packets. ([@Flat]) [c:86ec810]

### Removed

- [builder] Remove unused `build`-method. ([@Lakelezz]) [c:c6ae140]
- [client] Remove `quit` method. ([@zeyla]) [c:f7109ee]
- [utils] Remove `VecMap`. ([@acdenisSK]) [c:9450d4b]

## rc-1

### Added

- [model] Add missing fields of `current_application_info` ([@mattico]) [c:23bed41]
- [builder] Allow for channels to be (or not be) set as nsfw ([@acdenisSK]) [c:1bd5bbc]
- [framework] Bring back old parsing behaviour ([@acdenisSK]) [c:64e97c5]
- [http] `AsRef<Http>` Implementation for `Http` ([@Lakelezz]) [c:b425ceb]

### Fixed

- [misc.] Fix Doc-Links and update Changelog ([@Lakelezz]) [c:c63eaea]

### Changed

- [framework] Take into equation ignoring bots and webhooks for help ([@acdenisSK]) [c:b1559bc]
- [general] Increase minimum support Rust version ([@acdenisSK]) [c:61ac765]
- [general/framework] Shackle the minimum version of uwl to 0.3.2 ([@acdenisSK]) [c:decbc04]

## rc-0

## Added

- [builder/model] Permit sending files through the `CreateMessage` builder. ([@Roughsketch]) [c:5405ac2]
- [client] Add Rich Presence parsing support ([@zeyla]) [c:f7360e6]
- [model] Add Slow Mode Rate ([@Lakelezz]) [c:7512c19]
- [voice] Voice reconnection ([@FelixMcFelix]) [c:25cb595] [c:4026d77] [c:2f613c0] [c:0a58e85]
- [model] Add a position propagation method to Channel ([@Erk-]) [c:59b4c60]
- [misc.] Re-export `typemap::sharemap` ([@zeyla]) [c:d2233e2]
- [framework] Add new Check System ([@Lakelezz]) [c:2969561]
- [http/gateway] Rustls support ([@Erk-]) [c:faa773a]
- [model] Add news channel ([@Lakelezz]) [c:1074b28]
- [client] Add EventHandler for raw Events ([@DarkKirb]) [c:2b453c3]
- [model] Add millisecond accuracy to `ID.created_at()` ([@DarkKirb]) [c:965fa7b]
- [http/gateway] Add Rustls and Native-TLS Backends ([@Lakelezz]) [c:15e2c41]

## Changed

- [model] Make MessageUpdateEvent::embeds a Vec<Embed> ([@zeyla]) [c:00f465c]
- [voice] Voice fixes, better API adherence, bitrate control, documentation ([@FelixMcFelix]) [c:393a5ae]
- [builder] Make builders mutably borrowed ([@zeyla], [@Flat], [@Lakelezz], [@Celti]) [c:1546171] [c:6d87d71] [c:b7a6fee] [c:b012ab7]
- [utils] Make Message Builder use &mut self instead of self ([@PvdBerg1998]) [c:1546171]
- [misc.] Update `parking_lot` and `multipart` dependencies ([@Kroisse]) [c:1e50d30]
- [framework] Make sure `delimiter` clears current and default delimiters. ([@Lakelezz]) [c:3f81cf3]
- [framework] Underline command name and "Commands" in plain help ([@hyarsan]) [c:87bc6ca]
- [http]  Replace `hyper` with `reqwest` ([@Lakelezz]) [c:86a8b60]
- [client/gateway] Switch to tungstenite from rust-websocket ([@zeyla]) [c:a5aa2a9]
- [misc.] Update to Rust 2018 ([@Lakelezz]) [c:21518c8]
- [http/model/all] Remove global Cache and HTTP ([@Lakelezz]) [c:712cfa5] [c:3f0ea69]
- [client] Change the `Context::data` field to use an `RwLock` ([@Erk-]) [c:661d778]
- [cache] Pass old Message to `message_update` ([@Mishio595]) [c:40bf272]
- [framework] Check for Ownership in Help System ([@Lakelezz]) [c:fa0376c]
- [framework] Improve Help Consistency ([@Lakelezz]) [c:51b48f4]
- [misc.] Adhere to Rust 2018's idioms ([@Lakelezz]) [c:5d6dc37]
- [client] Add different `Context::new`s based on feature-set. ([@Lakelezz]) [c:625b764]
- [framework] Remodel `Args`'s API ([@acdenisSK]) [c:c472ddd]
- [framework] Rewrite the framework to attributes ([@acdenisSK]) [c:cc81e47]
- [framework] Handle Sub-Groups in the Help-System ([@Lakelezz]) [c:9b591ec]
- [voice] Fewer ffprobe calls when playing audio through ffmpeg ([@FelixMcFelix]) [c:5dff7eb]
- [voice] Optional impls and additional events for AudioReceiver ([@FelixMcFelix]) [c:d955df4]
- [voice] ClientConnect message handling ([@FelixMcFelix]) [c:fa11a30]
- [client] Pass the old voice state if the cache is enabled ([@andreasots]) [c:bd45e42]
- [http] Specify Header's Content Length as `0` ([@eatsfoobars]) [c:a713b40]
- [voice] Switch to `audiopus` ([@Lakelezz]) [c:4af7a98]
- [model] Make `enum`s non-exhaustive ([@Lakelezz]) [c:9cc8816]
- [http] Make the HttpError Send+Sync ([@Erk-]) [c:6cfc0e1]
- [framework] Update `on_mention` to take a `UserId` ([@Celti]) [c:d995fa0]
- [utils] Simplify `from_rgb`, turn some of Colour's functions to `const`. ([@hyarsan]) [c:c149e36]

## Fixed

- Fix ActivityFlags/ActivityTimestamps/ActivityParty deserialization ([@zeyla]) [c:0a77330] [c:d01eeae]
- Fix `MessageBuilder`'s doctests ([@Flat]) [c:a3477a2]

## Removed

- [client] Remove deprecated `Context::edit_profile` ([@zeyla]) [c:bc0d82e]
- [misc.] Remove everything marked `deprecated` since `v0.5.x` or older ([@Lakelezz]) [c:70720ae]

## [0.6.0-rc.2] - 2019-6-14

Thanks to the following for their contributions:
- [@acdenisSK]
- [@AregevDev]
- [@Erk-]
- [@Flat]
- [@icewind1991]
- [@Lakelezz]
- [@nycex]
- [@xacrimon]
- [@zeyla]

A crucial release fixing a lot of misbehaviour:

### Added

- [command_attr] Add some utility structs. ([@acdenisSK]) [c:9162929]
- [command_attr] Add docs for `lacking_ownership`. ([@acdenisSK]) [c:15e7fde]
- [example] Add new Example about Eventing and Timing. ([@Lakelezz]) [c:10b9cc2]
- [example] Add example of embedding a local image in an embed. ([@Erk-]) [c:709c9e4]
- [framework] Add back blocking guilds, channels, and users. ([@acdenisSK]) [c:33f8383]
- [framework] Output the `#[example]` text in help ([@Flat]) [c:7aea26c]
- [model] Add and use `AttachmentId`. ([@Lakelezz]) [c:c8a5f69]
- [model] Add a `channel_id_from_name`-method on `Guild`. ([@xacrimon]) [c:aae22a2]
- [model] Add `GuildChannel::members`. ([@Lakelezz]) [c:ddf7a3]
- [model] Add more guild fields from guild boosting. ([@AregevDev]) [c:4541935]
- [utils] Add `EmbedMessageBuilding`-trait. ([@zeyla]) [c:7c61f95]

### Fixed

- [builder] Return `&mut self` on `voice_channel`. ([@Lakelezz]) [c:0e55b73]
- [ci] Fix Azure Windows build. ([@Lakelezz]) [c:fc3a1f6]
- [client] Fix updates giving only new data. ([@zeyla]) [c:5f7231d]
- [clippy] Fix Clippy-lints. ([@Lakelezz]) [c:cd7d07e]
- [clippy] Implement suggestions from Clippy & remove Clippy arg max config. ([@Flat]) [c:6586830]
- [command_attr] Add missing `s`, `owner_only` became `owners_only`. ([@acdenisSK]) [c:3cf673e]
- [command_attr] Report errors from parsing group options, if any. ([@acdenisSK]) [c:8e01ff6]
- [doc] Small doc fixes for the command macro. ([@acdenisSK]) [c:186e914]
- [doc] Revise Guild's `voice_states` doc. ([@nycex]) [c:0a640a4]
- [example] Update the mentioned feature `methods` to `utils`. ([@Lakelezz]) [c:c970f44]
- [framework] Fix Help displaying Groups and their Commands. ([@Lakelezz]) [c:eca204a]
- [framework] Ensure to properly hide groups. ([@Lakelezz]) [c:5e66cd1]
- [framework] Add help for nested groups and their commands. ([@Lakelezz]) [c:6a37535]
- [framework] Get rid of a redundant feature gate. ([@acdenisSK]) [c:2ae3a48]
- [framework] Treat the actual name and aliases equally. ([@Lakelezz]) [c:82d97c2]
- [framework] Check if message author is in owners HashSet. ([@Flat]) [c:d91594b]
- [framework] Change `owners_privilege` to bypass all permission-checks. ([@Flat]) [c:98532da]
- [http] Fix setting role positions ([@icewind1991]) [c:c14ca32]
- [model] Fix lifetime issue with `send_message`. ([@acdenisSK]) [c:3902caf]
- [model] Fix no-default-features compilation. ([@zeyla]) [c:3de5378]
- [model] Fix `contains_case_insensitive` and `starts_with_case_insensitive`. ([@Flat]) [c:d27d391]
- [voice] Pipe youtube-dl to ffmpeg directly. ([@Flat]) [c:4793a84]

### Changed

- [builder] Use `ToString` on builder-arguments instead of `Display`. ([@acdenisSK]) [c:13fae29]
- [builder] Increase the capabilities for creating a channel. ([@acdenisSK]) [c:f2ff97a]
- [client] Improve `cached`'s name and documentation. ([@Lakelezz]) [c:7706475]
- [command_attr] Rectify command parsing. ([@acdenisSK]) [c:b1eff27]
- [command_attr] Use the function-name. ([@acdenisSK]) [c:05254c8]
- [command_attr] Change `only` to `only_in`. ([@acdenisSK]) [c:26b072f]
- [command_attr] Reinvent `group!` parsing. ([@acdenisSK]) [c:7f9c4e1]
- [command_attr] Stop appending `_HELP_COMMAND` to the generated instance from `#[help]` ([@acdenisSK]) [c:9783b35]
- [example] Update the framework example. ([@acdenisSK]) [c:0fcb43c]
- [example] Update to use shard manager. ([@zeyla]) [c:5375827]
- [framework] Take into regard prefixless groups ([@acdenisSK]) [c:ef15739]
- [framework] Ensure prefixes to be mandatory on help ([@Lakelezz]) [c:ab34f75]
- [model] Update `*Id::created_at()` to return a `DateTime<FixedOffset>` instead. ([@AregevDev]) [c:8d50840]
- [model] Replace `Context` as argument ([@Lakelezz]) [c:62e19a7] [c:58fa50c]
- [rustc] Set minimum Rust version to `1.35.0`. ([@Lakelezz]) [c:8c83fec]
- [voice] Add infinite retry arg to ytdl for rst packets. ([@Flat]) [c:86ec810]

### Removed

- [builder] Remove unused `build`-method. ([@Lakelezz]) [c:c6ae140]
- [client] Remove `quit` method. ([@zeyla]) [c:f7109ee]
- [utils] Remove `VecMap`. ([@acdenisSK]) [c:9450d4b]


## [0.6.0-rc.1] - 2019-5-14

Thanks to the following for their contributions:
- [@acdenisSK]
- [@Lakelezz]
- [@mattico]

A short release for some things we overlooked.

## Added

- [model] Add missing fields of `current_application_info` ([@mattico]) [c:23bed41]
- [builder] Allow for channels to be (or not be) set as nsfw ([@acdenisSK]) [c:1bd5bbc]
- [framework] Bring back old parsing behaviour ([@acdenisSK]) [c:64e97c5]
- [http] `AsRef<Http>` Implementation for `Http` ([@Lakelezz]) [c:b425ceb]

## Fixed

- [misc.] Fix Doc-Links and update Changelog ([@Lakelezz]) [c:c63eaea]

## Changed

- [framework] Take into equation ignoring bots and webhooks for help ([@acdenisSK]) [c:b1559bc]
- [general] Increase minimum support Rust version ([@acdenisSK]) [c:61ac765]
- [general/framework] Shackle the minimum version of uwl to 0.3.2 ([@acdenisSK]) [c:decbc04]

## [0.6.0-rc.0] - 2019-5-6

Thanks to the following for their contributions:

- [@acdenisSK]
- [@andreasots]
- [@Celti]
- [@DarkKirb]
- [@eatsfoobars]
- [@Erk-]
- [@FelixMcFelix]
- [@Flat]
- [@hyarsan]
- [@Kroisse]
- [@Lakelezz]
- [@Mishio595]
- [@PvdBerg1998]
- [@Roughsketch]
- [@zeyla]

# Release candidate

This is a *testing release* for receiving feedback regarding the new big changes introduced, whether they’re satisfactory, or horrid and should be revised, before we officially stabilise them.

Please inform us of any suggestions, or bugs you might have!

# Major breaking changes

Serenity has migrated to the 2018 Rust edition, whose lints and idioms are enforced in its codebase.

The cache and http are no longer globally accessible.  The `Context` now carries instances to them, and as such, all functions that had used the cache and http before, now accept the context as their first parameter in order to operate. *Passing the fields present on the context is acceptable too.*

The framework had been swayed off of builders, and proselytised to procedural, macro-based attributes.
Giving options to your commands might have looked like this:

```rust
command!(foo(ctx, msg, args) {
    ...
});

framework.command("foo", |c|
   c.description("I am foobar")
       .min_args(1)
       .max_args(2)
       .usage("#foo bar baz")
       .cmd(foo));
```

But now, it will be:

```rust
#[command] // Marks this function as a command.
#[description = "I am foobar"] // These are the "parameter" attributes, for providing the options to the attribute macro.
#[min_args(1)]
#[max_args(2)]
#[usage("#foo bar baz")]
fn foo(ctx: &mut Context, msg: &Message, args: Args) -> CommandResult {
    ...

    Ok(())
}
```

The same happened to creating groups, but with `macro!` style flavour, which have become a compulsory step in registering your commands:

```rust
group!({
    name: "fizzbuzz",
    options: {
        prefix: "fezz",
        ...
    },
    commands: [foo],
});
```

All `.command`s and `.on`s are thus replaced with simple calls to `.group`:

```rust
framework.group(&FIZZBUZZ_GROUP); // !
```

! - procedural macros are functions that accept Rust code, return Rust code. The Rust code that the `#[command]` (and similarly, `group!`) macro generates is the function you supplied it with, and a `static` instance of options that you've configured the command with. The static is assigned a suffixed, all uppercase version of the function’s name (or in the case of `group!`,  of the `name` field). Hence this weird identifier from nowhere.

# Book

To help new (and existing) users familiarise themselves with the library better, we have decided to write a book similar to one of Rust's official learning material to the language, [The Book](https://doc.rust-lang.org/book/ch00-00-introduction.html).

It's no ready yet, but we hope that on its release that it will clear misunderstandings (if any), explain the why and how of the library and put you in the right direction of Discord bot making!

## Added

- [builder/model] Permit sending files through the `CreateMessage` builder. ([@Roughsketch]) [c:5405ac2]
- [client] Add Rich Presence parsing support ([@zeyla]) [c:f7360e6]
- [model] Add Slow Mode Rate ([@Lakelezz]) [c:7512c19]
- [voice] Voice reconnection ([@FelixMcFelix]) [c:25cb595] [c:4026d77] [c:2f613c0] [c:0a58e85]
- [model] Add a position propagation method to Channel ([@Erk-]) [c:59b4c60]
- [misc.] Re-export `typemap::sharemap` ([@zeyla]) [c:d2233e2]
- [framework] Add new Check System ([@Lakelezz]) [c:2969561]
- [http/gateway] Rustls support ([@Erk-]) [c:faa773a]
- [model] Add news channel ([@Lakelezz]) [c:1074b28]
- [client] Add EventHandler for raw Events ([@DarkKirb]) [c:2b453c3]
- [model] Add millisecond accuracy to `ID.created_at()` ([@DarkKirb]) [c:965fa7b]
- [http/gateway] Add Rustls and Native-TLS Backends ([@Lakelezz]) [c:15e2c41]

## Changed

- [model] Make MessageUpdateEvent::embeds a Vec<Embed> ([@zeyla]) [c:00f465c]
- [voice] Voice fixes, better API adherence, bitrate control, documentation ([@FelixMcFelix]) [c:393a5ae]
- [builder] Make builders mutably borrowed ([@zeyla], [@Flat], [@Lakelezz], [@Celti]) [c:1546171] [c:6d87d71] [c:b7a6fee] [c:b012ab7]
- [utils] Make Message Builder use &mut self instead of self ([@PvdBerg1998]) [c:1546171]
- [misc.] Update `parking_lot` and `multipart` dependencies ([@Kroisse]) [c:1e50d30]
- [framework] Make sure `delimiter` clears current and default delimiters. ([@Lakelezz]) [c:3f81cf3]
- [framework] Underline command name and "Commands" in plain help ([@hyarsan]) [c:87bc6ca]
- [http]  Replace `hyper` with `reqwest` ([@Lakelezz]) [c:86a8b60]
- [client/gateway] Switch to tungstenite from rust-websocket ([@zeyla]) [c:a5aa2a9]
- [misc.] Update to Rust 2018 ([@Lakelezz]) [c:21518c8]
- [http/model/all] Remove global Cache and HTTP ([@Lakelezz]) [c:712cfa5] [c:3f0ea69]
- [client] Change the `Context::data` field to use an `RwLock` ([@Erk-]) [c:661d778]
- [cache] Pass old Message to `message_update` ([@Mishio595]) [c:40bf272]
- [framework] Check for Ownership in Help System ([@Lakelezz]) [c:fa0376c]
- [framework] Improve Help Consistency ([@Lakelezz]) [c:51b48f4]
- [misc.] Adhere to Rust 2018's idioms ([@Lakelezz]) [c:5d6dc37]
- [client] Add different `Context::new`s based on feature-set. ([@Lakelezz]) [c:625b764]
- [framework] Remodel `Args`'s API ([@acdenisSK]) [c:c472ddd]
- [framework] Rewrite the framework to attributes ([@acdenisSK]) [c:cc81e47]
- [framework] Handle Sub-Groups in the Help-System ([@Lakelezz]) [c:9b591ec]
- [voice] Fewer ffprobe calls when playing audio through ffmpeg ([@FelixMcFelix]) [c:5dff7eb]
- [voice] Optional impls and additional events for AudioReceiver ([@FelixMcFelix]) [c:d955df4]
- [voice] ClientConnect message handling ([@FelixMcFelix]) [c:fa11a30]
- [client] Pass the old voice state if the cache is enabled ([@andreasots]) [c:bd45e42]
- [http] Specify Header's Content Length as `0` ([@eatsfoobars]) [c:a713b40]
- [voice] Switch to `audiopus` ([@Lakelezz]) [c:4af7a98]
- [model] Make `enum`s non-exhaustive ([@Lakelezz]) [c:9cc8816]
- [http] Make the HttpError Send+Sync ([@Erk-]) [c:6cfc0e1]
- [framework] Update `on_mention` to take a `UserId` ([@Celti]) [c:d995fa0]
- [utils] Simplify `from_rgb`, turn some of Colour's functions to `const`. ([@hyarsan]) [c:c149e36]

## Fixed

- Fix ActivityFlags/ActivityTimestamps/ActivityParty deserialization ([@zeyla]) [c:0a77330] [c:d01eeae]
- Fix `MessageBuilder`'s doctests ([@Flat]) [c:a3477a2]

## Removed

- [client] Remove deprecated `Context::edit_profile` ([@zeyla]) [c:bc0d82e]
- [misc.] Remove everything marked `deprecated` since `v0.5.x` or older ([@Lakelezz]) [c:70720ae]

## [0.5.14] - 2019-5-17

This release fixes a few bugs.

Thanks to the following for their contributions:

- [@acdenisSK]
- [@Lakelezz]

## Added

- [model] Allow turning on and off the nsfw property of guilds channels ([@acdenisSK]) [c:68c4f5c]

## Changed

- [builder] Remove `type`-field from `edit`'s request body. ([@Lakelezz]) [c:f648d90]

## Fixed

- [model] Handle serde_json's "arbitrary precision" feature. ([@acdenisSK]) [c:33f4adf]
- [framework] Do not display commands their `help_available` is set to `false`. ([@Lakelezz]) [c:1705338]
- [framework] Ignore bots when using the help-command if framework's `ignore_bots` is set to `true`. ([@acdenisSK]) [c:e40758e]
- [misc.] Rename the `methods`-feature inside the third example to `utils`. ([@Lakelezz]) [c:a7ee6a6]

## [0.5.13] - 2019-3-10

Thanks to the following for their contributions:

- [@acdenisSK]
- [@Erk-]
- [@FelixMcFelix]
- [@ijks]
- [@JellyWX]
- [@Lakelezz]
- [@MOZGIII]

# Breaking change

As unusual as it may sound for a patch version, we had to bump our minimum supported Rust version to 1.31.1 as a consequence of certain dependencies publishing releases containing uncompilable code for 1.25 rustc.

## Added

- [misc.] Include the latest releases in CHANGELOG.md  (@acdenisSK) [c:201dab8] [c:201bc56]
- [misc.] Transition from Travis to Microsoft's Azure CI (@Erk-, @acdenisSK, @Lakelezz) [c:68263ac] [c:bca2f4b] [c:3b8ae67] [c:a0b1dd8] [c:bd48ac5]

## Changed

- [docs] Compile, but do not run tests that were previously ignored (@MOZGIII) [c:00990c0]
- [misc.] Lock `cc` and `base64` dependencies to specific versions (@Lakelezz) [c:bc3d978]
- [general] Update Discord's welcome messages as of 2018-12-19 (@Erk-) [c:e94388]
- [voice] Fewer ffprobe calls when playing audio through ffmpeg (@FelixMcFelix) [c:cfcd342] (Improperly credited under @acdenisSK due to a hiccup on Github's part.)
- [misc.] Define 1.31.1 as the new minimum Rust version (@acdenisSK) [c:07e81b0]
- [misc.] Revert commit [c:bc3d978](https://github.com/serenity-rs/serenity/commit/bc3d978b65ae6d07342bfba4618c249d0beae98e) (@acdenisSK) [c:498e41c]
- [misc.] Bump sodiumoxide to version 0.2 (@DoumanAsh, @MOZGIII) [c:23ae9d8] (Commit done by @acdenisSK, but the intention to upgrade the version were [Douman's](https://github.com/serenity-rs/serenity/pull/454) and [Mozgiii's](https://github.com/serenity-rs/serenity/pull/490))

## Fixed

- [model] Fix "no-cache with http" feature combo of `has_role` (@Erk-) [c:3899547]
- [docs] Use a normal `main` to fix Rust 1.25 compilation (@acdenisSK) [c:b469611]
- [docs]  Fix wording of `timestamp`'s documentation (@acdenisSK) [c:7c09cdd]
- [misc.] Fix typos and perform some language improvements (@ijks) [c:88d914e]
- [docs] Fix tests to work with default features without `cache` (@Lakelezz) [c:e6694f2]
- [voice] Fix connection error being thrown on leaving voice (@JellyWX) [c:62a1aa2](https://github.com/serenity-rs/serenity/commit/62a1aa2abcf0919bf38ef90590aaa363eb03aae0)

## [0.5.12] - 2019-2-14

This is a celebratory release for Valentine's day, which we present to you with utmost courtesy.

Thanks to the following for their contributions:

- [@acdenisSK]
- [@Erk-]
- [@Flat]
- [@Lakelezz]
- [@Mishio595]
- [@mshenrick]
- [@zeyla]

### Upgrade Path

`typemap` does not need to be pulled in as a dependency for serenity anymore.
You can remove `typemap` from your `#[dependencies]` table in your `Cargo.toml` and simply import its types from the prelude:

```rust
use serenity::prelude::{ShareMap, TypeMapKey};
```

### Added

- [model] Add a position propagation method to `Channel` ([@Erk-]) [c:2cb67df]
- [model] Implement `Into<u64>` and `Into<i64>` for ID types ([@Lakelezz]) [c:794393c]
- [general/prelude]  Re-export `ShareMap` and `Key` types from `typemap` ([@zeyla]) [c:b11b4e2]
- [utils/MessageBuilder]  Add support for spoilers ([@acdenisSK]) [c:a56d014]
- [general/example] Add usage of `colour` in example 11 ([@Erk-]) [c:7066ed2]

### Changed

- [http] Limit users from requesting over 100 users ([@Flat]) [c:8bf39a7]
- [client/event-handler] Elaborate causes for `guild_member_removal` ([@Lakelezz]) [c:dd75410]

### Fixed

- [model] Make `Region`’s `Japan`-variant lowercase (fixes inconsistency) ([@Lakelezz]) [c:065f55b]
- [model] Fix imports in `create_channel`’s example. ([@acdenisSK]) [c:bca1530]
- [framework] Fix aliases not being added to commands when using `cmd`. ([@Mishio595]) [c:e8d0628]
- [model] Fix no-cache compilation for `User::nick_in`. ([@zeyla], [@acdenisSK]) [c:11d5b72] [c:98bece3]
- [model] Fix `Emoji::url` to use `.gif` for animated emoji ([@mshenrick]) [c:ae0fc14]
- [http] Correct query string in `Route::guild_ban_optioned` ([@Mishio595]) [c:3c166e3]
- [model] Fix `has_role` temporarily (@Erk-) [c:204e0b9]

## [0.5.11] - 2018-11-12

Mini-release.

Thanks to the following for their contributions:

- [@DoumanAsh]
- [@Lakelezz]

### Added

- [framework] A callback handler that signifies a normal message was received ([@Lakelezz]) [c:16bc3815]
- [model] Convenience methods for getting a nickname ([@Lakelezz]) [c:ed17114c]
- [general] Add link for the `Voice on Windows` wiki entry to README.md ([@Lakelezz]) [c:99b72358]

### Changed

- [general] Update the `base64` and `sodiumoxide` dependencies ([@DoumanAsh]) [c:5f9ed749]
- [general/examples] Turn `unwrap`s to `expect`s and update to nested imports ([@Lakelezz]) [c:d6c4beea]

## [0.5.10] - 2018-11-5

This is a celebration release for the anniversary of the failed Gunpowder Plot enacted against King James of England and Scotland in 1605.

Thanks to the following for their contributions:

- [@acdenisSK]
- [@Bond-009]
- [@Erk-]
- [@Lakelezz]
- [@Proximyst]
- [@perryprog]
- [@zeyla]

# Known issues

Systems with OpenSSL 1.x.x installed will not be able to compile Serenity as it depends on an older version of OpenSSL (0.9.x). To circumvent this, you need to add a `patch` section to your `Cargo.toml` for [ishitatsuyuki's fork](https://github.com/ishitatsuyuki/rust-openssl), which is compatible with 1.x.x, like so:

```toml
[patch.crates-io]
openssl = { git = "https://github.com/ishitatsuyuki/rust-openssl", branch = "0.9.x" }
```

### Upgrade Path

Discord no longer considers channels with the prefix `nsfw-` in their name as NSFW. Per [c:75fb5c04], the `utils::is_nsfw` has been deprecated. Instead, the `is_nsfw` methods on the channel structs (as in `GuildChannel::is_nfsw`) are to be used for checking their nsfw-ness.

### Added

- [general/contributing] Add guideline about maximum characters per line ([@Lakelezz]) [c:12534348]
- [cache] Add a write-lock configuration option ([@Erk-]) [c:b2362dbb] [c:41ff44ba]
- [framework] Prefix-only command (@Lakelezz) [c:6a68f68e]
- [framework] Add an option to disable bypassing checks for owners ([@Lakelezz]) [c:c5285ae1]
- [framework] Add a method for trimming the current argument ([@acdenisSK], [@Lakelezz]) [c:3b050f49] [c:e763d80b]
- [model] Parse the id out of any mention ([@acdenisSK]) [c:d529cf79]
- [utils] Add function to neutralise mentions ([@Lakelezz]) [c:867a7447]

### Fixed

- [client] Compile the client without `cache` feature ([@Erk-]) [c:176fde29]
- [framework] Compile the framework without `cache` feature ([@Bond-009]) [c:9f834b2b]
- [framework] Fix Default Command to inherit group-options ([@Lakelezz]) [c:e32f9b57]
- [model] Fix NSFW checks ([@Lakelezz]) [c:75fb5c04]
- [http/docs] Fix dead links ([@Erk-]) [c:9d141bfc]

### Misc.

- [voice] Don't log event deserialization failures ([@zeyla]) [c:08511dae]
- [voice] Remove unused variable ([@Proximyst]) [c:69931fe3]
- [http] Remove inconsistent braces ([@Proximyst]) [c:ccfa7fdc]
- [cache/http] Change to UNIX line endings ([@Erk-]) [c:8e401f03]
- [docs] Typo fixes ([@perryprog]) [c:9865d9cc]
- [framework] Simplify code by removing negation ([@Lakelezz]) [c:093a1bab]
- [travis] Add `travis_wait` to extend build-time ([@Lakelezz]) [c:5b6574c3]

## [0.5.9] - 2018-09-14

This is a maintenance release fixing a number of bugs with a few miscellaneous
internal changes.

Thanks to the following for their contributions:

- [@acdenisSK]
- [@Lakelezz]
- [@Mishio595]
- [@perryprog]
- [@Roughsketch]
- [@zeyla]

### Added

- [model] Add session start info in `BotGateway` ([@zeyla]) [c:12bbc1a]
- [model] Add `Member::user_id` ([@zeyla]) [c:669da40]
- [framework] Suggest similar commands when using help ([@Lakelezz]) [c:ce79f01]
- [framework] add single group help ([@Lakelezz]) [c:75f6516]

### Fixed

- [http] Fix routing for `http::create_private_channel` (regression from 0.5.6)
  ([@zeyla]) [c:30a325e]
- [http] Fix `GuildChannel::_permissions_for` on no-cache builds (regression
  from 0.5.8) ([@zeyla]) [c:e59f766]
- [http] Change HTTP bulk delete from DELETE to POST (regression from 0.5.6)
  ([@Mishio595]) [c:ebbc324]
- [framework] Make `is_command_visible` work with DMs ([@Roughsketch])
  [c:7295079]
- [utils] Add newline in `MessageBuilder::push_codeblock_safe` ([@zeyla])
  [c:e66812a]
- [framework] Fix `has_correct_permissions` when no guild is available
  ([@Lakelezz]) [c:19c65bd]
- [framework] Check if bots are ignored before dispatching
  `unrecognised_command` ([@Lakelezz]) [c:966cb3e]
- [framework] Fix group prefix ambiguity in help ([@Lakelezz]) [c:f01e6e3],
  [c:c49e02c]
- [framework] Add missing usage on plain help ([@Lakelezz]) [c:823b829]
- [framework] Add usage sample back to help ([@Lakelezz]) [c:82dbff2]
- [framework] Check if group is empty and exclude if so ([@Lakelezz])
  [c:4778e69]
- [model] Message: avoid permission checks in non-guild channels ([@zeyla])
  [c:3fbab76]

### Misc.

- [docs] Fix a broken link in README ([@Mishio595]) [c:41b6e24]
- [docs] Properly link to User in Game docs ([@zeyla]) [c:dec3f13]
- [http] Move low-level http functions to `http::raw` and re-export ([@zeyla])
  [c:6157f61]
- [utils] Add more unit tests for `MessageBuilder` ([@zeyla]) [c:14c6099]
- [framework] Refactor help ([@Lakelezz]) [c:28cdc53]
- [docs] Update client docs to not say user token ([@perryprog]) [c:6ca4bea]
- [framework] Add tests for help ([@Lakelezz]) [c:79d8843]
- [model] Remove cache requirement on `Message::is_private` ([@zeyla])
  [c:fe69ef0]

## [0.5.8] - 2018-08-12

This is a hotfix release for incorrect routing and to fix a large number of
broken documentation links.

Thanks to the following for their contributions:

- [@acdenisSK]
- [@Erk-]
- [@Lakelezz]
- [@Lymia]
- [@Mishio595]
- [@zeyla]


### Upgrade Path

Per [c:71edc3a], methods on ID structs like `ChannelId::find` have been
deprecated and replace with `UserId::to_channel_cached`. Similarly, methods like
`GuildId::get` have been replaced with `GuildId::to_partial_guild`. While the
original methods have not been removed, they have been deprecated.

### Added

- [utils] Add `Colour::hex` ([@Mishio595]) [c:8bec4af]

### Fixed

- [http] Fix various incorrect routes ([@Lymia]) [c:826220f]
- [docs] Fix all the dead links in the docs ([@Erk-]) [c:40053a7]
- [voice] Stop attempting to send silent frames (reverts a commit) ([@zeyla])
  [c:0bbe5f5]

### Changed

- [model] Add `to_*`, `as_*` methods on Id types, deprecate `get` and `find`
  methods ([@Lakelezz]) [c:71edc3a]

### Misc.

- [framework] Fix doctest for `Args::iter_quoted` ([@acdenisSK]) [c:7b0cff6]
- [framework] Remove some code duplication ([@Lakelezz]) [c:516ede3]
- [framework] Don't trim command on failure in default command ([@Lakelezz])
  [c:46b4194]

## [0.5.7] - 2018-08-09

This is a hotfix release for an incorrect warning about cache deadlocking during
event dispatches in the client and fixing some routing method typos due to the
HTTP rewrite.

Thanks to the following for their contributions:

- [@acdenisSK]
- [@Lymia]
- [@zeyla]

### Fixed

- [client] Fix erroneous deadlock detection messages ([@Lymia]) [c:d1266fc]
- [http] Fix some routing issues ([@zeyla]) [c:04b410e]

### Misc.

- Slightly reword a cache update comment ([@acdenisSK]) [c:3a58090]

## [0.5.6] - 2018-08-07

This is a bugfix release that fixes a long-standing bug causing shards to
randomly die under certain rare conditions when dispatching the Ready event,
and compilation of the `cache` and `client` features without the `framework`
feature. This also contains an internal rewrite of the HTTP module.

The minimum required rustc version is now pinned at 1.25.0.

Thanks to the following for their contributions:

- [@acdenisSK]
- [@Erk-]
- [@Lakelezz]
- [@Mishio595]
- [@Roughsketch]
- [@zeyla]

### Upgrade Path

Per [c:01e3c33], `Context::edit_profile` has been deprecated. Call
`serenity::http::edit_profile` instead.

### Added

- [model] `impl AsRef<MessageId> for Message` ([@Mishio595]) [c:1de3937]
- [model] Add `From` impls for `Game`, genericify `Game` params ([@zeyla])
  [c:e1332a5], [c:a4c3fec]
- [http] Make `http::fire`, `http::request` public ([@zeyla]) [c:0d55363]
- [framework] Add no-parse getters and advancer to `Args` ([@acdenisSK])
  [c:73ab20f]
- [model] Add support for new `PRIORITY_SPEAKER` permission ([@Erk-])
  [c:2179623]

### Fixed

- [client] Don't delay Ready event with cache enabled ([@zeyla]) [c:12d5321]
- [framework] Handle no delimiters in `Args` ([@acdenisSK]) [c:e5ea6c1],
  [c:9568e3b]
- [client] Add missing `mut`, fixing no-framework compilation ([@acdenisSK])
  [c:90c7ec4]
- [framework] Check if message is empty in `Args` ([@acdenisSK]) [c:0501020]
- [client] Fix potential cache deadlocking when dispatching ([@zeyla])
  [c:f064d65]
- [framework] Pass failed sub-command to default command ([@Lakelezz])
  [c:db21036]
- [framework] Fix default command upon shortcut prefix ([@Lakelezz]) [c:8f128b2]

### Changed

- [client] Deprecate `Context::edit_profile` ([@zeyla]) [c:01e3c33]

### Misc.

- [model] Fix `ChannelId::send_message`'s dead links ([@acdenisSK]) [c:7a93557]
- [model] Add note about cache in `UserId::get` docs ([@zeyla]) [c:e2873c8]
- [general] Reduce required rustc to 1.25.0 ([@zeyla]) [c:f3f22d7], [c:b324774]
- [model] Make `GuildId::member` use cache when possible ([@Roughsketch])
  [c:21eb42f]
- [framework] Reword some `StandardFramework::complex_bucket` docs
  ([@acdenisSK]) [c:02de778]
- [framework] Internally refactor `positions` ([@acdenisSK]) [c:2a6c3b1]
- [framework] Update `Configuration` default value listings ([@zeyla])
  [c:602c5a7]
- [http] Maintain a single, re-used HTTP client ([@zeyla]) [c:8c0e5a3]
- [http] Redo the HTTP module internally ([@zeyla]) [c:a0b0dd2], [c:4648f58],
  [c:8918201], [c:8301333], [c:bbbf638], [c:9a863bd], [c:c458099], [c:aa437d4]
- [docs] Don't return Result from tests ([@acdenisSK]) [c:e290b03]
- [docs] Fix all dead links in permissions ([@Erk-]) [c:869fff5]

## [0.5.5] - 2018-07-25

This release is mostly a bugfix release. Thanks to the following for their
contributions:

- [@acdenisSK]
- [@drklee3]
- [@foxbot]
- [@Lakelezz]
- [@Mishio595]
- [@perryprog]
- [@TheUnitedStatesOfAmerica]
- [@zeyla]

### Added

- [framework] Add `Args::rest` ([@acdenisSK]) [c:9b2cd75]
- [model] Add `Message::guild_id` structfield ([@foxbot], [@zeyla]) [c:a9e8626],
  [c:3121f90]
- [framework] Improve logic for displaying help ([@Lakelezz]) [c:7937025]
- [http] Add `http::ratelimiting::offset` ([@zeyla]) [c:55555b8]
- [cache] Make the Cache Update API public ([@zeyla]) [c:9e56062]
- [utils] Add associated consts in `utils::Colour` ([@zeyla]) [c:bbfc8e2]
- [model] `impl From<&ID> for ID` for all Id types ([@zelya]) [c:9e45642],
  [c:530ea76]
- [cache] Add a Message cache API ([@zeyla]) [c:e602630]
- [voice] Add `streamer::ffmpeg_optioned` ([@zeyla]) [c:5dab87b], [c:1f3a57e]
- [model] Implement Mentionable for `GuildChannel` ([@Mishio595]) [c:ce8da79]
- [framework] Allow nil prefixes in DMs ([@acdenisSK]) [c:10bbffe]
- [model] Implement `Mentionable` for `ChannelCategory`, `Group`,
  `PrivateChannel` ([@zeyla]) [c:dd3744b], [c:8ce8234], [c:d11d916], [c:5abc7d1]
- [framework] Add checks for groups ([@Lakelezz]) [c:29480e5]
- [framework] Support multiple prefixes for command groups ([@Lakelezz])
  [c:305d200]
- [framework] Add default commands for command groups ([@Lakelezz]) [c:40c8248],
  [c:8aefde0]

### Fixed

- [framework] Handle debug impls better ([@acdenisSK]) [c:caeab28], [c:7eac4d5]
- [framework] Reorder some dispatch checks to fix an owner override bug
  ([@acdenisSK]) [c:8114a7a], [c:93f453b]
- [framework] Force `Args::find{,_n}` to be quote-aware ([@acdenisSK])
  [c:f0f06b7]
- [framework] Fix an `Args` test ([@zeyla]) [c:2ef660e]
- [framework] Fix command visibility on no help ([@Lakelezz]) [c:aeb89af]
- [framework] Add missing `Send + Sync` bounds on `Check` ([@acdenisSK])
  [c:f09b661]
- [utils] Fix `utils::is_nsfw` slicing ([@acdenisSK], [@zeyla]) [c:0067c33],
  [c:ccd2506]
- [utils] Fix `nsfw-` case in `utils::is_nsfw` ([@zeyla]) [c:bd4aa0a]
- [framework] Don't assume all characters at end are 1-length ([@acdenisSK])
  [c:4e4dcb1]
- [framework] Don't suggest command if no command is related to input
  ([@Lakelezz]) [c:614402f]

### Changed

- [model] Make `Invite::guild` and `RichInvite::guild` optional ([@zeyla])
  [c:3a647e3]

### Misc.

- [framework] Fix example typo ([@perryprog]) [c:d0d363f]
- [framework] Add more docs to `Args` ([@acdenisSK]) [c:04b0be1]
- [general] Fix extraneous spaces at the end of lines ([@zeyla]) [c:6ddfef8]
- [http] Add (late) april fool's functions ([@TheUnitedStatesOfAmerica])
  [c:5ffdcea]
- Rename https://github.com/serenity-rs/serenity/commit/6e1edde4a3fe27d0d90db7ea906ca5f115a2d5fb
- [framework] Remove some repitition repition ([@acdenisSK]) [c:10f7548],
  [c:1ec1086]
- [docs] Add more docs to `CreateEmbed::fields` ([@acdenisSK]) [c:703d135]
- [docs] Remove some dead links ([@acdenisSK], [@Lakelezz]) [c:eae624e],
  [c:4cf83d0]
- [docs] Remove old notice about `CreateEmbed::field` ([@acdenisSK]) [c:5b66ace]
- [examples] Add `CreateEmbed::field` and `CreateEmbed::fields` usage to example
  11 ([@drklee3]) [c:a9a2c27]
- [general] Monomorphize all functions ([@zeyla]) [c:7b9764c]
- [general] Update README logo URI ([@zeyla]) [c:2ff765b]
- [docs] Fix doc links with no anchor ([@zeyla]) [c:0d6e019]
- [docs] Add docs for `Args::new` ([@acdenisSK]) [c:b520ec7]
- [general] Fix some clippy lints ([@zeyla]) [c:9da7669]

## [0.5.4] - 2018-06-07

Thanks to the following for their contributions:

- [@acdenisSK]
- [@drklee3]
- [@Lakelezz]
- [@vityafx]
- [@zeyla]

### Added

- [model] Add `Message::member` structfield ([@zeyla]) [c:0e1e8fb]
- [docs] Document example binding names for EventHandler method signatures
  ([@acdenisSK]) [c:08a7110]
- [model] Implement `Mentionable` for `CurrentUser` ([@zeyla]) [c:4a24c90]
- [model] Implement `From<CurrentUser> for User` and
  `From<&CurrentUser> for User` ([@zeyla]) [c:af7f176]
- [framework] Add option for bots to work only in certain channels ([@vityafx])
  [c:457a17e]
- [framework] Differentiate in help whether a command is unavailable in DMs or
  guilds ([@Lakelezz]) [c:89a18aa]
- [framework] Improve `Args` docs ([@acdenisSK]) [c:2603063]
- [model] Add `Message::mentions_user_id`, `Message::mentions_user`
  ([@Lakelezz]) [c:1162e68]
- [docs] Update voice example 06 to make joining join the command invoker's
  voice channel ([@drklee3]) [c:a80aab2]

### Fixed

- [framework] Fix a framework example so it makes sense ([@acdenisSK])
  [c:63fe032]
- [model] Remove deadlocking in `Member::highest_role_info` ([@zeyla])
  [c:c659bbd]
- [framework] Dispatch to a threadpool only if required ([@Lakelezz])
  [c:23c5398]
- [framework] Fix strikethrough behaviour ([@Lakelezz]) [c:32c3bed]

### Misc.

- [general] Fix links to the new repo location ([@Lakelezz], [@zeyla])
  [c:152fe3d] [c:0324e01]
- [framework] Switch to `str::match_indices` for some Args ops ([@acdenisSK])
  [c:cc6b567]
- [framework] Remove `if length == 1` branch in Args functions ([@acdenisSK])
  [c:6346975]
- [framework] Optimize `Args::find`, `Args::find_n` ([@acdenisSK]) [c:5ba521b]
- [framework] Revamp `Args` from the ground up ([@acdenisSK]) [c:ff9edc0]

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
[0.5.0:example-09]: https://github.com/serenity-rs/serenity/blob/91cf5cd401d09a3bca7c2573b88f2e3beb9c0948/examples/09_shard_manager/src/main.rs
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

[v0.1.1:example 06]: https://github.com/serenity-rs/serenity/tree/ccb9d16e5dbe965e5a604e1cb402cd3bc7de0df5/examples/06_command_framework

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

[0.8.0]: https://github.com/serenity-rs/serenity/compare/v0.7.5...v0.8.0
[0.7.5]: https://github.com/serenity-rs/serenity/compare/v0.7.4...v0.7.5
[0.7.4]: https://github.com/serenity-rs/serenity/compare/v0.7.3...v0.7.4
[0.7.3]: https://github.com/serenity-rs/serenity/compare/v0.7.2...v0.7.3
[0.7.2]: https://github.com/serenity-rs/serenity/compare/v0.7.1...v0.7.2
[0.7.1]: https://github.com/serenity-rs/serenity/compare/v0.7.0...v0.7.1
[0.7.0]: https://github.com/serenity-rs/serenity/compare/v0.6.4...v0.7.0
[0.6.4]: https://github.com/serenity-rs/serenity/compare/v0.6.3...v0.6.4
[0.6.3]: https://github.com/serenity-rs/serenity/compare/v0.6.2...v0.6.3
[0.6.2]: https://github.com/serenity-rs/serenity/compare/v0.6.1...v0.6.2
[0.6.1]: https://github.com/serenity-rs/serenity/compare/v0.6.0...v0.6.1
[0.6.0]: https://github.com/serenity-rs/serenity/compare/v0.6.0-rc.2...v0.6.0
[0.6.0-rc.2]: https://github.com/serenity-rs/serenity/compare/v0.6.0-rc.0...v0.6.0-rc.2
[0.6.0-rc.1]: https://github.com/serenity-rs/serenity/compare/v0.6.0-rc.0...v0.6.0-rc.1
[0.6.0-rc.0]: https://github.com/serenity-rs/serenity/compare/v0.5.14...v0.6.0-rc.0
[0.5.14]: https://github.com/serenity-rs/serenity/compare/v0.5.13...v0.5.14
[0.5.13]: https://github.com/serenity-rs/serenity/compare/v0.5.12...v0.5.13
[0.5.12]: https://github.com/serenity-rs/serenity/compare/v0.5.11...v0.5.12
[0.5.11]: https://github.com/serenity-rs/serenity/compare/v0.5.10...v0.5.11
[0.5.10]: https://github.com/serenity-rs/serenity/compare/v0.5.9...v0.5.10
[0.5.9]: https://github.com/serenity-rs/serenity/compare/v0.5.8...v0.5.9
[0.5.8]: https://github.com/serenity-rs/serenity/compare/v0.5.7...v0.5.8
[0.5.7]: https://github.com/serenity-rs/serenity/compare/v0.5.6...v0.5.7
[0.5.6]: https://github.com/serenity-rs/serenity/compare/v0.5.5...v0.5.6
[0.5.5]: https://github.com/serenity-rs/serenity/compare/v0.5.4...v0.5.5
[0.5.4]: https://github.com/serenity-rs/serenity/compare/v0.5.3...v0.5.4
[0.5.3]: https://github.com/serenity-rs/serenity/compare/v0.5.2...v0.5.3
[0.5.2]: https://github.com/serenity-rs/serenity/compare/v0.5.1...v0.5.2
[0.5.1]: https://github.com/serenity-rs/serenity/compare/v0.5.0...v0.5.1
[0.5.0]: https://github.com/serenity-rs/serenity/compare/v0.4.7...v0.5.0
[0.4.5]: https://github.com/serenity-rs/serenity/compare/v0.4.4...v0.4.5
[0.4.4]: https://github.com/serenity-rs/serenity/compare/v0.4.3...v0.4.4
[0.4.3]: https://github.com/serenity-rs/serenity/compare/v0.4.2...v0.4.3
[0.4.2]: https://github.com/serenity-rs/serenity/compare/v0.4.1...v0.4.2
[0.4.1]: https://github.com/serenity-rs/serenity/compare/v0.4.0...v0.4.1
[0.4.0]: https://github.com/serenity-rs/serenity/compare/v0.3.0...v0.4.0
[0.3.0]: https://github.com/serenity-rs/serenity/compare/v0.2.0...v0.3.0
[0.2.0]: https://github.com/serenity-rs/serenity/compare/v0.1.5...v0.2.0
[0.1.5]: https://github.com/serenity-rs/serenity/compare/v0.1.4...v0.1.5
[0.1.4]: https://github.com/serenity-rs/serenity/compare/v0.1.3...v0.1.4
[0.1.3]: https://github.com/serenity-rs/serenity/compare/v0.1.2...v0.1.3
[0.1.2]: https://github.com/serenity-rs/serenity/compare/v0.1.1...v0.1.2
[0.1.1]: https://github.com/serenity-rs/serenity/compare/v0.1.0...v0.1.1
[0.1.0]: https://github.com/serenity-rs/serenity/tree/403d65d5e98bdfa9f0c018610000c4a0b0c7d8d5
[crates.io listing]: https://crates.io/crates/serenity
[library organization]: https://github.com/serenity-rs
[semver]: http://semver.org

[issue:56]: https://github.com/serenity-rs/serenity/issues/56
[rust-websocket:issue:137]: https://github.com/cyderize/rust-websocket/issues/137

[@Alch-Emi]: https://github.com/Alch-Emi
[@Arcterus]: https://github.com/Arcterus
[@AregevDev]: https://github.com/AregevDev
[@abalabahaha]: https://github.com/abalabahaha
[@acdenisSK]: https://github.com/acdenisSK
[@andreasots]: https://github.com/andreasots
[@benjaminrsherman]: https://github.com/benjaminrsherman
[@Bond-009]: https://github.com/Bond-009
[@barzamin]: https://github.com/barzamin
[@bikeshedder]: https://github.com/bikeshedder
[@bippum]: https://github.com/bippum
[@blaenk]: https://github.com/blaenk
[@Caemor]: https://github.com/Caemor
[@CarlGroth]: https://github.com/CarlGroth
[@Celti]: https://github.com/Celti
[@ConcurrentMarxistGC]: https://github.com/ConcurrentMarxistGC
[@DarkKirb]: https://github.com/DarkKirb
[@DeltaEvo]: https://github.com/DeltaEvo
[@DoumanAsh]: https://github.com/DoumanAsh
[@drklee3]: https://github.com/drklee3
[@Elinvynia]: https://github.com/Elinvynia
[@Erk-]: https://github.com/Erk-
[@eatsfoobars]: https://github.com/eatsfoobars
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
[@hyarsan]: https://github.com/hyarsan
[@icewind1991]: https://github.com/icewind1991
[@iCrawl]: https://github.com/iCrawl
[@ikkerens]: https://github.com/ikkerens
[@imnotbad]: https://github.com/imnotbad
[@indiv0]: https://github.com/indiv0
[@ijks]: https://github.com/ijks
[@JellyWX]: https://github.com/JellyWX
[@Jerald]: https://github.com/Jerald
[@jhelwig]: https://github.com/jhelwig
[@jkcclemens]: https://github.com/jkcclemens
[@joek13]: https://github.com/joek13
[@KamranMackey]: https://github.com/KamranMackey
[@Kroisse]: https://github.com/Kroisse
[@kyranet]: https://github.com/kyranet
[@Lakelezz]: https://github.com/Lakelezz
[@LeSeulArtichaut]: https://github.com/LeSeulArtichaut
[@leo-lb]: https://github.com/leo-lb
[@lolzballs]: https://github.com/lolzballs
[@Lymia]: https://github.com/Lymia
[@khazhyk]: https://github.com/khazhyk
[@MaxOhn]: https://github.com/MaxOhn
[@Mendess2526]: https://github.com/Mendess2526
[@mattico]: https://github.com/mattico
[@molenzwiebel]: https://github.com/molenzwiebel
[@megumisonoda]: https://github.com/megumisonoda
[@Mishio595]: https://github.com/Mishio595
[@mshenrick]: https://github.com/mshenrick
[@MOZGIII]: https://github.com/MOZGIII
[@NieDzejkob]: https://github.com/NieDzejkob
[@Noituri]: https://github.com/Noituri
[@NovusTheory]: https://github.com/NovusTheory
[@nabijaczleweli]: https://github.com/nabijaczleweli
[@natsukagami]: https://github.com/natsukagami
[@nickelc]: https://github.com/nickelc
[@nitsuga5124]: https://github.com/nitsuga5124
[@nycex]: https://github.com/nycex
[@Proximyst]: https://github.com/Proximyst
[@perryprog]: https://github.com/perryprog
[@PvdBerg1998]: https://github.com/PvdBerg1998
[@Roughsketch]: https://github.com/Roughsketch
[@rsaihe]: https://github.com/rsaihe
[@Scetch]: https://github.com/Scetch
[@shnarazk]: https://github.com/shnarazk
[@Sreyas-Sreelal]: https://github.com/Sreyas-Sreelal
[@sschroe]: https://github.com/sschroe
[@SunDwarf]: https://github.com/SunDwarf
[@tahahawa]: https://github.com/tahahawa
[@ThatsNoMoon]: https://github.com/ThatsNoMoon
[@TheUnitedStatesOfAmerica]: https://github.com/TheUnitedStatesOfAmerica
[@thelearnerofcode]: https://github.com/thelearnerofcode
[@TitusEntertainment]: https://github.com/TitusEntertainment
[@timotree3]: https://github.com/timotree3
[@vityafx]: https://github.com/vityafx
[@vivianhellyer]: https://github.com/vivianhellyer
[@xentec]: https://github.com/xentec
[@xacrimon]: https://github.com/xacrimon
[@xSke]: https://github.com/xSke
[@Zalaxx]: https://github.com/Zalaxx
[@zeyla]: https://github.com/zeyla

[c:f5dd8bf]: https://github.com/serenity-rs/serenity/commit/f5dd8bf42a7a952c1093925ecd60b46b8f716f60

[c:e005ef1]: https://github.com/serenity-rs/serenity/commit/e005ef19b695fb444f921e84741b97fb2a9d0687
[c:f27c7c1]: https://github.com/serenity-rs/serenity/commit/f27c7c148b14239b7170b3acd6a17137c6986737
[c:d6b0038]: https://github.com/serenity-rs/serenity/commit/d6b003832a37a3ca8d473e5253a1dd656fcc49c8
[c:683ff27]: https://github.com/serenity-rs/serenity/commit/683ff27169cc62a2f7fcfcc84acd9169dd4b3e9c
[c:1361b33]: https://github.com/serenity-rs/serenity/commit/1361b3377ebf00fb31375f5010863f59b66c3f3a
[c:820d50e]: https://github.com/serenity-rs/serenity/commit/820d50ee4fbc72a41a2040f6ced240df7aaa6fa8
[c:581eb2f]: https://github.com/serenity-rs/serenity/commit/581eb2fbbd2a5d401b30f68168fcb2f9d7776452
[c:48c4b59]: https://github.com/serenity-rs/serenity/commit/48c4b5920d41278c741c8ccdb988f24e12dd89af
[c:f2d0ad5]: https://github.com/serenity-rs/serenity/commit/f2d0ad5182c6321aee97fa43896450cd7cefaa85
[c:3ca41fd]: https://github.com/serenity-rs/serenity/commit/3ca41fd1afed11469f623850536263ad97c34e77
[c:a23acc7]: https://github.com/serenity-rs/serenity/commit/a23acc7ad6a78205a04c054b50f96425beb55747
[c:a692bcd]: https://github.com/serenity-rs/serenity/commit/a692bcdc66e0f294486c4c006d55274903e83e6c
[c:a44f16d]: https://github.com/serenity-rs/serenity/commit/a44f16de8f11d108eaa15d3b136ae4e0eef89d05
[c:ec306ee]: https://github.com/serenity-rs/serenity/commit/ec306ee98cccbee0f68ba9dc929dee26f3b13135
[c:5f9a27a]: https://github.com/serenity-rs/serenity/commit/5f9a27a49abb95b9fe100241ed5725b2cf5c6f59
[c:61bcfbc]: https://github.com/serenity-rs/serenity/commit/61bcfbcd42e186b66b670e374ac42fc73f8397b7
[c:91f10dd]: https://github.com/serenity-rs/serenity/commit/91f10dd7ded8120f1364bd0be65ca7b347d79713
[c:d2b19a2]: https://github.com/serenity-rs/serenity/commit/d2b19a27128e80ad298bc87414d6cb0a10060bc6
[c:c3d5264]: https://github.com/serenity-rs/serenity/commit/c3d5264f384429050d64989587ef6b9ba045b7c2
[c:3a313c8]: https://github.com/serenity-rs/serenity/commit/3a313c8c9f8ab3a03992cb6222794b010128b899
[c:4b67d8e]: https://github.com/serenity-rs/serenity/commit/4b67d8e0fd908f9373cb89b14ecb72a84eb84e03
[c:d851fea]: https://github.com/serenity-rs/serenity/commit/d851fea1c012b8a6a62b46ac38311909d0b10989
[c:8bba7b0]: https://github.com/serenity-rs/serenity/commit/8bba7b0269e10dc9859da13a51f5bc9101546bc3
[c:5b0e9f3]: https://github.com/serenity-rs/serenity/commit/5b0e9f3f4ed4975e0b0085ec652dd624b67178f3

[c:05044b6]: https://github.com/serenity-rs/serenity/commit/05044b6f1bdf9047ebfc447e19d199f4a35816e6
[c:8326dc7]: https://github.com/serenity-rs/serenity/commit/8326dc761e0c86672e37a78eecfd8cab22589c82
[c:d189624]: https://github.com/serenity-rs/serenity/commit/d18962468010b7beed1b6311b8b1df42edff4530
[c:5dbe078]: https://github.com/serenity-rs/serenity/commit/5dbe078c5395d0d39bda6a202e1dde367bf0de33
[c:28a91c6]: https://github.com/serenity-rs/serenity/commit/28a91c61f13e08e15adc2b0cb7f211e4cf610c86
[c:b2c951d]: https://github.com/serenity-rs/serenity/commit/b2c951d90acc2061824926573cc1dfe331d0c1e5
[c:d4b45f4]: https://github.com/serenity-rs/serenity/commit/d4b45f4758e8545e913c70dcbf41a122e487ce5d
[c:3a3f9b3]: https://github.com/serenity-rs/serenity/commit/3a3f9b380bfda440c002876eb0726d2f0a8a7d14
[c:08d894e]: https://github.com/serenity-rs/serenity/commit/08d894ef5937c1877c1209816f103421b4c67b80
[c:5b01290]: https://github.com/serenity-rs/serenity/commit/5b0129018301a2235d7cb97b52c9f89cff75da39
[c:6916bfc]: https://github.com/serenity-rs/serenity/commit/6916bfc4d7a30ff331acc4635cd3f30a19c80f80
[c:b5deb39]: https://github.com/serenity-rs/serenity/commit/b5deb391bf0521d049152218a8d774c8db474e5a
[c:3c2f9ad]: https://github.com/serenity-rs/serenity/commit/3c2f9adf4d540597b620866773ac4b574eb71d12
[c:3a449ee]: https://github.com/serenity-rs/serenity/commit/3a449ee8a11cadf1c09d93f860c97ef2dfb522a8
[c:e807288]: https://github.com/serenity-rs/serenity/commit/e8072881af7cf8f826c2f3ae77684ad2a4893841
[c:ab7f631]: https://github.com/serenity-rs/serenity/commit/ab7f6316b551ed0485dac99d328321d8363405e2
[c:db5a09e]: https://github.com/serenity-rs/serenity/commit/db5a09e8c513befd5f41894baededd259f8b6df7
[c:cc2f918]: https://github.com/serenity-rs/serenity/commit/cc2f918fba4b3a1dc8f0bb939a24bf020007de42
[c:240d3e6]: https://github.com/serenity-rs/serenity/commit/240d3e63432f6a1d35ef0a40cbe48d5e9826409c
[c:0b3ad00]: https://github.com/serenity-rs/serenity/commit/0b3ad00853d23db316e2bd0b5759617b598e61aa
[c:173f7fa]: https://github.com/serenity-rs/serenity/commit/173f7fad1486e31c473293ed2ef7ff25a6ce2a08
[c:aed4b24]: https://github.com/serenity-rs/serenity/commit/aed4b24e8e8c511748ae28f33a5ff81280ad1069
[c:ae0acd0]: https://github.com/serenity-rs/serenity/commit/ae0acd0c6442e937c90e49843d66dfd11c544cfa
[c:f897a8d]: https://github.com/serenity-rs/serenity/commit/f897a8dfc39598e055a838e92550de915aa4ef50
[c:346a7fe]: https://github.com/serenity-rs/serenity/commit/346a7feb0da9167760bd69d1d1cc3478c6c379b6
[c:d6699c2]: https://github.com/serenity-rs/serenity/commit/d6699c26c2ae2c29be1e1463b254e104fbd80064
[c:69f2dff]: https://github.com/serenity-rs/serenity/commit/69f2dffdb3caa24e3387a2702593bcb57ba5e690
[c:caeeda1]: https://github.com/serenity-rs/serenity/commit/caeeda155d1853125d2100f3d55b7060d9a27888
[c:487fa04]: https://github.com/serenity-rs/serenity/commit/487fa0413c05f5c1ad688bb534dcedb15a428de8

[c:e6ed1b5]: https://github.com/serenity-rs/serenity/commit/e6ed1b5987814174fcf66dff084be45386a68136
[c:42937e9]: https://github.com/serenity-rs/serenity/commit/42937e9b7414455a9baefeb0c902ba81ff242de4
[c:918273b]: https://github.com/serenity-rs/serenity/commit/918273b8e936796e6424b0c28c0a929f7ce6bf03

[c:2734e27]: https://github.com/serenity-rs/serenity/commit/2734e27a163a1cc585bd7f4f7b5aa0855792ed58
[c:c4b1c60]: https://github.com/serenity-rs/serenity/commit/c4b1c6033c7b21ac314ce6845be957ca69b1d223

[c:de9e8a6]: https://github.com/serenity-rs/serenity/commit/de9e8a673f906311957bb2f6e31026cc57fd86b1
[c:445810f]: https://github.com/serenity-rs/serenity/commit/445810f0673319462b685d849c6ac87ab739f44d
[c:2d3e585]: https://github.com/serenity-rs/serenity/commit/2d3e585506d20c4ffab34ff015679a1dcca30575
[c:65837f5]: https://github.com/serenity-rs/serenity/commit/65837f54a671a30a869fe09e2a1abc70d64a5226
[c:730c959]: https://github.com/serenity-rs/serenity/commit/730c959c73b0e3227a42dc2373aed646e286c3a4
[c:e762ea9]: https://github.com/serenity-rs/serenity/commit/e762ea948d6ee3fdf76991f60e743adcb8c3d8ae
[c:711882b]: https://github.com/serenity-rs/serenity/commit/711882baabde1127b9bf6e2e39116306961f671a
[c:0183714]: https://github.com/serenity-rs/serenity/commit/0183714d450b2285cfae3c619063965783af95c1
[c:21c95fd]: https://github.com/serenity-rs/serenity/commit/21c95fdfd9b4fe8a98d3a0e459e8ab94ceecaa23
[c:ebdeb4e]: https://github.com/serenity-rs/serenity/commit/ebdeb4e456c206ea0cccd94318e4eb19660241a0
[c:d427da4]: https://github.com/serenity-rs/serenity/commit/d427da4a17dd78fe5f4f681855e028abb3fbccee
[c:96b49f9]: https://github.com/serenity-rs/serenity/commit/96b49f97c080ea6fdc2e1bbd1cd1e90958adceb1

[c:50d2a76]: https://github.com/serenity-rs/serenity/commit/50d2a7654e0aa5248c16941b68da30d758262419

[c:aa1070d]: https://github.com/serenity-rs/serenity/commit/aa1070d05f23ea2a7a57857ee47e7b41af36815b
[c:720d9ad]: https://github.com/serenity-rs/serenity/commit/720d9adda4d432cf3fb5ceb890fc0aa751f927bb
[c:c45c1d4]: https://github.com/serenity-rs/serenity/commit/c45c1d47ec70168e90091e676d3fdf0a0d4e4c8c
[c:8594c29]: https://github.com/serenity-rs/serenity/commit/8594c29a2e993da7960d0c63a571bae203e07ea3
[c:85dd1a0]: https://github.com/serenity-rs/serenity/commit/85dd1a011593c293319c26a1fd5e7a45ba0c693d
[c:6d06632]: https://github.com/serenity-rs/serenity/commit/6d066322e1a6a2fd0d2a577b2f7f0b59b842789f
[c:22f3d2a]: https://github.com/serenity-rs/serenity/commit/22f3d2a32e16ef0a12a17ec67415e27a531b095d
[c:8e926f9]: https://github.com/serenity-rs/serenity/commit/8e926f97bccf53e0a2637f81fa8fa6913ed96f9a
[c:3a4e2ed]: https://github.com/serenity-rs/serenity/commit/3a4e2eda25dde94d377fee2bdc088a8c8a2d4e8e
[c:6f7797e]: https://github.com/serenity-rs/serenity/commit/6f7797e45cb9cb887dd0f89eb2f5063fb04d32ee
[c:2439275]: https://github.com/serenity-rs/serenity/commit/2439275d57630fd4e325efe149646c5ef25442bf
[c:ec687ad]: https://github.com/serenity-rs/serenity/commit/ec687adbe0eeba513a462bfa26f779d3bcd4e63e
[c:ccbba0a]: https://github.com/serenity-rs/serenity/commit/ccbba0a67da7514bf0abbdd976beebd0f3a6e30c
[c:09c1e01]: https://github.com/serenity-rs/serenity/commit/09c1e015c2b4ce3c3ed94ca7a44988caf2aff187
[c:e8da420]: https://github.com/serenity-rs/serenity/commit/e8da420e8bdb47da950f8344d7000c5a9d543460

[c:26192fa]: https://github.com/serenity-rs/serenity/commit/26192fa1e8df9a7bd7be6065657890a200432661
[c:e6c5d41]: https://github.com/serenity-rs/serenity/commit/e6c5d418390a90632fb2dee75ddcfd5cc1cc2672
[c:d2df2b9]: https://github.com/serenity-rs/serenity/commit/d2df2b9db9dd23bec2bb3bf8f217a8e437582e2f
[c:d280ed1]: https://github.com/serenity-rs/serenity/commit/d280ed18fedf324699c1173432fd63fd5d0dd657
[c:81d5af1]: https://github.com/serenity-rs/serenity/commit/81d5af16d0d4262f71fe9a3480ad57543d7e2d10
[c:a8f0387]: https://github.com/serenity-rs/serenity/commit/a8f03870c423b4633d6490adb140f5be5d150f40
[c:abd84c2]: https://github.com/serenity-rs/serenity/commit/abd84c202f4932bda8924349126757d1cee93e2a
[c:67f5e3d]: https://github.com/serenity-rs/serenity/commit/67f5e3d970e23da320c091b19fc90314e15db83a
[c:1a209e8]: https://github.com/serenity-rs/serenity/commit/1a209e8815e8319711e012639c90d3de9d322791
[c:45d44cb]: https://github.com/serenity-rs/serenity/commit/45d44cbd65938a8d1f8f65ff865b45316f11a48e
[c:8cdfd7c]: https://github.com/serenity-rs/serenity/commit/8cdfd7cd9a618d4a0edf6e5738979082462cea62
[c:4cf4b21]: https://github.com/serenity-rs/serenity/commit/4cf4b21365bfcd17130867abcae42cee4ca4803b
[c:b7b3a85]: https://github.com/serenity-rs/serenity/commit/b7b3a855c29a036b452f0fbf5ee3f19395bb42f1
[c:eddef7b]: https://github.com/serenity-rs/serenity/commit/eddef7b57f9d1f1380d77ed42e7497015b97ba49
[c:cd4ca1b]: https://github.com/serenity-rs/serenity/commit/cd4ca1b98071b3617e55407a30a3837c2dcfaebc

[c:17f1dc2]: https://github.com/serenity-rs/serenity/commit/17f1dc214f95be129d0ade54ebe8e4e7ab93fbdc
[c:13595ff]: https://github.com/serenity-rs/serenity/commit/13595ff25d35fcaf9bd69f7fe8d75c67f72e676e
[c:e628614]: https://github.com/serenity-rs/serenity/commit/e62861464d96d42a150cce1678cd4afdbea6f121
[c:b28716c]: https://github.com/serenity-rs/serenity/commit/b28716cf09a66b0fc717643c4c6a3e0e8c4afb57

[c:335701e]: https://github.com/serenity-rs/serenity/commit/335701ee06f0083ab98cc245a59a0a77f6d6bd62
[c:3a72058]: https://github.com/serenity-rs/serenity/commit/3a72058d3ef3aa7324c1348e05435575f46f7211
[c:e5081db]: https://github.com/serenity-rs/serenity/commit/e5081db9f8adf370f193340f645f6ab54612b413
[c:759a278]: https://github.com/serenity-rs/serenity/commit/759a2788896f08c79972f1e8fa91ca212c104e52
[c:030bb4d]: https://github.com/serenity-rs/serenity/commit/030bb4d76df5a40fe90bc531d8cd05c4b99599f0
[c:7a0d169]: https://github.com/serenity-rs/serenity/commit/7a0d1698576ecae1159b1079e5689ce0d483b85f
[c:1527838]: https://github.com/serenity-rs/serenity/commit/1527838bfd8d3984ce6c57686d3aac70493e6c55
[c:3e15bb8]: https://github.com/serenity-rs/serenity/commit/3e15bb8ad240431d4351a1ab00d5aed249434fd5

[c:90b7829]: https://github.com/serenity-rs/serenity/commit/90b78294c74bb4fe7f861fad0a1896a5b1ee280f
[c:638b642]: https://github.com/serenity-rs/serenity/commit/638b642c853e0567fe008298691daaa765ef4a5f
[c:c8a8d4f]: https://github.com/serenity-rs/serenity/commit/c8a8d4f2f5b351def970b344045a16d0504d9d8f
[c:0daaac1]: https://github.com/serenity-rs/serenity/commit/0daaac1519e7b583c9d1ea9e31779d7e6d00e5a0
[c:dddd417]: https://github.com/serenity-rs/serenity/commit/dddd417c1b55b4a4908fd65e2cfd2a0010b31e0d
[c:d1addff]: https://github.com/serenity-rs/serenity/commit/d1addff0dff6f199cacb0ed161ca013cb96d7d02
[c:638bb1a]: https://github.com/serenity-rs/serenity/commit/638bb1af7711898267b67b1513d512d55de97d80
[c:5e77718]: https://github.com/serenity-rs/serenity/commit/5e77718d93c97d0b118e4ad77842f311c9382ba9

[c:ab34f75]: https://github.com/serenity-rs/serenity/commit/ab34f75281750ddca64ada640515fef4e01bdf23
[c:58fa50c]: https://github.com/serenity-rs/serenity/commit/58fa50cd4d1fd6660ed6d3692e125cc4f292097b
[c:7aea26c]: https://github.com/serenity-rs/serenity/commit/7aea26c00ffbca9b18f0ac633df8f54252150614
[c:9783b35]: https://github.com/serenity-rs/serenity/commit/9783b354d313e8753134ad1bfd9c53f3aa966684
[c:c970f44]: https://github.com/serenity-rs/serenity/commit/c970f443e645121033b2b3605ba4f15d33b144f6
[c:c8a5f69]: https://github.com/serenity-rs/serenity/commit/c8a5f6999559f3cf3ebb776b31a445b6b7078968
[c:3cf673e]: https://github.com/serenity-rs/serenity/commit/3cf673e670ecac615a1565ed4c96a513327f7e05
[c:8e01ff6]: https://github.com/serenity-rs/serenity/commit/8e01ff64fe5ed4e15a56dce2acd85574ae5a9d0c
[c:c6ae140]: https://github.com/serenity-rs/serenity/commit/c6ae1402d5be9fad62c348549141e06aa08cb43a
[c:13fae29]: https://github.com/serenity-rs/serenity/commit/13fae29e053dda813448dca97c667d4b5a0519a4
[c:9450d4b]: https://github.com/serenity-rs/serenity/commit/9450d4b55e824bd841577f7bb58f9916d98b9c09
[c:b1eff27]: https://github.com/serenity-rs/serenity/commit/b1eff278bdb876612f5ab99a566e680ffc1db11a
[c:4793a84]: https://github.com/serenity-rs/serenity/commit/4793a8482915b6a4438b5e209144d0fa5f0948ca
[c:aae22a2]: https://github.com/serenity-rs/serenity/commit/aae22a2011f3fc65eecd7a89f473df9de9fd5232
[c:10b9cc2]: https://github.com/serenity-rs/serenity/commit/10b9cc23bd22bc50732fc698c7af4c12c306f695
[c:186e914]: https://github.com/serenity-rs/serenity/commit/186e9148a616f19fdbd526b8a6c7191268ee2936
[c:86ec810]: https://github.com/serenity-rs/serenity/commit/86ec810e06e4f5fad11e10a787aad3f33d7fe9a1
[c:eca204a]: https://github.com/serenity-rs/serenity/commit/eca204a8cc4828eecbe914cabe099fbc50901656
[c:cd7d07e]: https://github.com/serenity-rs/serenity/commit/cd7d07e02aef5810806c8fea09cceb58d7c92578
[c:5e66cd1]: https://github.com/serenity-rs/serenity/commit/5e66cd13a2459bd6b93bffde9827929112443c25
[c:9162929]: https://github.com/serenity-rs/serenity/commit/916292909e9b1cc9db9bd96536632999d9777fdc
[c:6a37535]: https://github.com/serenity-rs/serenity/commit/6a3753589946f9cdd1915aa4277cf61212347025
[c:33f8383]: https://github.com/serenity-rs/serenity/commit/33f83838942cd89f6ba3f981575c20a2f19039a0
[c:05254c8]: https://github.com/serenity-rs/serenity/commit/05254c8376b6b198aff6734aa8c0b58560e3a756
[c:0e55b73]: https://github.com/serenity-rs/serenity/commit/0e55b73f244f8903878f384cddaaf7d67feb0530
[c:0a640a4]: https://github.com/serenity-rs/serenity/commit/0a640a43122a24125f05f7610934dd09d267177c
[c:2ae3a48]: https://github.com/serenity-rs/serenity/commit/2ae3a48cdb6a62066c72c08fcfd31300f70943ea
[c:26b072f]: https://github.com/serenity-rs/serenity/commit/26b072f67f9662214738f0c0db7856c7fe7ef4b7
[c:15e7fde]: https://github.com/serenity-rs/serenity/commit/15e7fdee8de2068c5023a7a6d5b372117ba0b4c5
[c:709c9e4]: https://github.com/serenity-rs/serenity/commit/709c9e45d59b30797062cc32afa910a6b0da7476
[c:3902caf]: https://github.com/serenity-rs/serenity/commit/3902caf9881c9bc2a007e6f417002caef81a3ae5
[c:ddf7a3]: https://github.com/serenity-rs/serenity/commit/ddf7a3f960d06e666288f68a36567f9c73a9fde8
[c:6586830]: https://github.com/serenity-rs/serenity/commit/6586830e9737a6fea256c8b48e83760898e33285
[c:7706475]: https://github.com/serenity-rs/serenity/commit/77064758e63b21825f33569d10008edaa6bcd4d5
[c:fc3a1f6]: https://github.com/serenity-rs/serenity/commit/fc3a1f6f911d2a9615c9647b252624c5bdeffb97
[c:7f9c4e1]: https://github.com/serenity-rs/serenity/commit/7f9c4e1b4d829ce99614271c278a06eb678b778e
[c:f2ff97a]: https://github.com/serenity-rs/serenity/commit/f2ff97aed2c2640a99205ab9f947237aa2ccf01e
[c:0fcb43c]: https://github.com/serenity-rs/serenity/commit/0fcb43c06895efde905482e4b9120c1bde3a671c
[c:82d97c2]: https://github.com/serenity-rs/serenity/commit/82d97c2513e547dd5275f07fe7327e0433f18860
[c:d91594b]: https://github.com/serenity-rs/serenity/commit/d91594baadf49c725a2478b85509194c17a3ba67
[c:4541935]: https://github.com/serenity-rs/serenity/commit/4541935243d794cc760f46520f0cfa3f4994a9a3
[c:7c61f95]: https://github.com/serenity-rs/serenity/commit/7c61f958b139159a6c4595f6c99d7812d69d339b
[c:5f7231d]: https://github.com/serenity-rs/serenity/commit/5f7231db550ba5233773b801710fd593642cbb2f
[c:f7109ee]: https://github.com/serenity-rs/serenity/commit/f7109ee74d62652569f860ac39b70962bc08bbeb
[c:5375827]: https://github.com/serenity-rs/serenity/commit/5375827523034e5074a3490a70f002caefd77b7b
[c:3de5378]: https://github.com/serenity-rs/serenity/commit/3de537875734a9a847ec1ac986430371f1f27d4a
[c:8d50840]: https://github.com/serenity-rs/serenity/commit/8d508401371ea7595030a7a2bafa76c7b1c76251
[c:98532da]: https://github.com/serenity-rs/serenity/commit/98532da727b8f813985f333290a5d954d0a654c6
[c:d27d391]: https://github.com/serenity-rs/serenity/commit/d27d391afaf55ba3b96c13701a71beb0ce5a2844
[c:8c83fec]: https://github.com/serenity-rs/serenity/commit/8c83fec4d85dee65078bad65b9abc51daf564af1
[c:ef15739]: https://github.com/serenity-rs/serenity/commit/ef15739329809b0c3c07d696b5bce4faf2a0e345
[c:62e19a7]: https://github.com/serenity-rs/serenity/commit/62e19a721c1b04bf9e5cd2f9ab3429b4f6634a6f
[c:c14ca32]: https://github.com/serenity-rs/serenity/commit/c14ca326ab49ae01f13df3a08c7a64d23a1429ae

[c:23bed41]: https://github.com/serenity-rs/serenity/commit/23bed417864a53c6e050ec732f72cf175ef293df
[c:1bd5bbc]: https://github.com/serenity-rs/serenity/commit/1bd5bbc8004dab8150ba4c631b8323440634cd35
[c:c63eaea]: https://github.com/serenity-rs/serenity/commit/c63eaeafe5c23006712021cc3efde9988efb2fd2
[c:64e97c5]: https://github.com/serenity-rs/serenity/commit/64e97c51367630f04d1d58ab917d584cd6e9c15a
[c:b425ceb]: https://github.com/serenity-rs/serenity/commit/b425ceb51ac34e55ce055ad981569818fa2558ea
[c:b1559bc]: https://github.com/serenity-rs/serenity/commit/b1559bc38f99a65cdf3231ddafa84c51b5829b1a
[c:61ac765]: https://github.com/serenity-rs/serenity/commit/61ac765c9d5ad75a848322418d4d2c4ad0c54236
[c:decbc04]: https://github.com/serenity-rs/serenity/commit/decbc04c01cbb24755a0ac1219c9778b9c53d63c

[c:5405ac2]: https://github.com/serenity-rs/serenity/commit/5405ac2a46c5f90451de0a2c68f6f6d323ce299b
[c:f7360e6]: https://github.com/serenity-rs/serenity/commit/f7360e6675d100c7af9c0a67ed47d76e64672e37
[c:7512c19]: https://github.com/serenity-rs/serenity/commit/7512c19ff3b395e57fccaa8f4cfb6e5923138414
[c:25cb595]: https://github.com/serenity-rs/serenity/commit/25cb5959dea20803a236ff151d38b015554e3ea4
[c:4026d77]: https://github.com/serenity-rs/serenity/commit/4026d77a229027170f516ceee14763422d1b5ba2
[c:2f613c0]: https://github.com/serenity-rs/serenity/commit/2f613c0e817cd880941b9d72f4aaed3f67d6722a
[c:0a58e85]: https://github.com/serenity-rs/serenity/commit/0a58e858ea8370a90c465f47b6cf8b4c83263c65
[c:59b4c60]: https://github.com/serenity-rs/serenity/commit/59b4c60a1db57663428c4ea476dc099af1af75a7
[c:d2233e2]: https://github.com/serenity-rs/serenity/commit/d2233e25e1badb9379bd1b91896afd579f7f0106
[c:2969561]: https://github.com/serenity-rs/serenity/commit/2969561517ff4a0625e4a22230dd807ab62c4aa8
[c:faa773a]: https://github.com/serenity-rs/serenity/commit/faa773a301ca1a277f4912f2bcc62abf7caeda31
[c:1074b28]: https://github.com/serenity-rs/serenity/commit/1074b28560c3e1a9bb9ec4796d693136b7f6714b
[c:2b453c3]: https://github.com/serenity-rs/serenity/commit/2b453c365c0169475c67977f2e081f67083b734a
[c:965fa7b]: https://github.com/serenity-rs/serenity/commit/965fa7bf088e5ee5a79efb0b7167478bb0fe719c
[c:15e2c41]: https://github.com/serenity-rs/serenity/commit/15e2c41ca95bcc9666e45eea542f7c712ede9949
[c:00f465c]: https://github.com/serenity-rs/serenity/commit/00f465ceb93f6f4809e121ea00dd9c7bba630e62
[c:393a5ae]: https://github.com/serenity-rs/serenity/commit/393a5aec2a9823549ac7cc1d376991651a61f33d
[c:1546171]: https://github.com/serenity-rs/serenity/commit/15461712914708b51b19f8cf0ddfd0851b63f93e
[c:6d87d71]: https://github.com/serenity-rs/serenity/commit/6d87d7105deda09dc2c08f554afd1371dc2eadf3
[c:b7a6fee]: https://github.com/serenity-rs/serenity/commit/b7a6feee7c9896f0ad4fecc31f3cba4ec5d40429
[c:b012ab7]: https://github.com/serenity-rs/serenity/commit/b7a6feee7c9896f0ad4fecc31f3cba4ec5d40429
[c:1546171]: https://github.com/serenity-rs/serenity/commit/15461712914708b51b19f8cf0ddfd0851b63f93e
[c:1e50d30]: https://github.com/serenity-rs/serenity/commit/1e50d30d405050751a91e3cbc3d8b4aaeef9217a
[c:3f81cf3]: https://github.com/serenity-rs/serenity/commit/3f81cf3392436ca82d9eb55949fb7c7f5557d820
[c:87bc6ca]: https://github.com/serenity-rs/serenity/commit/87bc6ca529e211bbb427f05a58519724eed5c443
[c:86a8b60]: https://github.com/serenity-rs/serenity/commit/86a8b60b4c84cf2239d6870454dda3c5abab98eb
[c:a5aa2a9]: https://github.com/serenity-rs/serenity/commit/a5aa2a9c16e96741e59524df78be3ae3d4c5788c
[c:21518c8]: https://github.com/serenity-rs/serenity/commit/21518c8590c055d3eab6c99ebdd824721b7b0a73
[c:712cfa5]: https://github.com/serenity-rs/serenity/commit/712cfa58c3e426c698b90a0bc49de3f81258c59b
[c:3f0ea69]: https://github.com/serenity-rs/serenity/commit/3f0ea6985e2333d3b04af174a811357d96aa3e02
[c:661d778]: https://github.com/serenity-rs/serenity/commit/661d7787ecb399803ae8794adffdea7df44f6839
[c:40bf272]: https://github.com/serenity-rs/serenity/commit/40bf272cee3d9bcded6598a830c2b54dfad2d504
[c:fa0376c]: https://github.com/serenity-rs/serenity/commit/fa0376c543f545f53c67213cc3d3ee4aebe26ea8
[c:51b48f4]: https://github.com/serenity-rs/serenity/commit/51b48f498cba54e3d05b8bcd79d370e429501f9a
[c:5d6dc37]: https://github.com/serenity-rs/serenity/commit/5d6dc37082412fb0f254000cc19211f78bbb7327
[c:625b764]: https://github.com/serenity-rs/serenity/commit/625b7643fddb6c1a13eb5dab6ae3c536e44f2780
[c:c472ddd]: https://github.com/serenity-rs/serenity/commit/c472ddd8aad713c0a378af3d7740c799f36d95ab
[c:cc81e47]: https://github.com/serenity-rs/serenity/commit/cc81e47d518402de79e287b6d0e80a3a59a74d26
[c:9b591ec]: https://github.com/serenity-rs/serenity/commit/9b591ec0219f62c3c84fc9355c3746e73ce76042
[c:5dff7eb]: https://github.com/serenity-rs/serenity/commit/5dff7eb3d9d59cf8ead692b4ca00bf69e888649f
[c:d955df4]: https://github.com/serenity-rs/serenity/commit/d955df401a3e7b91cb22c037965a272978e5a575
[c:fa11a30]: https://github.com/serenity-rs/serenity/commit/fa11a30bafa050a56df7138275556ddd54895b93
[c:bd45e42]: https://github.com/serenity-rs/serenity/commit/bd45e42ce75b25f3fd9abff9098e08d82b288c17
[c:a713b40]: https://github.com/serenity-rs/serenity/commit/a713b400c79995daa306ec975ac7a99dcabc3c65
[c:4af7a98]: https://github.com/serenity-rs/serenity/commit/4af7a986579b67d56bdbf6254256187184aa4897
[c:9cc8816]: https://github.com/serenity-rs/serenity/commit/9cc8816ec56295541230c87992500fee0aaa8696
[c:6cfc0e1]: https://github.com/serenity-rs/serenity/commit/6cfc0e18181a8c93906ed4ea25eb68796e0f4720
[c:d995fa0]: https://github.com/serenity-rs/serenity/commit/d995fa0e08c67e1a7f217b427d4d9b4dcedaa45b
[c:0a77330]: https://github.com/serenity-rs/serenity/commit/0a773302eb57c5b7e024f91336cc0547a8746616
[c:d01eeae]: https://github.com/serenity-rs/serenity/commit/d01eeae4fbd7676ceb2bb903b3933b7e939ba64e
[c:a3477a2]: https://github.com/serenity-rs/serenity/commit/a3477a2cad7d36110acb0316df927bf8611ebece
[c:bc0d82e]: https://github.com/serenity-rs/serenity/commit/bc0d82eb73f1d5c277dbe9865540b7a623d373b2
[c:70720ae]: https://github.com/serenity-rs/serenity/commit/70720aeeee44d67a4cb2d58a0c375a54c9be95a2

[c:f648d90]: https://github.com/serenity-rs/serenity/commit/f648d9093f87354bbec03228fa647f6dd9afb03a
[c:33f4adf]: https://github.com/serenity-rs/serenity/commit/33f4adfe0f6303ac6b39e8c3db6f413e2289c81b
[c:1705338]: https://github.com/serenity-rs/serenity/commit/17053381b1481e753abdcd319143ddd63467605d
[c:e40758e]: https://github.com/serenity-rs/serenity/commit/e40758eefdbe5a2b62f252cde69e7dec04898a09
[c:a7ee6a6]: https://github.com/serenity-rs/serenity/commit/a7ee6a674ae158839466db58ad7e090bb64dc797
[c:68c4f5c]: https://github.com/serenity-rs/serenity/commit/68c4f5c907993c70fc4a590a6f7d06ee0dba98ee

[c:201dab8]: https://github.com/serenity-rs/serenity/commit/201dab8fa4c31d6e840f88b691772c5b0961780f
[c:201bc56]: https://github.com/serenity-rs/serenity/commit/201bc56f79ab913db32b3453bc8d61a7bfa0ee17
[c:68263ac]: https://github.com/serenity-rs/serenity/commit/68263acc637010f854d986bdcd838593b0dc02f0
[c:bca2f4b]: https://github.com/serenity-rs/serenity/commit/bca2f4be1e603122e681423410b7f30562912727
[c:3b8ae67]: https://github.com/serenity-rs/serenity/commit/3b8ae6712cb1a6d771704de97b2ea7af3f1f7138
[c:a0b1dd8]: https://github.com/serenity-rs/serenity/commit/a0b1dd8b2f8dc4c4a3b924d6ed686e5ba006e60d
[c:bd48ac5]: https://github.com/serenity-rs/serenity/commit/bd48ac5106c540358babff1e31a81e1e1ab7cec0
[c:00990c0]: https://github.com/serenity-rs/serenity/commit/00990c05bb5bfaf0e0ee5e52ed1faefe142277a3
[c:bc3d978]: https://github.com/serenity-rs/serenity/commit/bc3d978b65ae6d07342bfba4618c249d0beae98e
[c:e94388]: https://github.com/serenity-rs/serenity/commit/e94388115845aba533eb7d08f13fddd46ef30f69
[c:cfcd342]: https://github.com/serenity-rs/serenity/commit/cfcd342baf254d35da948cce9fe1c39a284ce6cb
[c:07e81b0]: https://github.com/serenity-rs/serenity/commit/07e81b02143d57f5abced7e0d4fe13d40da7a5de
[c:498e41c]: https://github.com/serenity-rs/serenity/commit/498e41c91453a1d895e5c46e76310f92b44127c0
[c:3899547]: https://github.com/serenity-rs/serenity/commit/3899547968abbdf3a51071a5d9eccdfaad4cc0c4
[c:b469611]: https://github.com/serenity-rs/serenity/commit/b4696115c05eefaabff8b0f5ceb3b12b900dda2b
[c:7c09cdd]: https://github.com/serenity-rs/serenity/commit/7c09cdd27d28264444f34ab8157bc9aa78cbc096
[c:88d914e]: https://github.com/serenity-rs/serenity/commit/88d914e32071ef9cefcfbc4cd74df8f664b66377
[c:e6694f2]: https://github.com/serenity-rs/serenity/commit/e6694f27d74069c80f642ab17ce73d45962fd547
[c:62a1aa2]: https://github.com/serenity-rs/serenity/commit/62a1aa2abcf0919bf38ef90590aaa363eb03aae0
[c:23ae9d8]: https://github.com/serenity-rs/serenity/commit/23ae9d83f96e0d1ad28905deb73eb4ad4fed61a8

[c:2cb67df]: https://github.com/serenity-rs/serenity/commit/2cb67df6a0441b70462434429030cef759c7c57c
[c:794393c]: https://github.com/serenity-rs/serenity/commit/794393c9656e2de916ca72fd63393b26d4fef7a0
[c:b11b4e2]: https://github.com/serenity-rs/serenity/commit/b11b4e23ca576b0b23d67c0b28c7578340762b44
[c:a56d014]: https://github.com/serenity-rs/serenity/commit/a56d0146c6c287f912d65882cc3fa7516355d156
[c:7066ed2]: https://github.com/serenity-rs/serenity/commit/7066ed24a9d4f3386972f9173673034109546378
[c:8bf39a7]: https://github.com/serenity-rs/serenity/commit/8bf39a7a6971543d2deacec4ec77bd871e046ff2
[c:dd75410]: https://github.com/serenity-rs/serenity/commit/dd75410c977dd21fe540e01c3b68731c0718e941
[c:065f55b]: https://github.com/serenity-rs/serenity/commit/065f55b05b174acdf37ea78e9f00b5ed9b10dd28
[c:bca1530]: https://github.com/serenity-rs/serenity/commit/bca1530a6cebafe028d4a3a58f332b7985d5229f
[c:e8d0628]: https://github.com/serenity-rs/serenity/commit/e8d0628a35adeca44386e7a9a5e58714a66a4ac2
[c:11d5b72]: https://github.com/serenity-rs/serenity/commit/11d5b724a95e47001028a5d0d75d91380463b1bc
[c:98bece3]: https://github.com/serenity-rs/serenity/commit/98bece30c6f4ddb265c635b8c285aec1ea42c6d9
[c:ae0fc14]: https://github.com/serenity-rs/serenity/commit/ae0fc14793e6e55a09ba05f2769441f95461f327
[c:3c166e3]: https://github.com/serenity-rs/serenity/commit/3c166e38e00fa9363504eba69d2e2a15ccf046c7
[c:204e0b9]: https://github.com/serenity-rs/serenity/commit/204e0b94a9a4ea2b7f9c0fc88e3a7b097f2c00bc

[c:16bc3815]: https://github.com/serenity-rs/serenity/commit/16bc3815b3420ae2224667e6e1bbdf768760fd87
[c:5f9ed749]: https://github.com/serenity-rs/serenity/commit/5f9ed7497dc08f238fad8e06d041e0b84ac1d00a
[c:ed17114c]: https://github.com/serenity-rs/serenity/commit/ed17114c3d5052eb88b95217bd248bba9a294e6a
[c:99b72358]: https://github.com/serenity-rs/serenity/commit/99b7235888fcabf6952812eb54150ee53989fa4a
[c:d6c4beea]: https://github.com/serenity-rs/serenity/commit/d6c4beeaf89940731c3f2fff14199414dc478342

[c:08511dae]: https://github.com/serenity-rs/serenity/commit/08511dae397f4d99e222d24f546035108a885da0
[c:9f834b2b]: https://github.com/serenity-rs/serenity/commit/9f834b2ba32444fdc6efebd601d062a7f71b3fcb
[c:3b050f49]: https://github.com/serenity-rs/serenity/commit/3b050f49fbc2055c3940b4cacd05b3068856c7b5
[c:e763d80b]: https://github.com/serenity-rs/serenity/commit/e763d80b7697f92e84c2d84ace5ea9fc50a9f9f0
[c:d529cf79]: https://github.com/serenity-rs/serenity/commit/d529cf79af4e493700aa9c69bbb690dbc47a80b8
[c:ccfa7fdc]: https://github.com/serenity-rs/serenity/commit/ccfa7fdc750f567f713e01e6f8e8ccfd0cdec2fb
[c:69931fe3]: https://github.com/serenity-rs/serenity/commit/69931fe349a55eee6ef2735dcfa3823a5988df69
[c:9d141bfc]: https://github.com/serenity-rs/serenity/commit/9d141bfcb23894c5017bae823faa3b144734d42e
[c:8e401f03]: https://github.com/serenity-rs/serenity/commit/8e401f03683b8f2cbce161be669cbd85c791e798
[c:9865d9cc]: https://github.com/serenity-rs/serenity/commit/9865d9ccd727a7f6c5c9a6094b87af0f6353831b
[c:093a1bab]: https://github.com/serenity-rs/serenity/commit/093a1babec70fc331ec08ba14f23f1c14a08178f
[c:5b6574c3]: https://github.com/serenity-rs/serenity/commit/5b6574c351daa8c28c0fbcf666a14289b4505be3
[c:e32f9b57]: https://github.com/serenity-rs/serenity/commit/e32f9b57d22c37a3238e83d086694304eb6c0cd5
[c:b2362dbb]: https://github.com/serenity-rs/serenity/commit/b2362dbb0014781bd7757a9e322ae3b8d5f2fadf
[c:c5285ae1]: https://github.com/serenity-rs/serenity/commit/c5285ae1824dd26adbbd2f0b876eed607f64baa1
[c:6a68f68e]: https://github.com/serenity-rs/serenity/commit/6a68f68e6cb95af38666a4f5d9a6ad4b39fa88c6
[c:75fb5c04]: https://github.com/serenity-rs/serenity/commit/75fb5c041511077e60e577e55039acc33d624569
[c:176fde29]: https://github.com/serenity-rs/serenity/commit/176fde296b143e230ea8889073c69b34a95d41f6
[c:12534348]: https://github.com/serenity-rs/serenity/commit/12534348792edf78cddccb3169e068da27cfcb5e
[c:41ff44ba]: https://github.com/serenity-rs/serenity/commit/41ff44ba4e4bdd7aa77bfbfce201b89c7990d9e4
[c:867a7447]: https://github.com/serenity-rs/serenity/commit/867a744720c46c0b04a2d34c2119ad366aa440ef

[c:12bbc1a]: https://github.com/serenity-rs/serenity/commit/12bbc1ab79176d39c2528cae3c762404b0d5f8ab
[c:14c6099]: https://github.com/serenity-rs/serenity/commit/14c6099ced49623b0c3a373b9a21d0574f9294c9
[c:19c65bd]: https://github.com/serenity-rs/serenity/commit/19c65bd27f27192cc9a8a04c1d38ec08b62652c8
[c:28cdc53]: https://github.com/serenity-rs/serenity/commit/28cdc5328687b74772e37da89caff5751e30a2a5
[c:30a325e]: https://github.com/serenity-rs/serenity/commit/30a325ea840755cf74f376657d9a1e9ac363e92e
[c:3fbab76]: https://github.com/serenity-rs/serenity/commit/3fbab7638be44914a7a28ac366ca03d4d1df9bba
[c:41b6e24]: https://github.com/serenity-rs/serenity/commit/41b6e247b0484fc2ff3e254bb61d104b2e94cbdb
[c:4778e69]: https://github.com/serenity-rs/serenity/commit/4778e6940131e97691f5e1e3d04a28480a9066cc
[c:6157f61]: https://github.com/serenity-rs/serenity/commit/6157f61600d656219491f21f533f63c8f362bd1b
[c:669da40]: https://github.com/serenity-rs/serenity/commit/669da407111f924a5dc498c15c0c0b43f7b42411
[c:6ca4bea]: https://github.com/serenity-rs/serenity/commit/6ca4bea21ac83034c3ff1d4adf79754c80df85ca
[c:7295079]: https://github.com/serenity-rs/serenity/commit/729507947c05c313d37b4b31059f41ba8e6f147a
[c:75f6516]: https://github.com/serenity-rs/serenity/commit/75f6516fceb6d8e124f91ae25a10f74f183337ad
[c:79d8843]: https://github.com/serenity-rs/serenity/commit/79d8843e3640bcc6ffffc0101f3ef458f6770684
[c:823b829]: https://github.com/serenity-rs/serenity/commit/823b8299bb88013ce900e2f8d4b5745556380c72
[c:82dbff2]: https://github.com/serenity-rs/serenity/commit/82dbff282d4eefe7a7125f4393eef2d2eee3beb5
[c:966cb3e]: https://github.com/serenity-rs/serenity/commit/966cb3e00a7c8a803a299db8f792d42542d5896a
[c:c49e02c]: https://github.com/serenity-rs/serenity/commit/c49e02ca024b0263d2b7e23e67338558555101ea
[c:ce79f01]: https://github.com/serenity-rs/serenity/commit/ce79f0183d9fc457ce0fc10fa94e3a1350f33f66
[c:dec3f13]: https://github.com/serenity-rs/serenity/commit/dec3f13ac10b7d22a45ae8393dda95f0a796aee7
[c:e59f766]: https://github.com/serenity-rs/serenity/commit/e59f766c24b53b9c98109e8cfeafdec36feed161
[c:e66812a]: https://github.com/serenity-rs/serenity/commit/e66812aa3b8458634901ca7226e5547f0e4be9eb
[c:ebbc324]: https://github.com/serenity-rs/serenity/commit/ebbc32438e1cca94da80b00ae753e3cde86fb73f
[c:f01e6e3]: https://github.com/serenity-rs/serenity/commit/f01e6e35c42372984f52d53ae8a7d4fa4712047b
[c:fe69ef0]: https://github.com/serenity-rs/serenity/commit/fe69ef034c2d6561e05ff67f6a419b7b4a42c04e

[c:0bbe5f5]: https://github.com/serenity-rs/serenity/commit/0bbe5f5dde6989a8d6a4d4910bf026b1b801fef9
[c:40053a7]: https://github.com/serenity-rs/serenity/commit/40053a71931bb63c43eb6f469ee3c94383c9e90a
[c:46b4194]: https://github.com/serenity-rs/serenity/commit/46b419460254edc2343b5a184952ab5c6e53b287
[c:516ede3]: https://github.com/serenity-rs/serenity/commit/516ede3649b74bca8631d05397e330cde0632fee
[c:71edc3a]: https://github.com/serenity-rs/serenity/commit/71edc3a11ac450728bca19ca7cab7c84079d59f0
[c:7b0cff6]: https://github.com/serenity-rs/serenity/commit/7b0cff66f483687b26f3129e7b093f6a87fb1383
[c:826220f]: https://github.com/serenity-rs/serenity/commit/826220f351a688b2a6f1c6ec527e65a996861d22
[c:8bec4af]: https://github.com/serenity-rs/serenity/commit/8bec4af635c3e50b111d19f6c20d56eafbb81193

[c:04b410e]: https://github.com/serenity-rs/serenity/commit/04b410ee75b2eb29f32e66fc137d3992a4972f1d
[c:3a58090]: https://github.com/serenity-rs/serenity/commit/3a580909c489c328f3faa10741debd4b063e7fbd
[c:d1266fc]: https://github.com/serenity-rs/serenity/commit/d1266fc3051a436f87a4778c5081c2228eb50b1c

[c:01e3c33]: https://github.com/serenity-rs/serenity/commit/01e3c331ed188e2b95bafa2fa0fc63d5c0c03905
[c:02de778]: https://github.com/serenity-rs/serenity/commit/02de7789d72141434264e8bd7cee7e1fc65a043f
[c:0501020]: https://github.com/serenity-rs/serenity/commit/05010204eaded91b29aef0561fc8fb668b522760
[c:0d55363]: https://github.com/serenity-rs/serenity/commit/0d553630c1a9da216e42e7c0a9bedaccfedf678d
[c:12d5321]: https://github.com/serenity-rs/serenity/commit/12d53214f39211a4c02026d9389b9aa2bfa8a5ee
[c:1de3937]: https://github.com/serenity-rs/serenity/commit/1de39377a2e428f9652d887627f420349337c5b1
[c:2179623]: https://github.com/serenity-rs/serenity/commit/2179623ebf12f7d8e16cc87e193ecd4de0f7b1fe
[c:21eb42f]: https://github.com/serenity-rs/serenity/commit/21eb42f96f9721d4e004dbc70aedf60e6d1ae7c4
[c:2a6c3b1]: https://github.com/serenity-rs/serenity/commit/2a6c3b1d1e24ec7dc3b1f19baf87594e362ded27
[c:4648f58]: https://github.com/serenity-rs/serenity/commit/4648f58e8ddc878d06a5a4a1d2840180c359ddf0
[c:602c5a7]: https://github.com/serenity-rs/serenity/commit/602c5a7b78dda42b9c3d5426c39099d48e74bca5
[c:73ab20f]: https://github.com/serenity-rs/serenity/commit/73ab20f271c9cc6dadb7bb76938ae64d19cee71e
[c:7a93557]: https://github.com/serenity-rs/serenity/commit/7a935574ffe0b7d19c1ed5c5befe1b7e7e4f0e0d
[c:8301333]: https://github.com/serenity-rs/serenity/commit/830133377a5832784c311302e543f86f85194e3b
[c:869fff5]: https://github.com/serenity-rs/serenity/commit/869fff566ca7a3669f7f08461a6bd481af3649d3
[c:8918201]: https://github.com/serenity-rs/serenity/commit/891820102ff7b9025c67e03ac59f5ecd75959aac
[c:8c0e5a3]: https://github.com/serenity-rs/serenity/commit/8c0e5a377ad7db3c40e37740123c0ebf3d7e36ae
[c:8f128b2]: https://github.com/serenity-rs/serenity/commit/8f128b2c041d5f708378082af3653ff1ee2df919
[c:90c7ec4]: https://github.com/serenity-rs/serenity/commit/90c7ec45d6cc01b25296de9619b7d3a6288244fe
[c:9568e3b]: https://github.com/serenity-rs/serenity/commit/9568e3b24816bb180740789d1e30c29f3658dc8b
[c:9a863bd]: https://github.com/serenity-rs/serenity/commit/9a863bd78e8edc5849e56e979888f1191b1d5845
[c:a0b0dd2]: https://github.com/serenity-rs/serenity/commit/a0b0dd226f9ad2476729fa79dbc680bd08aa44b3
[c:a4c3fec]: https://github.com/serenity-rs/serenity/commit/a4c3fec493d3b85ad1b43f3a5c4927d0d5cdc717
[c:aa437d4]: https://github.com/serenity-rs/serenity/commit/aa437d4dbc4a59ffa65f80c7eafa6efc37eedc86
[c:b324774]: https://github.com/serenity-rs/serenity/commit/b3247749f745c524b1eb0f44118c8358868e722a
[c:bbbf638]: https://github.com/serenity-rs/serenity/commit/bbbf63868a8ef3c0f21c1896f7afb96f4d8fbcc1
[c:c458099]: https://github.com/serenity-rs/serenity/commit/c45809973f9ed333d9c13905a376af14a73d920b
[c:db21036]: https://github.com/serenity-rs/serenity/commit/db210367f3752d8e8ad018742ea0b590ddc54009
[c:e1332a5]: https://github.com/serenity-rs/serenity/commit/e1332a54af46eff6051097ff4989c8d0fde4ca37
[c:e2873c8]: https://github.com/serenity-rs/serenity/commit/e2873c820c1134ea7cc4cfbe99467aac350fa892
[c:e290b03]: https://github.com/serenity-rs/serenity/commit/e290b038242cec6d4465f96c22cff24578f1a068
[c:e5ea6c1]: https://github.com/serenity-rs/serenity/commit/e5ea6c176ba96988efc612a8e14eea90f9c293e1
[c:f064d65]: https://github.com/serenity-rs/serenity/commit/f064d65486d0c8a3c510ee398e7d0bbf6b283bdb
[c:f3f22d7]: https://github.com/serenity-rs/serenity/commit/f3f22d7e072477028c9853d467dd18cf50e1589f

[c:0067c33]: https://github.com/serenity-rs/serenity/commit/0067c3335929325f54a3a0fe3693703e16de219c
[c:04b0be1]: https://github.com/serenity-rs/serenity/commit/04b0be18b101186d618f9593fc8d2569ee845487
[c:0d6e019]: https://github.com/serenity-rs/serenity/commit/0d6e019c258a8f2e743bcab196acab50b01e3958
[c:10bbffe]: https://github.com/serenity-rs/serenity/commit/10bbffe9332edf8b8835d98cfffb8ec411162145
[c:10f7548]: https://github.com/serenity-rs/serenity/commit/10f7548d4d57864b599dd7a760d2609144a2ec63
[c:1ec1086]: https://github.com/serenity-rs/serenity/commit/1ec1086026971c903858128a8d38c5143f3f0f6f
[c:1f3a57e]: https://github.com/serenity-rs/serenity/commit/1f3a57eb6c0a1419614927d52bd3e798db36b043
[c:29480e5]: https://github.com/serenity-rs/serenity/commit/29480e5eeccc12afc0e9020373647786736aabc7
[c:2ef660e]: https://github.com/serenity-rs/serenity/commit/2ef660e34c4cca96ec30049e42c79e899c573be0
[c:2ff765b]: https://github.com/serenity-rs/serenity/commit/2ff765bbe74e2dc36a6c0c221c7ab06aac74462a
[c:305d200]: https://github.com/serenity-rs/serenity/commit/305d2008216b5351d9fdd357381027ea42f4740b
[c:3121f90]: https://github.com/serenity-rs/serenity/commit/3121f90a9f98e82fab48d62cf95cd316ae9f0496
[c:3a647e3]: https://github.com/serenity-rs/serenity/commit/3a647e3b7f6762fa6a078bc539e5b3e8012b37d4
[c:40c8248]: https://github.com/serenity-rs/serenity/commit/40c8248d107b3c6cad785502e6d619669aba6431
[c:4cf83d0]: https://github.com/serenity-rs/serenity/commit/4cf83d0d6b2a4fe156d3c54c06db4ce32293efb0
[c:4e4dcb1]: https://github.com/serenity-rs/serenity/commit/4e4dcb11586520f798c831956dc42778c0205386
[c:530ea76]: https://github.com/serenity-rs/serenity/commit/530ea76cfd05ffa64a826e6afa342860c730fd00
[c:55555b8]: https://github.com/serenity-rs/serenity/commit/55555b88dd44366e27d2c7cc02166995a3835a69
[c:5abc7d1]: https://github.com/serenity-rs/serenity/commit/5abc7d1d7fe7130e73e4848c6333627d9881cb9e
[c:5dab87b]: https://github.com/serenity-rs/serenity/commit/5dab87b0ff0097eb78abc1089c6a51ea05aa2273
[c:5b66ace]: https://github.com/serenity-rs/serenity/commit/5b66ace77b55c3d7272aab9b49db919c180ec33f
[c:5ffdcea]: https://github.com/serenity-rs/serenity/commit/5ffdceafcbc75947365004107e640783ec033335
[c:614402f]: https://github.com/serenity-rs/serenity/commit/614402f7b963a713bfa98bc5b1cfa968e8d6c103
[c:6ddfef8]: https://github.com/serenity-rs/serenity/commit/6ddfef8359a619be9a49be7b33b466724eed0ecb
[c:703d135]: https://github.com/serenity-rs/serenity/commit/703d13564f9081839eb77e4e4699d711b1de895a
[c:7937025]: https://github.com/serenity-rs/serenity/commit/7937025a484955cc8d74fb10004ba8b49dcc2bb0
[c:7b9764c]: https://github.com/serenity-rs/serenity/commit/7b9764cf1097b0620d871fabe67b5593f0cd4a4a
[c:7eac4d5]: https://github.com/serenity-rs/serenity/commit/7eac4d5fcf6c16db64e118de3d69825909979d5b
[c:8114a7a]: https://github.com/serenity-rs/serenity/commit/8114a7ace3ad51b9903a6017993aa526742bd72d
[c:8aefde0]: https://github.com/serenity-rs/serenity/commit/8aefde08465a050ad7bae12e6003fe514f43af5f
[c:8ce8234]: https://github.com/serenity-rs/serenity/commit/8ce82346846f235357b8dc53cb3ff399e70fcb4a
[c:93f453b]: https://github.com/serenity-rs/serenity/commit/93f453b07b9e8f813e6bfb0ddd2648a8e626d136
[c:9b2cd75]: https://github.com/serenity-rs/serenity/commit/9b2cd75baf1fa7ee063f47e966ee3f6566a6d45c
[c:9da7669]: https://github.com/serenity-rs/serenity/commit/9da766976929417c4b8f487f8ec05b6f8b3f43ef
[c:9e45642]: https://github.com/serenity-rs/serenity/commit/9e456427ccd496c4128bde841df0c0af7a262047
[c:9e56062]: https://github.com/serenity-rs/serenity/commit/9e560628deb1cf66e0c5029f41a79404fadffb40
[c:a9a2c27]: https://github.com/serenity-rs/serenity/commit/a9a2c27d7aefa6061dd9ca58a96c5ba617a78a6a
[c:a9e8626]: https://github.com/serenity-rs/serenity/commit/a9e8626c4cd642087f828c5b32481bee9e4d368b
[c:aeb89af]: https://github.com/serenity-rs/serenity/commit/aeb89af4eff59bb3ea9eb7623685bf7ad7520496
[c:b520ec7]: https://github.com/serenity-rs/serenity/commit/b520ec708c375e09838b9f25fd285790b856bb97
[c:bbfc8e2]: https://github.com/serenity-rs/serenity/commit/bbfc8e2d0250f41d5bf4230b6efb428419133de8
[c:bd4aa0a]: https://github.com/serenity-rs/serenity/commit/bd4aa0aabda4a2986e6145e3a793e8b2a391f8dd
[c:caeab28]: https://github.com/serenity-rs/serenity/commit/caeab28059d029a92b784f3b5ae1f79c412c8404
[c:ccd2506]: https://github.com/serenity-rs/serenity/commit/ccd250649665b1726b0ca852b2375c113da6ed57
[c:ce8da79]: https://github.com/serenity-rs/serenity/commit/ce8da793d3142cb001d9b155ff4224c15fe833ce
[c:d0d363f]: https://github.com/serenity-rs/serenity/commit/d0d363fb2a3475c68d40b02ec22ab728059fd55e
[c:d11d916]: https://github.com/serenity-rs/serenity/commit/d11d916a94b8a96fde218db4550d6c2428b4bc2a
[c:dd3744b]: https://github.com/serenity-rs/serenity/commit/dd3744b08887debba0d44fd0bceddef5f8ed1356
[c:e602630]: https://github.com/serenity-rs/serenity/commit/e6026308b33c80aa33f0001c89cd271cc5cb6687
[c:eae624e]: https://github.com/serenity-rs/serenity/commit/eae624e3f18681971a654c95624d917afe00695a
[c:f09b661]: https://github.com/serenity-rs/serenity/commit/f09b661be9085c7525a6c9f6929b50deebffae9b
[c:f0f06b7]: https://github.com/serenity-rs/serenity/commit/f0f06b7d3b890d2ddcb84e00b3f62e195da80090

[c:0324e01]: https://github.com/serenity-rs/serenity/commit/0324e011f1ea0eed0709c92fe86319c812a42206
[c:08a7110]: https://github.com/serenity-rs/serenity/commit/08a71106748e356d2618e48d8797e6da60d7eb54
[c:0e1e8fb]: https://github.com/serenity-rs/serenity/commit/0e1e8fbbe564c23530a709a7ec407b08f63944e2
[c:1162e68]: https://github.com/serenity-rs/serenity/commit/1162e686592f23f4dc5ad509051858e453c82d1f
[c:152fe3d]: https://github.com/serenity-rs/serenity/commit/152fe3ded89c71580a9ab9d3bb05587abee97e72
[c:23c5398]: https://github.com/serenity-rs/serenity/commit/23c5398d8c6b0a3e5ad28cb43fadd48002195d3c
[c:2603063]: https://github.com/serenity-rs/serenity/commit/26030630bee7750c047b155708a62a03a6a5edf3
[c:32c3bed]: https://github.com/serenity-rs/serenity/commit/32c3bed1afa65d14a93d4e3d4e9e8471cfd77ced
[c:457a17e]: https://github.com/serenity-rs/serenity/commit/457a17e059395aab3d1a23bd1cfe6e01ea0b5a61
[c:4a24c90]: https://github.com/serenity-rs/serenity/commit/4a24c9004a1aac31eb552e5cdef6e986a6e903b3
[c:5ba521b]: https://github.com/serenity-rs/serenity/commit/5ba521b4e8bb5ff10bd83303436cd454f27c93ab
[c:6346975]: https://github.com/serenity-rs/serenity/commit/63469753a960431e5edacf520589493121f0e62d
[c:89a18aa]: https://github.com/serenity-rs/serenity/commit/89a18aa919d8c08cf9fba9a98ebe32c9fd59d5d4
[c:63fe032]: https://github.com/serenity-rs/serenity/commit/63fe032eaa29bbee51b6efd3f19392cc41b992e0
[c:a80aab2]: https://github.com/serenity-rs/serenity/commit/a80aab264b3d81c2457eb6bef755a5b0d1a88e4b
[c:af7f176]: https://github.com/serenity-rs/serenity/commit/af7f176101aea9bcf43551fbcd3261469bbc0b43
[c:c659bbd]: https://github.com/serenity-rs/serenity/commit/c659bbd756391fc26e9e862937ef77113ee892ed
[c:cc6b567]: https://github.com/serenity-rs/serenity/commit/cc6b5677d8e5ba61a53f7aeae26b66207db9fd3d
[c:ff9edc0]: https://github.com/serenity-rs/serenity/commit/ff9edc063fd987b9ba14c8d211ed26f6b480f751

[c:36d7a54]: https://github.com/serenity-rs/serenity/commit/36d7a541ec53051007fee74f621919bea721c8f2
[c:40db3c0]: https://github.com/serenity-rs/serenity/commit/40db3c024dd5e13c5a02e10ab4f7a7e9c31a6876
[c:42063a2]: https://github.com/serenity-rs/serenity/commit/42063a240682f0cfa93b7dce9fcb79b2dfe7ef99
[c:526c366]: https://github.com/serenity-rs/serenity/commit/526c366eb355ff2cdfd5769621448d35926fe123
[c:64dcced]: https://github.com/serenity-rs/serenity/commit/64dccedc999b6f2cdf4b60830d4c5fb1126bd37c
[c:7b6b601]: https://github.com/serenity-rs/serenity/commit/7b6b6016078c3492d2873d3eed0ddb39323079ab
[c:7f9c01e]: https://github.com/serenity-rs/serenity/commit/7f9c01e4e4d413979e2e66eea1d3cdf9157c4dc5
[c:83a0c85]: https://github.com/serenity-rs/serenity/commit/83a0c85b0bf87cb4272b5d6e189d139fc31a6d23
[c:a481df6]: https://github.com/serenity-rs/serenity/commit/a481df6e67d83216617a40d07991ba8ea04d0075
[c:e546fa2]: https://github.com/serenity-rs/serenity/commit/e546fa2a32a05a9bbc351b9aa789233ee71e88f0
[c:fd77a91]: https://github.com/serenity-rs/serenity/commit/fd77a91f2ba7c782f3e0e67ecee19df17bb31e26

[c:003dc2e]: https://github.com/serenity-rs/serenity/commit/003dc2eed0f09cd214373f1581b7794d1483a689
[c:02dc506]: https://github.com/serenity-rs/serenity/commit/02dc5064d9402f73ef514c9b8ffa318f5d4235ff
[c:0d779ba]: https://github.com/serenity-rs/serenity/commit/0d779ba85ad43eb7e95be3844ad3fcd74335b47c
[c:21fe999]: https://github.com/serenity-rs/serenity/commit/21fe999d23cb0e4e76812537b48edadeab5a1540
[c:217e1c6]: https://github.com/serenity-rs/serenity/commit/217e1c6ebd9577fa5a538b4ecd3c1303ee034462
[c:24d2233]: https://github.com/serenity-rs/serenity/commit/24d2233ba583221b35eca02cf321e7a4a1adf76d
[c:2791ed7]: https://github.com/serenity-rs/serenity/commit/2791ed78df18f2721264352611b1ba26b3077196
[c:2937792]: https://github.com/serenity-rs/serenity/commit/29377922d3d79848efcb8d3bd0fbd52c21e81c5d
[c:2ab714f]: https://github.com/serenity-rs/serenity/commit/2ab714f9c3683db83eba1f6fe7bb3887a47d4f3f
[c:2e1eb4c]: https://github.com/serenity-rs/serenity/commit/2e1eb4c35b488ac68a33e76502d0c8f56a72c4b6
[c:3d67a4e]: https://github.com/serenity-rs/serenity/commit/3d67a4e2cd33b17747c7499e07d0a0e05fe73253
[c:5166a0a]: https://github.com/serenity-rs/serenity/commit/5166a0a81bac494b0e337d00f946a85d65e4bbbd
[c:77c399b]: https://github.com/serenity-rs/serenity/commit/77c399ba7b3bff0cf3180df5edad5d6ff6dcb10d
[c:7d162b9]: https://github.com/serenity-rs/serenity/commit/7d162b96686a56eed052984c18f22f54ad76f780
[c:7e0d908]: https://github.com/serenity-rs/serenity/commit/7e0d9082fe0262c7b4c4668ca1e1208dffa4796f
[c:7f09642]: https://github.com/serenity-rs/serenity/commit/7f09642fa66517fc983f63bb2e414638382a6512
[c:80dfcb0]: https://github.com/serenity-rs/serenity/commit/80dfcb0539c88e9434dc4875d5af55df1aafa725
[c:82e21a6]: https://github.com/serenity-rs/serenity/commit/82e21a61ae17d8466834f486108d1ace2791efc4
[c:9baf167]: https://github.com/serenity-rs/serenity/commit/9baf1675b0d1837fe010cfbadac8e80fd9d53898
[c:a4cc582]: https://github.com/serenity-rs/serenity/commit/a4cc5821c0ca194d321141985cfaf30241f54acf
[c:b71d99f]: https://github.com/serenity-rs/serenity/commit/b71d99fde84135fa66f73c4817d340ffbe8bddae
[c:b9fa745]: https://github.com/serenity-rs/serenity/commit/b9fa7457c2011130b28f5eca063f88bdf72be544
[c:bd195de]: https://github.com/serenity-rs/serenity/commit/bd195dea4422f516b727868209116ff868e3941b
[c:beebff5]: https://github.com/serenity-rs/serenity/commit/beebff50eb4f447f41162299fde81cb6fa9b336d
[c:c6a5fe4]: https://github.com/serenity-rs/serenity/commit/c6a5fe4ef9dc07b9b14f742440a35ba6ea02058b
[c:d0ae9bb]: https://github.com/serenity-rs/serenity/commit/d0ae9bba89d5e66129da7bed7faf39abfd4fb17d
[c:d8c9d89]: https://github.com/serenity-rs/serenity/commit/d8c9d89c80ffb0ae95b91c4dcc593fd70fa976d8
[c:dbfc06e]: https://github.com/serenity-rs/serenity/commit/dbfc06e9c6df506839fb178eaeb9db704aefd357
[c:e506e9f]: https://github.com/serenity-rs/serenity/commit/e506e9f30584d0fd67bf28a33b1033c4f1e5cd8b
[c:e5bcee7]: https://github.com/serenity-rs/serenity/commit/e5bcee7b2c2d2ff873982c6a3bf39ab16ec4e1e3
[c:e814e9a]: https://github.com/serenity-rs/serenity/commit/e814e9aa9b6117defa4f885ef0027e61706112d5
[c:f115c17]: https://github.com/serenity-rs/serenity/commit/f115c177def1a35fc532c896713a187bb468088e
[c:fdcf44e]: https://github.com/serenity-rs/serenity/commit/fdcf44e1463e708cd8b612c183e302db9af0febd
[c:ffc5ea1]: https://github.com/serenity-rs/serenity/commit/ffc5ea1da38cb7d9c302fa8d5614253c1f361311

[c:03a7e3e]: https://github.com/serenity-rs/serenity/commit/03a7e3e1d82ca667ca065367d2cf21b847f984ac
[c:13b0de1]: https://github.com/serenity-rs/serenity/commit/13b0de121eda30e59849fd442c8a0926a63df2b8
[c:27c83e8]: https://github.com/serenity-rs/serenity/commit/27c83e8ef0def0a62e8a5ce5bfd4849892749c83
[c:324a288]: https://github.com/serenity-rs/serenity/commit/324a288fbb0dd7d135aa9aab876cf39dabb6a02e
[c:5a0b8a6]: https://github.com/serenity-rs/serenity/commit/5a0b8a68c133c3093260a5aeb08b02eb3595c18d
[c:55fa37a]: https://github.com/serenity-rs/serenity/commit/55fa37ade187aa68ef3eec519d22767920aae4ab
[c:76bcf7d]: https://github.com/serenity-rs/serenity/commit/76bcf7dcef91fd2658fb3348acf6df0ecc33fcdf
[c:7912f23]: https://github.com/serenity-rs/serenity/commit/7912f23bed7ddc540c46aee0ecd64c6b60daa0f4
[c:8578d5f]: https://github.com/serenity-rs/serenity/commit/8578d5fe6e3bdc2842cda9417c242169f93b1a99
[c:92c91b8]: https://github.com/serenity-rs/serenity/commit/92c91b81490b621de4519e0d87830dbce53dd689
[c:ab1f11a]: https://github.com/serenity-rs/serenity/commit/ab1f11a37d64166c08f833042d7b3bcde2ea586d
[c:aba1ba6]: https://github.com/serenity-rs/serenity/commit/aba1ba67dc78a0c14e5de3c8ac650829e436e96f
[c:b9a7e50]: https://github.com/serenity-rs/serenity/commit/b9a7e50579718a20e60a19f0c0d410661ee3e77a
[c:ce2952a]: https://github.com/serenity-rs/serenity/commit/ce2952ad0d783b4a256171e48602c6caf1125c61
[c:d193975]: https://github.com/serenity-rs/serenity/commit/d1939756f6bf4b4bb3c60fbb81a397c218492d62
[c:d240074]: https://github.com/serenity-rs/serenity/commit/d2400742f657d9f8c432a440810d49e63339f5aa
[c:e4612ac]: https://github.com/serenity-rs/serenity/commit/e4612acf58dc42fdc32094426c14274bd61203dd
[c:eee3168]: https://github.com/serenity-rs/serenity/commit/eee3168b4ed266001571abe4e1a6ae4ef06b93e0

[c:031fc92]: https://github.com/serenity-rs/serenity/commit/031fc92e02c314cce9fc8febcc7900fa2d885941
[c:032c5a7]: https://github.com/serenity-rs/serenity/commit/032c5a75620c3ff185a749113d93fb3051b38acb
[c:0525ede]: https://github.com/serenity-rs/serenity/commit/0525ede086ccffa5781c9a1876a368ac3531813e
[c:05f6ed4]: https://github.com/serenity-rs/serenity/commit/05f6ed429aeac1920307ea692fef122bbd2dffff
[c:0881e18]: https://github.com/serenity-rs/serenity/commit/0881e18c07113cc7b2f6cec38cadcb1ea03dda12
[c:08db9fa]: https://github.com/serenity-rs/serenity/commit/08db9fa2adef141743ab9681c46dd91489278063
[c:08febb0]: https://github.com/serenity-rs/serenity/commit/08febb0d8d95bbbae9861130a756e842eae40eef
[c:0aa55a2]: https://github.com/serenity-rs/serenity/commit/0aa55a2b9b757321d5b8bb9e512813aa9d0a62ca
[c:0b1f684]: https://github.com/serenity-rs/serenity/commit/0b1f684106031657d6bf581206c06e5443d06da9
[c:10c56a9]: https://github.com/serenity-rs/serenity/commit/10c56a9385c0d410241d33525f8f49242daced2d
[c:114e43a]: https://github.com/serenity-rs/serenity/commit/114e43a3d0079072593588ad7b2de9f8588e041d
[c:12317b9]: https://github.com/serenity-rs/serenity/commit/12317b98a0cc145e717e9da3cdbe8bc1ff8fc4f1
[c:141bbfc]: https://github.com/serenity-rs/serenity/commit/141bbfcb1e4843eaeb55bf07e10e2c0aa4bbe1e4
[c:143fddd]: https://github.com/serenity-rs/serenity/commit/143fddd83f1fc93c070e36bf31906d2631c68f97
[c:14b9222]: https://github.com/serenity-rs/serenity/commit/14b92221184fcaca0f4a60a3b31d5a9470b14b1f
[c:1735e57]: https://github.com/serenity-rs/serenity/commit/1735e57ea57bcd4d75b73ac9398e13bee5198c5b
[c:1fa83f7]: https://github.com/serenity-rs/serenity/commit/1fa83f73577d926664518d849bc26e46087611b4
[c:1fd652b]: https://github.com/serenity-rs/serenity/commit/1fd652be41f4de96d26efaf20055cf7a80e42bf1
[c:2032a40]: https://github.com/serenity-rs/serenity/commit/2032a402c387b1310f2ae62621f3e07c86b76aef
[c:222382c]: https://github.com/serenity-rs/serenity/commit/222382ca48cb9786aaf5d0b5fc16958e482e7c5f
[c:25d79ac]: https://github.com/serenity-rs/serenity/commit/25d79ac7e07654dbf77166d46db33d186faf9885
[c:25dddb6]: https://github.com/serenity-rs/serenity/commit/25dddb6695109eeead9e19593cb85a22096c2c7a
[c:26fe139]: https://github.com/serenity-rs/serenity/commit/26fe139363a847542bbe609fe4d15accbf4fef14
[c:2abeea5]: https://github.com/serenity-rs/serenity/commit/2abeea53745b5ddfce33d9e1160c682888850344
[c:2c9b682]: https://github.com/serenity-rs/serenity/commit/2c9b6824a7bf6231a08d5c5465c1db5417ea5d8a
[c:2d23d8b]: https://github.com/serenity-rs/serenity/commit/2d23d8b50386e38fece6987286bd0b3d56d1cada
[c:2edba81]: https://github.com/serenity-rs/serenity/commit/2edba816f6901db46e7fc0f6664058033a56d5e7
[c:39a1435]: https://github.com/serenity-rs/serenity/commit/39a1435be57335e99039ddea731032221eb6d96e
[c:3a0c890]: https://github.com/serenity-rs/serenity/commit/3a0c8908ce837f6fe64f865a1a7a9de63cbd237c
[c:3a4cb18]: https://github.com/serenity-rs/serenity/commit/3a4cb18be8ca33d507cbfc88fec79b2a6c5d8bfc
[c:3b9f0f8]: https://github.com/serenity-rs/serenity/commit/3b9f0f8501f7581d710e3f7ebbfcd3176d14a9b1
[c:3ca7e15]: https://github.com/serenity-rs/serenity/commit/3ca7e15e55de640200edb3898a33b838946a506c
[c:3d24033]: https://github.com/serenity-rs/serenity/commit/3d24033f550623f78ad71a37f6ec847e7d0a2c78
[c:3e14067]: https://github.com/serenity-rs/serenity/commit/3e1406764cf655694fef0e04e43324d58499bba3
[c:40c5c12]: https://github.com/serenity-rs/serenity/commit/40c5c12373e2a2c7acd3501f43c79f9bf3e7c685
[c:45c1f27]: https://github.com/serenity-rs/serenity/commit/45c1f27edbeedcb30aa3e9daa78eba44817f7260
[c:470f366]: https://github.com/serenity-rs/serenity/commit/470f366000b3d3f8080e02b185f0f7fef592a736
[c:4bd223a]: https://github.com/serenity-rs/serenity/commit/4bd223a88cfacc335814ef3ddc0c1aaa88fc05f7
[c:4e20277]: https://github.com/serenity-rs/serenity/commit/4e20277de4f164705074ba41199e4530332056b3
[c:524b8f8]: https://github.com/serenity-rs/serenity/commit/524b8f8ab5153e20ad86be2df7fba6bbed159b7c
[c:551f166]: https://github.com/serenity-rs/serenity/commit/551f16673fe775a80a1da788fd7e1db20f6eae29
[c:5d4301b]: https://github.com/serenity-rs/serenity/commit/5d4301bbd2aaa4abe47fbbc2a7a2853ba9b728f2
[c:60613ef]: https://github.com/serenity-rs/serenity/commit/60613ef696b093dbbac3a4e9e033c226c5d358ea
[c:612e973]: https://github.com/serenity-rs/serenity/commit/612e973f286ba6b711824333551b07b88df6740c
[c:62647f5]: https://github.com/serenity-rs/serenity/commit/62647f53fd01a670cf5ad01c7d0a68cb69bf92cf
[c:65233ad]: https://github.com/serenity-rs/serenity/commit/65233ad6f3d002f72942aaf811514fa9d29ad068
[c:65e3279]: https://github.com/serenity-rs/serenity/commit/65e3279ce7b3c4807e8b1310551e9493d3868b94
[c:68156c9]: https://github.com/serenity-rs/serenity/commit/68156c9ce93edc86a70f50cf10986615cfb9f93a
[c:7566f32]: https://github.com/serenity-rs/serenity/commit/7566f32c194bc4e62e89adc57bfb6104cc99458e
[c:78c6df9]: https://github.com/serenity-rs/serenity/commit/78c6df9ed3640c097ef088519ec99a6a01796bfc
[c:7a5aa3c]: https://github.com/serenity-rs/serenity/commit/7a5aa3c57951ee5c7267fabf38f2729b06629b34
[c:7c911d5]: https://github.com/serenity-rs/serenity/commit/7c911d57eb3db3ac51cfc51cf9b3a5884e0f4ea3
[c:7cf1e52]: https://github.com/serenity-rs/serenity/commit/7cf1e523f0c0bee1b7ec11ff6e6565c68f9d173e
[c:7e46d8f]: https://github.com/serenity-rs/serenity/commit/7e46d8f3ac5a968df9a05f8f0006522ad14891ef
[c:82b87f1]: https://github.com/serenity-rs/serenity/commit/82b87f196425ff8553bc9dcb84ddac9764b971e4
[c:84706f1]: https://github.com/serenity-rs/serenity/commit/84706f1fc0a934a851d57f524697da5b177b9be8
[c:84ff27b]: https://github.com/serenity-rs/serenity/commit/84ff27be8455d9ec885b190150a2b592cffdf2a2
[c:85d7d5f]: https://github.com/serenity-rs/serenity/commit/85d7d5f6a6df9841659bc7ad8e392f31c1aae46c
[c:8c83866]: https://github.com/serenity-rs/serenity/commit/8c83866748bf7bf339df9a234c3297c8008ffa46
[c:8c85664]: https://github.com/serenity-rs/serenity/commit/8c85664a94f7439ab4bc3a132f313a9e26d94fe7
[c:8c9baa7]: https://github.com/serenity-rs/serenity/commit/8c9baa74c2716d62c405d909bb453ffea636c94d
[c:8d68503]: https://github.com/serenity-rs/serenity/commit/8d685039d89fd2130e88c9a9e0421492a3e99da6
[c:91c8ec4]: https://github.com/serenity-rs/serenity/commit/91c8ec4ae7540956a714ce9584074538b45467cc
[c:9232b8f]: https://github.com/serenity-rs/serenity/commit/9232b8f065deb4637a74e7f85ab617bb527c51be
[c:934eb3a]: https://github.com/serenity-rs/serenity/commit/934eb3aa0b1f9c0aaad003627bd65932114654c1
[c:93e0a42]: https://github.com/serenity-rs/serenity/commit/93e0a4215c915b98cf433ac6d0bcfbc60f0168ec
[c:9908999]: https://github.com/serenity-rs/serenity/commit/9908999a6bae1585bb70b7814f13b49bf99b6c32
[c:99d17d2]: https://github.com/serenity-rs/serenity/commit/99d17d2975143b0d588c969f7ae6f8d11e62a9e1
[c:9aaa555]: https://github.com/serenity-rs/serenity/commit/9aaa55542d6bee1e953a080612ee6af765b8a5a5
[c:9aad1aa]: https://github.com/serenity-rs/serenity/commit/9aad1aa375168d6131cb6f68d6998b2af6fb00a3
[c:9da642a]: https://github.com/serenity-rs/serenity/commit/9da642a5bea8b4ac2d291058ad22e4cbe27b1f94
[c:a17fea7]: https://github.com/serenity-rs/serenity/commit/a17fea783cd91b2adcd1330b7038cf3ca2d79a85
[c:a359f77]: https://github.com/serenity-rs/serenity/commit/a359f77d1fd03def94fc08367132a616ec2ea599
[c:a7b67df]: https://github.com/serenity-rs/serenity/commit/a7b67df6d77f5acacf83710807b231866397d551
[c:a96be90]: https://github.com/serenity-rs/serenity/commit/a96be90385b58a9098b918e0fd17288d89229752
[c:aad4744]: https://github.com/serenity-rs/serenity/commit/aad4744fb751e3e1147f085781323172755d4ef2
[c:ad0dcb3]: https://github.com/serenity-rs/serenity/commit/ad0dcb305d959a2bb273a63dd2dd1b5594f5c49d
[c:ae50886]: https://github.com/serenity-rs/serenity/commit/ae50886a1a8f69c114d9e99a0913c878aaaaabe2
[c:b146501]: https://github.com/serenity-rs/serenity/commit/b14650193342297746f985f8794e4b93ceeac52b
[c:b19b031]: https://github.com/serenity-rs/serenity/commit/b19b031a5052a268f323a116403ea66ca71ea575
[c:b215457]: https://github.com/serenity-rs/serenity/commit/b215457ab46c9d10bf47300d6525f9a2641d3b17
[c:b328b3e]: https://github.com/serenity-rs/serenity/commit/b328b3e09b0095abb54530dc4d50db6b4e3e1779
[c:b52eb9f]: https://github.com/serenity-rs/serenity/commit/b52eb9f108fb7af182e0cf29259cd4d522ed7f89
[c:b60d037]: https://github.com/serenity-rs/serenity/commit/b60d0378548a53ffefda17aab403c073d3438cf6
[c:b62dfd4]: https://github.com/serenity-rs/serenity/commit/b62dfd431668b4bdb6021d21120da05d17ab77d5
[c:b7542f4]: https://github.com/serenity-rs/serenity/commit/b7542f44306fedb7f79f7b8cd5c8d6afd6ccb7ad
[c:b8efeaf]: https://github.com/serenity-rs/serenity/commit/b8efeaf5e920cbfc775cdee70f23aa41ab7b9dd5
[c:bcd16dd]: https://github.com/serenity-rs/serenity/commit/bcd16dddb8cc3086a13524c79676f3a8bebbc524
[c:be43836]: https://github.com/serenity-rs/serenity/commit/be43836839a31714f58e3ffe81dd4b0aeab2ab59
[c:c3aa63f]: https://github.com/serenity-rs/serenity/commit/c3aa63faee8b3ae6d5126aa27a74876766c61e4c
[c:d1113c0]: https://github.com/serenity-rs/serenity/commit/d1113c07f061149b5d090c1f15c3c03806f8b0cf
[c:d264cc3]: https://github.com/serenity-rs/serenity/commit/d264cc3496f56d2757cf9c1735d5d8a68577c2f5
[c:d5a9aa8]: https://github.com/serenity-rs/serenity/commit/d5a9aa8b1e0a94094ef5bda98a76dd259a6e7a3a
[c:d90b90c]: https://github.com/serenity-rs/serenity/commit/d90b90c7f3d8a368acbab46150f199866562229a
[c:e02a842]: https://github.com/serenity-rs/serenity/commit/e02a842fb76b1e591287395ac223cc1c04913820
[c:e0e7617]: https://github.com/serenity-rs/serenity/commit/e0e76173f63b6071b9df3ff8f53371b4b6c4ee1e
[c:e5a6f3a]: https://github.com/serenity-rs/serenity/commit/e5a6f3a8ed367bd3d780fd23a0a27f8a80567879
[c:e611776]: https://github.com/serenity-rs/serenity/commit/e6117760e3c020ed41d643a8b34d7bfeb62d3bfa
[c:e678883]: https://github.com/serenity-rs/serenity/commit/e6788838556d13d4a4f19253ce297ca2f72168ee
[c:e748d1f]: https://github.com/serenity-rs/serenity/commit/e748d1ff80dbbeb82b23f8ac9fec9cf8c7e4a69e
[c:eb9e8df]: https://github.com/serenity-rs/serenity/commit/eb9e8dfbc9d778de405d7369579d90c49a2bf90c
[c:ee207b3]: https://github.com/serenity-rs/serenity/commit/ee207b331d571d5afb5c35c8f119937d0196663a
[c:ee2bbca]: https://github.com/serenity-rs/serenity/commit/ee2bbcaa0b62c09a6c0df352bfddcbf99d06e61d
[c:f0a56f4]: https://github.com/serenity-rs/serenity/commit/f0a56f46ce7ef6c2a65d64d8953ac43e0b7b5b4d
[c:f0ee805]: https://github.com/serenity-rs/serenity/commit/f0ee805a8ee20b6180b9f54d5096a8a9b73b4be2
[c:f10b9d7]: https://github.com/serenity-rs/serenity/commit/f10b9d77f0b94864fa20688e3c99de6cec7ca6f9
[c:f26dad8]: https://github.com/serenity-rs/serenity/commit/f26dad86aea82070aab9cc081f50d0144ee4c778
[c:f2c21ef]: https://github.com/serenity-rs/serenity/commit/f2c21ef5b15ef1f345cdc30f4b793e55905f15f4
[c:f2fa349]: https://github.com/serenity-rs/serenity/commit/f2fa349d831c1db59993341284049ffbd56dde3b
[c:f61816c]: https://github.com/serenity-rs/serenity/commit/f61816ca141add5024e36e073764b7c824872ca4
[c:fd19446]: https://github.com/serenity-rs/serenity/commit/fd19446fcc4c7ad2c9f634c97fa1c056440a6abd

[c:52403a5]: https://github.com/serenity-rs/serenity/commit/52403a5084ed7f0589bde3351844907a92de2d62
[c:795eaa1]: https://github.com/serenity-rs/serenity/commit/795eaa15bca61116fbde9c2482c765f2d47a7696

[c:77f462e]: https://github.com/serenity-rs/serenity/commit/77f462ea2044ef7d2d12fd1289ea75a6a33cb5dd

[c:1b7101f]: https://github.com/serenity-rs/serenity/commit/1b7101fe71335c0e18bf855c0703acc23d87e427
[c:2ba4d03]: https://github.com/serenity-rs/serenity/commit/2ba4d03f15d57d9f0fb1cc4d4f4355ebbc483d0a
[c:3be6e2e]: https://github.com/serenity-rs/serenity/commit/3be6e2e28b0c3e9baaef19f405c463e3a41fed25
[c:800e58f]: https://github.com/serenity-rs/serenity/commit/800e58f4603ce99ab69569b30cbec756301a6a63
[c:c99091d]: https://github.com/serenity-rs/serenity/commit/c99091d241f240c6b76ac969655a8ec4423aaf80
[c:d3eddc6]: https://github.com/serenity-rs/serenity/commit/d3eddc68e07bbc31e2043577cbf48741f0547ed3
[c:dcac271]: https://github.com/serenity-rs/serenity/commit/dcac27168915b4f22745950ec0ef0c0af696774e
[c:e219a6a]: https://github.com/serenity-rs/serenity/commit/e219a6a9d6a890b008fc390a909ae504a0c1a329

[c:002ce3a]: https://github.com/serenity-rs/serenity/commit/002ce3aa272fa51b84e820f12db39cb87a461a83
[c:022e35d]: https://github.com/serenity-rs/serenity/commit/022e35d5b12322bd77bbe74a1a3b2ad319977390
[c:05f158f]: https://github.com/serenity-rs/serenity/commit/05f158fc89f2adc82e31cf4b93706dc7d25e11d8
[c:08d390c]: https://github.com/serenity-rs/serenity/commit/08d390c19f187986fd2856fe5cbb9035a0877e0f
[c:09a8a44]: https://github.com/serenity-rs/serenity/commit/09a8a444f5bcefaee8b83dc129a3cea2de8792f9
[c:0d1c0f1]: https://github.com/serenity-rs/serenity/commit/0d1c0f1356fd3a891232498c2230d0bb4d2ed4ff
[c:0df77b9]: https://github.com/serenity-rs/serenity/commit/0df77b933ff5e98725252116069afad2dec9f89b
[c:0ed1972]: https://github.com/serenity-rs/serenity/commit/0ed19727debf28a8aa0818b44713090e97dd6eee
[c:11b85ca]: https://github.com/serenity-rs/serenity/commit/11b85ca6799b9984481119851f983d8e3c84cdc0
[c:1b167b5]: https://github.com/serenity-rs/serenity/commit/1b167b5496ce816cbcacb0e4f6e63399dffaa25c
[c:1bf4d9c]: https://github.com/serenity-rs/serenity/commit/1bf4d9cb9823dca8c4bb77147c66eac2d53f609f
[c:1d4ecb2]: https://github.com/serenity-rs/serenity/commit/1d4ecb2f13258d286378c44d59c2ee4b1c68349d
[c:21e194b]: https://github.com/serenity-rs/serenity/commit/21e194bffc37f396f007d390170f5b60e22f5d02
[c:3b2c246]: https://github.com/serenity-rs/serenity/commit/3b2c2462cb34b5ae5190ebc4a9e04968dc8d5335
[c:483b069]: https://github.com/serenity-rs/serenity/commit/483b069cc0c821ec673ac475b168809e3a41525a
[c:55167c3]: https://github.com/serenity-rs/serenity/commit/55167c300598536a852b3596fcf1c420aeb96c3a
[c:683691f]: https://github.com/serenity-rs/serenity/commit/683691f762bbf58e3abf3bc67381e18112f5c8ad
[c:6b9dcf5]: https://github.com/serenity-rs/serenity/commit/6b9dcf5272458499c1caef544cb82d5a8624258b
[c:71f709d]: https://github.com/serenity-rs/serenity/commit/71f709d0aceedb6d3091d0c28c9535e281270f71
[c:7945094]: https://github.com/serenity-rs/serenity/commit/794509421f21bee528e582a7b109d6a99284510a
[c:7befcd5]: https://github.com/serenity-rs/serenity/commit/7befcd5caa9ccdf44d90ecc12014c335b1bd2be7
[c:8109619]: https://github.com/serenity-rs/serenity/commit/8109619184867fc843a1e73d18d37726a34f7fbf
[c:8565fa2]: https://github.com/serenity-rs/serenity/commit/8565fa2cb356cf8cbccfeb09828c9d136ad3d614
[c:8572943]: https://github.com/serenity-rs/serenity/commit/857294358d5f3029850dc79c174b831c0b0c161c
[c:86d8bdd]: https://github.com/serenity-rs/serenity/commit/86d8bddff3e3242186d0c2607b34771e5422ba5b
[c:917dd30]: https://github.com/serenity-rs/serenity/commit/917dd3071dc8a145b9c379cb3a8a84731c690340
[c:9b0c053]: https://github.com/serenity-rs/serenity/commit/9b0c053725e04c60eb7ddcfeb847be4189b3dbf6
[c:b3aa441]: https://github.com/serenity-rs/serenity/commit/b3aa441c2d61ba324396deaf70f2c5818fd3f528
[c:c98cae4]: https://github.com/serenity-rs/serenity/commit/c98cae4e838147eaa077bbc68ffebf8834ff7b6b
[c:cf40386]: https://github.com/serenity-rs/serenity/commit/cf403867400110f446720fc20fad6781cf8c6b13
[c:d7621aa]: https://github.com/serenity-rs/serenity/commit/d7621aa4dfb2a3dea22e7848eb97e2b4cc1ade14

[c:005437f]: https://github.com/serenity-rs/serenity/commit/005437f56869e846ff677b6516605def0c4de7bc
[c:0186754]: https://github.com/serenity-rs/serenity/commit/01867549709ef73ee09ed442e1d5ea938fd7f74d
[c:0240717]: https://github.com/serenity-rs/serenity/commit/02407175e463b2b75295364d6b0e182fe34966ed
[c:03b6d78]: https://github.com/serenity-rs/serenity/commit/03b6d78885b3a59ffa781ded3682c2dd24e65aa7
[c:05162aa]: https://github.com/serenity-rs/serenity/commit/05162aa18aa737c05fbc13917fed1c8c218064d5
[c:051d23d]: https://github.com/serenity-rs/serenity/commit/051d23d60d4898d331d046861035165bf2e6cd23
[c:069df4f]: https://github.com/serenity-rs/serenity/commit/069df4f85d8c462df58c1fce00595462f2825337
[c:078947e]: https://github.com/serenity-rs/serenity/commit/078947edc2b7036b2a0b49afc3cc54b12a39af18
[c:0810ab7]: https://github.com/serenity-rs/serenity/commit/0810ab7a6aa37ca684b10c22dde8f0e03d3f8ea2
[c:092f288]: https://github.com/serenity-rs/serenity/commit/092f288fdd22ae39b019e61a6f12420b6ca3b67c
[c:0d6965f]: https://github.com/serenity-rs/serenity/commit/0d6965f647396c84b2570e92b63244c3afaea863
[c:106a4d5]: https://github.com/serenity-rs/serenity/commit/106a4d5f8ff22a829a9486ce88fa8326184828fa
[c:125c1b8]: https://github.com/serenity-rs/serenity/commit/125c1b8feff65ed86136ca0c3b75cdfa073aefc3
[c:14fd41b]: https://github.com/serenity-rs/serenity/commit/14fd41b0d62ab441b6600028792641d813f09cd8
[c:16a5828]: https://github.com/serenity-rs/serenity/commit/16a5828394c21baf799366136f5d48e20447a49e
[c:192ac8a]: https://github.com/serenity-rs/serenity/commit/192ac8aec0afb33055352ed6e6838c506cbbbf8c
[c:1a08904]: https://github.com/serenity-rs/serenity/commit/1a089048138e85607bd298ebc07e30f57fb4ac53
[c:1ab8b31]: https://github.com/serenity-rs/serenity/commit/1ab8b31a19c6782b867b518c01bad9fbbdd06241
[c:1fad3dd]: https://github.com/serenity-rs/serenity/commit/1fad3dd60a0a9a0959f6e7e55896bef151bf3e9d
[c:25d4931]: https://github.com/serenity-rs/serenity/commit/25d49316133e2a8b7c4b26d3b6a44efdf5ad8834
[c:25e91da]: https://github.com/serenity-rs/serenity/commit/25e91dabd2380bd8fd98acbb7cb220dd90d238bd
[c:266411c]: https://github.com/serenity-rs/serenity/commit/266411cd6fc9ee96310da52c68264f303bcf5938
[c:26919cf]: https://github.com/serenity-rs/serenity/commit/26919cf9aad1d7bc5f0f8042b4caf6bfcddbd7d8
[c:29ee627]: https://github.com/serenity-rs/serenity/commit/29ee627207e0c2a0d3f5310ac00d90b232d910c0
[c:2b053ea]: https://github.com/serenity-rs/serenity/commit/2b053ea007d6ca9cc820cb910597e8b5dad89d70
[c:2fb12e2]: https://github.com/serenity-rs/serenity/commit/2fb12e2b3782fff211a41cb27cd316afc4320a7b
[c:3017f6d]: https://github.com/serenity-rs/serenity/commit/3017f6dbc02e6189c69491993e828e2a7595cbed
[c:32de2cb]: https://github.com/serenity-rs/serenity/commit/32de2cb941e8d4fdffde7b8b82599fcd78ab4c2f
[c:3582691]: https://github.com/serenity-rs/serenity/commit/35826915a174c7f3e5d82bbc320d3238ae308d8c
[c:3c2716b]: https://github.com/serenity-rs/serenity/commit/3c2716bbaeb71eca8cb2c7fca0dfd0b00cd34ba5
[c:3db42c9]: https://github.com/serenity-rs/serenity/commit/3db42c96c98fdd6d332347767cb1c276858da98b
[c:3e0b103]: https://github.com/serenity-rs/serenity/commit/3e0b1032d80a1847558a752e8316d97f9ae58f04
[c:40031d9]: https://github.com/serenity-rs/serenity/commit/40031d9ec55b1a4dd6e350a7566ea230751a54ed
[c:420f9bd]: https://github.com/serenity-rs/serenity/commit/420f9bdaa5a5022ff1d769f1d44a689a6fea12a4
[c:421c709]: https://github.com/serenity-rs/serenity/commit/421c709bbd706d4f04453baacf0ec6a88759f8cd
[c:428cbb9]: https://github.com/serenity-rs/serenity/commit/428cbb94de239e87d3258891591e1464cb9d2e06
[c:4532e4a]: https://github.com/serenity-rs/serenity/commit/4532e4a1e87d7b4f09446b1f10db178931eb314a
[c:45d72ef]: https://github.com/serenity-rs/serenity/commit/45d72eff173d87b1353d8b5d001775cc49129dab
[c:47ea8f7]: https://github.com/serenity-rs/serenity/commit/47ea8f79b4e980e38fb337b2f3cefc5c7d92fb33
[c:485ad29]: https://github.com/serenity-rs/serenity/commit/485ad299fec218ed3fd354f7207ce6160d803b06
[c:4be6b9d]: https://github.com/serenity-rs/serenity/commit/4be6b9d5008ff8bb3d1fdddff5647a6bb307513c
[c:4d4e9dc]: https://github.com/serenity-rs/serenity/commit/4d4e9dcf4b559423dd5b169ecef46efe6a0d1fca
[c:4e360cf]: https://github.com/serenity-rs/serenity/commit/4e360cf86a74051e2d4f98758c65ae29b97b7b8b
[c:4efe1d1]: https://github.com/serenity-rs/serenity/commit/4efe1d1271515e9ffecd318e368f127becfe273f
[c:4f2e47f]: https://github.com/serenity-rs/serenity/commit/4f2e47f399a10b281a1638fd7fcd3b945154d52c
[c:50d7f00]: https://github.com/serenity-rs/serenity/commit/50d7f00f1b01f4e0d9c86dbdd05a4d4f7b41f8b1
[c:511ec87]: https://github.com/serenity-rs/serenity/commit/511ec87280e8ddec6589f48fec8260bf2e598bdb
[c:52b8e29]: https://github.com/serenity-rs/serenity/commit/52b8e29193801aa254ac7ab105331fb6b0e8eec1
[c:561b0e3]: https://github.com/serenity-rs/serenity/commit/561b0e38b4cda6661425f76c8d707d58d0f12d09
[c:562ce49]: https://github.com/serenity-rs/serenity/commit/562ce49698a39d5da68d3ac58a3d8cf401aa9e42
[c:5a96724]: https://github.com/serenity-rs/serenity/commit/5a967241efabd49116a6d6d5a6eeb95d3281d93b
[c:5e5f161]: https://github.com/serenity-rs/serenity/commit/5e5f161f83b48367bc65d83f8d3cb7f4b1b61f0a
[c:5fd3509]: https://github.com/serenity-rs/serenity/commit/5fd3509c8cfe25370ca4fa66a8468bd2a9679ef5
[c:60c33db]: https://github.com/serenity-rs/serenity/commit/60c33db56bb3754bb0d2196d5f48fee63adf7730
[c:619a91d]: https://github.com/serenity-rs/serenity/commit/619a91de7a2d3e882cbcb8d8566ffeee3bc8192f
[c:64bfc54]: https://github.com/serenity-rs/serenity/commit/64bfc5471808cff59c9b4b5eef80a756f13ff5be
[c:6572580]: https://github.com/serenity-rs/serenity/commit/657258040376be45a8be0ef0e3bd762a23babb0a
[c:68c5be8]: https://github.com/serenity-rs/serenity/commit/68c5be8b6beec57618abea4d8b5bcca34489746e
[c:6a101c4]: https://github.com/serenity-rs/serenity/commit/6a101c4a409ae3abe4038f96dcd51f0788d4c0e4
[c:6c43fed]: https://github.com/serenity-rs/serenity/commit/6c43fed3702be3fdc1eafed26a2f6335acd71843
[c:6d6063f]: https://github.com/serenity-rs/serenity/commit/6d6063fc8334a4422465d30e938a045fd7a09d17
[c:6f147e1]: https://github.com/serenity-rs/serenity/commit/6f147e182b60817dd16e7868326b8cfa1f89ac88
[c:710fa02]: https://github.com/serenity-rs/serenity/commit/710fa02405d8d740c4ee952822d856af0e845aa8
[c:78e7b1b]: https://github.com/serenity-rs/serenity/commit/78e7b1b0624edce9bf69ff6d1d652f9cdfd3f841
[c:7c4b052]: https://github.com/serenity-rs/serenity/commit/7c4b052d7b5a50f234721249bd0221f037e48ea9
[c:7e8da0c]: https://github.com/serenity-rs/serenity/commit/7e8da0c6574ed051de5a9d51001ead0779dfb1de
[c:7e913b6]: https://github.com/serenity-rs/serenity/commit/7e913b6185468d2dd3740c711d418a300584b5bb
[c:824f8cb]: https://github.com/serenity-rs/serenity/commit/824f8cb63271ac3907a9c8223b08b7ee6ff0d746
[c:870a2a5]: https://github.com/serenity-rs/serenity/commit/870a2a5f821c9b0624cad03d873d04a8aad47082
[c:878684f]: https://github.com/serenity-rs/serenity/commit/878684f61fb48a25e117ed32548f78869cb027fc
[c:88765d0]: https://github.com/serenity-rs/serenity/commit/88765d0a978001ff88a1ee12798a725b7f5a90e9
[c:8a33329]: https://github.com/serenity-rs/serenity/commit/8a333290365f1304ad84a8e8f17c0d60728241c2
[c:8bf77fa]: https://github.com/serenity-rs/serenity/commit/8bf77fa431308451411670f20896e36f920997c5
[c:8cc2300]: https://github.com/serenity-rs/serenity/commit/8cc2300f7f2992ae858808033137440ee7e22cd8
[c:8d51ead]: https://github.com/serenity-rs/serenity/commit/8d51ead1747296eac5f2880332ae3e6de048ea4f
[c:8e1435f]: https://github.com/serenity-rs/serenity/commit/8e1435f29a2051f3f481131399fedf5528cb96e4
[c:8e29694]: https://github.com/serenity-rs/serenity/commit/8e296940b7e40879dcfbb185282b906804ba7e3d
[c:8e3b4d6]: https://github.com/serenity-rs/serenity/commit/8e3b4d601ffb78909db859640482f7e0bb10131f
[c:8f37f78]: https://github.com/serenity-rs/serenity/commit/8f37f78af0b9fda4cb0c4bf41e4c047958aa5a40
[c:924c447]: https://github.com/serenity-rs/serenity/commit/924c44759a79a8467cbf9f616a6aaa54c0e746cb
[c:948b27c]: https://github.com/serenity-rs/serenity/commit/948b27ce74e8dce458d427d8159f2a821d4d7cec
[c:97e84fe]: https://github.com/serenity-rs/serenity/commit/97e84fe136c5649ca3529c11790d9988dfe3bb92
[c:9900b20]: https://github.com/serenity-rs/serenity/commit/9900b20bf5cd4036cd8d8ba28bdcd852f2c89d2f
[c:9ccf388]: https://github.com/serenity-rs/serenity/commit/9ccf388e89b0cedddbf76a2236254d4d6ba0dd02
[c:9f02720]: https://github.com/serenity-rs/serenity/commit/9f02720d53ea117b1f6505a061b42fd7044219b9
[c:aa307b1]: https://github.com/serenity-rs/serenity/commit/aa307b160a263fb4d091d4aed06076b6c7f744b6
[c:aace5fd]: https://github.com/serenity-rs/serenity/commit/aace5fdb7f6eb71c143414c491005e378e299221
[c:ab67c1d]: https://github.com/serenity-rs/serenity/commit/ab67c1dd60b5f49541815b2527e8a3cb7712e182
[c:af1061b]: https://github.com/serenity-rs/serenity/commit/af1061b5e82ed1bf4e71ff3146cb98bc6cbb678c
[c:b249c82]: https://github.com/serenity-rs/serenity/commit/b249c8212ecd37cf3d52188fcc56f45268b3400e
[c:b602805]: https://github.com/serenity-rs/serenity/commit/b602805501df003d1925c2f0d0c80c2bac6d32a2
[c:b6af867]: https://github.com/serenity-rs/serenity/commit/b6af86779701110f7f21da26ae8712f4daf4ee3b
[c:bc3491c]: https://github.com/serenity-rs/serenity/commit/bc3491cf3a70a02ce5725e66887746567ae4660c
[c:bd05bda]: https://github.com/serenity-rs/serenity/commit/bd05bdad1765ad2038dcc4650e1ad4da8a2e020c
[c:bd9fcf7]: https://github.com/serenity-rs/serenity/commit/bd9fcf73a7912c900d194a0bebae586fb0d96d79
[c:bfdb57c]: https://github.com/serenity-rs/serenity/commit/bfdb57cdf35721f4953d436a819745ac5d44295e
[c:c2cf691]: https://github.com/serenity-rs/serenity/commit/c2cf6910b6a77c40d543d8950fca45c0d49b6073
[c:c68d4d5]: https://github.com/serenity-rs/serenity/commit/c68d4d5230e60ab48c5620f3d7daff666ded4a11
[c:c7b8ab8]: https://github.com/serenity-rs/serenity/commit/c7b8ab89c33c72b36b789dcc0648c164df523b1b
[c:ca0f113]: https://github.com/serenity-rs/serenity/commit/ca0f113324c1ed64a8646c42ed742dd8021fbccd
[c:caf69d6]: https://github.com/serenity-rs/serenity/commit/caf69d66893c2688f0856cc33f03702071d1314a
[c:cb18d42]: https://github.com/serenity-rs/serenity/commit/cb18d4207c3b9cf942bd561e76ae4059dd50979d
[c:cdedf36]: https://github.com/serenity-rs/serenity/commit/cdedf36330aa6da9e59d296164090f54b651b874
[c:d35d719]: https://github.com/serenity-rs/serenity/commit/d35d719518a48b1cf51c7ecb5ed9c717893784dc
[c:d8027d7]: https://github.com/serenity-rs/serenity/commit/d8027d7a3b9521565faa829f865c6248b3ba26c5
[c:d925f92]: https://github.com/serenity-rs/serenity/commit/d925f926c0f9f5b8010a998570441258417fc89a
[c:dbcb351]: https://github.com/serenity-rs/serenity/commit/dbcb3514f20409b3c4c4054fe51aaa2bd1792b96
[c:dbd6727]: https://github.com/serenity-rs/serenity/commit/dbd672783ef6f647664d3b1aa97957af9321d55c
[c:dc3a4df]: https://github.com/serenity-rs/serenity/commit/dc3a4dfafb1ee096b56c78d2506743e4012323f7
[c:deee38d]: https://github.com/serenity-rs/serenity/commit/deee38d87d71a918b6d8270dbfaffeb0a7234508
[c:e1912c2]: https://github.com/serenity-rs/serenity/commit/e1912c22fc806f97d9eb9025aa2432e785003f3b
[c:e1a8fe3]: https://github.com/serenity-rs/serenity/commit/e1a8fe3e9f619fbb94dd54993c8f5d25fd5dc375
[c:e2053dd]: https://github.com/serenity-rs/serenity/commit/e2053dd53f7c85175901ee57f7c028ba369487a9
[c:e218ce0]: https://github.com/serenity-rs/serenity/commit/e218ce0ec78b7b480e9a83628378dc9670e2cf4a
[c:e5889ed]: https://github.com/serenity-rs/serenity/commit/e5889ed1a62ddcb6bc11364800cd813329eb3ece
[c:e72e25c]: https://github.com/serenity-rs/serenity/commit/e72e25cf8b0160a3ec0de0be98dd8f1467d3b505
[c:e7a5ba3]: https://github.com/serenity-rs/serenity/commit/e7a5ba3e6c7e914c952408828f0cc71e15acea61
[c:ea1eba8]: https://github.com/serenity-rs/serenity/commit/ea1eba89087825e526e54fffdb27642fe72f9602
[c:ea432af]: https://github.com/serenity-rs/serenity/commit/ea432af97a87b8a3d673a1f40fe06cde4d84e146#diff-2e7fe478bd2e14b5b3306d2c679e4b5a
[c:eb47559]: https://github.com/serenity-rs/serenity/commit/eb47559fa00c13c8fdc8f40a8fe3d06690c0570c
[c:ebc4e51]: https://github.com/serenity-rs/serenity/commit/ebc4e51fe3b1e5bc61dc99da25a22d2e2277ffc6
[c:eee857a]: https://github.com/serenity-rs/serenity/commit/eee857a855831851599e5196750b27b26151eb16
[c:f05efce]: https://github.com/serenity-rs/serenity/commit/f05efce7af0cb7020e7da08c7ca58fa6f786d4ef
[c:f16af97]: https://github.com/serenity-rs/serenity/commit/f16af97707edfc36f52fa836791d07512e5d41ef
[c:f5a97d4]: https://github.com/serenity-rs/serenity/commit/f5a97d43b467130fd97af8c8a0dd1bbf0e7f5326
[c:f830f31]: https://github.com/serenity-rs/serenity/commit/f830f31f046b39124877a65fa1a95f789d125809
[c:fb2a1a9]: https://github.com/serenity-rs/serenity/commit/fb2a1a9262b481af62f9c0025a0f180626d19241
[c:fbc1ac7]: https://github.com/serenity-rs/serenity/commit/fbc1ac740e769e624637c490b6a959ed86ec3839
[c:fc9eba3]: https://github.com/serenity-rs/serenity/commit/fc9eba3d6d6a600f7d45a6f4e5918aae1191819d
[c:fd47b86]: https://github.com/serenity-rs/serenity/commit/fd47b865f3c32f5bbfce65162023898a6ecd29a1
[c:fd89d09]: https://github.com/serenity-rs/serenity/commit/fd89d09d3397eba21d1b454d3b6155ba9c3a829e
[c:fdbfbe0]: https://github.com/serenity-rs/serenity/commit/fdbfbe098c9d59000c234a0893496751744fd31e
[c:fdfb184]: https://github.com/serenity-rs/serenity/commit/fdfb1846083165629feca81b5169ceaf331289c5
[c:f6fcf32]: https://github.com/serenity-rs/serenity/commit/f6fcf32e7f62dfc207ac2f9f293f804446ea3423
[c:fdfd5bc]: https://github.com/serenity-rs/serenity/commit/fdfd5bcf708b6633b564fc58fb86935536310314

[c:00fb61b]: https://github.com/serenity-rs/serenity/commit/00fb61b5f306aebde767cc21a498a8ca0742d0be
[c:0102706]: https://github.com/serenity-rs/serenity/commit/0102706321a00cfb39b356bdf2cf8d523b93a8ec
[c:01f6872]: https://github.com/serenity-rs/serenity/commit/01f687204dd9d5564ec4bdc860f11bfd5e01454f
[c:04cfaa9]: https://github.com/serenity-rs/serenity/commit/04cfaa9a69dc1638e9cd1904a9b8e94c1a97f832
[c:060b06e]: https://github.com/serenity-rs/serenity/commit/060b06ec62b1f4e4cc2c11b877fd988b7dcfe627
[c:063a52f]: https://github.com/serenity-rs/serenity/commit/063a52f8c028c7432ee556380d2bd5c652d75d22
[c:0708ccf]: https://github.com/serenity-rs/serenity/commit/0708ccf85bac347e59053133a2b8b6f2eabe99ba
[c:096b0f5]: https://github.com/serenity-rs/serenity/commit/096b0f57aae04a5e0ea28414f5016eeafc5b9e0a
[c:0a2f5ab]: https://github.com/serenity-rs/serenity/commit/0a2f5ab525022fbf0055649f2262573fb07cf18c
[c:0b95db9]: https://github.com/serenity-rs/serenity/commit/0b95db916580b8b7eb8bf7e81e6051f849a9c0c8
[c:0b9bf91]: https://github.com/serenity-rs/serenity/commit/0b9bf91f62eef85a4eca703902077f4c04b3b6d1
[c:0c9ec37]: https://github.com/serenity-rs/serenity/commit/0c9ec377aa7281fb3d4bc390c896b426660a5387
[c:0d218e0]: https://github.com/serenity-rs/serenity/commit/0d218e02e043c043d7274c7169607b11c9897a5a
[c:0ec4dfb]: https://github.com/serenity-rs/serenity/commit/0ec4dfb785459c0d04c295f84a1c33e71c016eba
[c:0f41ffc]: https://github.com/serenity-rs/serenity/commit/0f41ffc811827fdd45e4e631884909e33fa8769e
[c:11a02db]: https://github.com/serenity-rs/serenity/commit/11a02db8e70c18a152bad9de6491817efc1d2f54
[c:13de5c2]: https://github.com/serenity-rs/serenity/commit/13de5c2e50410c3a68435dc774537b490bb7115c
[c:143337a]: https://github.com/serenity-rs/serenity/commit/143337ae717773f59562d67f85d0e9c44063a45b
[c:147cf01]: https://github.com/serenity-rs/serenity/commit/147cf01d4f13e3ee15eb03705ab2b7a006851cdd
[c:1561f9e]: https://github.com/serenity-rs/serenity/commit/1561f9e36384a215d2b866a752996f80d36a3ede
[c:1594961]: https://github.com/serenity-rs/serenity/commit/159496188b2c841a65829328cddafef620c517af
[c:16bd765]: https://github.com/serenity-rs/serenity/commit/16bd765112befd5d81818cab7b97ac59bd8a1b75
[c:16d1b3c]: https://github.com/serenity-rs/serenity/commit/16d1b3cad3982accd805f64ef93e51d921b3da55
[c:1700a4a]: https://github.com/serenity-rs/serenity/commit/1700a4a9090789d485c190c2a6ccd2c48986f5dd
[c:175d3a3]: https://github.com/serenity-rs/serenity/commit/175d3a3ef585f6fede959183138d507886192a4e
[c:2416813]: https://github.com/serenity-rs/serenity/commit/24168137ff7b1ec44d3ecdec0f516455fd3785a7
[c:268f356]: https://github.com/serenity-rs/serenity/commit/268f356a25f27175a5d72458fff92b0f770d0a5a
[c:2844ae1]: https://github.com/serenity-rs/serenity/commit/2844ae158f3d8297b17a584ff9a75f1f51116f48
[c:2845681]: https://github.com/serenity-rs/serenity/commit/28456813f6f05e9bdaf08e8cad641df1e3dfaff7
[c:2a743ce]: https://github.com/serenity-rs/serenity/commit/2a743cedaf08f7eb532e3c4b795cfc5f85bc96af
[c:2afab7c]: https://github.com/serenity-rs/serenity/commit/2afab7c6eb828e491721e15f11a76ae36e34796d
[c:2b237e7]: https://github.com/serenity-rs/serenity/commit/2b237e7de221beab9c516d6de29f83188ef63840
[c:2cb607d]: https://github.com/serenity-rs/serenity/commit/2cb607d72a39aa7ab3df866b23de4c9798e69a0f
[c:2d09152]: https://github.com/serenity-rs/serenity/commit/2d091528287b7f5dfd678e9bc77c25bf53b0f420
[c:2eaa415]: https://github.com/serenity-rs/serenity/commit/2eaa4159955260e7c9ade66803d69865f1f76018
[c:302d771]: https://github.com/serenity-rs/serenity/commit/302d771182308f907423ed73be9b736f268737fe
[c:3062981]: https://github.com/serenity-rs/serenity/commit/3062981bfc1412e93450b30fa9405e555624ce1e
[c:31aae7d]: https://github.com/serenity-rs/serenity/commit/31aae7d12763f94a7a08ea9fd0102921e8402241
[c:31becb1]: https://github.com/serenity-rs/serenity/commit/31becb16f184cd7d396b383ad4abed8095451fcb
[c:32e07e4]: https://github.com/serenity-rs/serenity/commit/32e07e4ac822d5cc1118f0db0fc92b549c1aaf81
[c:3348178]: https://github.com/serenity-rs/serenity/commit/3348178f151d8e1d7aa0432984a2dd23fa7b9e89
[c:345e140]: https://github.com/serenity-rs/serenity/commit/345e1401142d21a0fdabb2accd1f33e3a07c02c8
[c:38a484d]: https://github.com/serenity-rs/serenity/commit/38a484d0fec91e290bc1633fc871131f9decd0ca
[c:38db32e]: https://github.com/serenity-rs/serenity/commit/38db32e2cbb9dc8504e0dfbc2366b17596836da0
[c:39a28d3]: https://github.com/serenity-rs/serenity/commit/39a28d3bf5d7005c3549a09542d27c08660f49cb
[c:3c7c575]: https://github.com/serenity-rs/serenity/commit/3c7c575d988f4dc793678880560aee48456f4526
[c:3ca7ad9]: https://github.com/serenity-rs/serenity/commit/3ca7ad92507f056054d081485f437c08505bc7e5
[c:3f03f9a]: https://github.com/serenity-rs/serenity/commit/3f03f9adc97315bb61a5c71f52365306cb8e2d1a
[c:404a089]: https://github.com/serenity-rs/serenity/commit/404a089af267c5d5c33025a3d74826e02b6f8ca1
[c:4229034]: https://github.com/serenity-rs/serenity/commit/42290348bc05c876b7e70c570a485160e594e098
[c:4267bdb]: https://github.com/serenity-rs/serenity/commit/4267bdbae05d5516774ca72fe92789651cfa7230
[c:43a5c5d]: https://github.com/serenity-rs/serenity/commit/43a5c5d7eb8bffb8c9ca450ab1bc377d602fb8c3
[c:46b79dd]: https://github.com/serenity-rs/serenity/commit/46b79ddb45d03bfbe0eb10a9d5e1c53c9a15f55b
[c:494cc50]: https://github.com/serenity-rs/serenity/commit/494cc50ff3dcf8553a5588fa868754d27c237055
[c:49a6841]: https://github.com/serenity-rs/serenity/commit/49a684134df32427e9502192122c4fb22ef1a735
[c:4a14b92]: https://github.com/serenity-rs/serenity/commit/4a14b92ff58173acb98c7e0a135b4989a87a7529
[c:4cf8338]: https://github.com/serenity-rs/serenity/commit/4cf8338e364b0feefef26ece6649077e87962ff3
[c:4de39da]: https://github.com/serenity-rs/serenity/commit/4de39da887248e374b4d824472a6422c7e46dacc
[c:4f5fbb5]: https://github.com/serenity-rs/serenity/commit/4f5fbb54ae930dd56aa9a53878cf1b5e123de038
[c:51c15d0]: https://github.com/serenity-rs/serenity/commit/51c15d088054dd42c66fee10deed1431df931ec9
[c:543b604]: https://github.com/serenity-rs/serenity/commit/543b60421d1c6acd77e02cdd11c7dd2157399821
[c:55ccaca]: https://github.com/serenity-rs/serenity/commit/55ccaca57051b3fbd47cf7fa288014d9c36f6952
[c:57c060f]: https://github.com/serenity-rs/serenity/commit/57c060fa2fccfbb3b3d4b2d18aad2faa5929deb3
[c:585af23]: https://github.com/serenity-rs/serenity/commit/585af231028e46788d689f94e14e110c072a578e
[c:5918d01]: https://github.com/serenity-rs/serenity/commit/5918d01ed69541e43aed0e62ee6eadbf5ebb20d2
[c:5b275fc]: https://github.com/serenity-rs/serenity/commit/5b275fc425d4ef1c1a9eaa9d915db1f873f9c11d
[c:5bf6c2d]: https://github.com/serenity-rs/serenity/commit/5bf6c2d2cf0491951eddb10ab2641d02d0e730a1
[c:5c40e85]: https://github.com/serenity-rs/serenity/commit/5c40e85001b9b2620a76fcc57d8f0cddfb6f9b34
[c:5ee5fef]: https://github.com/serenity-rs/serenity/commit/5ee5feff615565b6f661ee3598fe19bb98bd6a88
[c:5fe6a39]: https://github.com/serenity-rs/serenity/commit/5fe6a3956d39e9b5caef19df78e8b392898b6908
[c:601704a]: https://github.com/serenity-rs/serenity/commit/601704acb94601a134ae43e795474afe8574b2ae
[c:626ffb2]: https://github.com/serenity-rs/serenity/commit/626ffb25af35f5b91a76fdccf6788382a1c39455
[c:62ed564]: https://github.com/serenity-rs/serenity/commit/62ed564e5f67f3e25d2307fbbf950d0489a28de8
[c:6355288]: https://github.com/serenity-rs/serenity/commit/635528875c59d34f0d7b2f2b0a3bd61d762f0e9c
[c:6502ded]: https://github.com/serenity-rs/serenity/commit/6502dedfcced471aaf17b7d459da827a1867807a
[c:651c618]: https://github.com/serenity-rs/serenity/commit/651c618f17cb92d3ea9bbd1d5f5c92a015ff64e0
[c:6579b1f]: https://github.com/serenity-rs/serenity/commit/6579b1fb0409410f303a4df5e7246c507a80f27b
[c:66546d3]: https://github.com/serenity-rs/serenity/commit/66546d36749f6c78a4957a616524fab734d5c972
[c:6853daf]: https://github.com/serenity-rs/serenity/commit/6853daf4d04719a9a8a081151bd85336e160a752
[c:68c473d]: https://github.com/serenity-rs/serenity/commit/68c473dd17a2098f97808b3d1f2a200621f67c9d
[c:69ec62a]: https://github.com/serenity-rs/serenity/commit/69ec62a42bcb143cdde056ad8ffce81922e88317
[c:6a887b2]: https://github.com/serenity-rs/serenity/commit/6a887b25f2712d70c65fc85b5cfbd8b6d4b41260
[c:6b0b9b2]: https://github.com/serenity-rs/serenity/commit/6b0b9b2491fa895bd7dd8e065f067470ea08639d
[c:6e11a10]: https://github.com/serenity-rs/serenity/commit/6e11a103f7a6a4ab43b1aa511aad9e04b1fd8c5a
[c:6f33a35]: https://github.com/serenity-rs/serenity/commit/6f33a35c4f85a06c45c4ce9e118db203c4951475
[c:70bf22a]: https://github.com/serenity-rs/serenity/commit/70bf22a00cd19651a0d994cc43e8d8c4bd8947fc
[c:70d4e75]: https://github.com/serenity-rs/serenity/commit/70d4e7538cefc21dd0e06d5451888b82f53acf38
[c:71f3dbb]: https://github.com/serenity-rs/serenity/commit/71f3dbb650f4b0d6434630137ae9eea502a1ebef
[c:760a47a]: https://github.com/serenity-rs/serenity/commit/760a47aa4d34160f44048e775afeb30f08891c99
[c:76f9095]: https://github.com/serenity-rs/serenity/commit/76f9095c012a8769c7bd27aca6540b7018574c28
[c:77b5b48]: https://github.com/serenity-rs/serenity/commit/77b5b480d67e747908f8f4fb9f910bab23b761b5
[c:7914274]: https://github.com/serenity-rs/serenity/commit/79142745cb571ba2d4284fd1dcbe53c14a0ed623
[c:7990381]: https://github.com/serenity-rs/serenity/commit/799038187d903a75d60f0c98d013ae87fb665d02
[c:7b45f16]: https://github.com/serenity-rs/serenity/commit/7b45f16f063a47dc8a302dce5b016cf43a3edcc1
[c:7b4b154]: https://github.com/serenity-rs/serenity/commit/7b4b1544603a70dd634b51593ea5173b4515889a
[c:7dbae6b]: https://github.com/serenity-rs/serenity/commit/7dbae6b5261b8f53200090c9eb1bf39a7498f07d
[c:7e254c5]: https://github.com/serenity-rs/serenity/commit/7e254c5c6098bb1a47bac26c9895098a46cdc53f
[c:7f04179]: https://github.com/serenity-rs/serenity/commit/7f041791aa95e38a0cacd2ab64f0423524c60052
[c:7fc49d8]: https://github.com/serenity-rs/serenity/commit/7fc49d8dd9e253b066ab1b82446d0344f800e2d7
[c:c832009]: https://github.com/serenity-rs/serenity/commit/c832009eae235881815186f740b716e0b7e63951
[c:8360f32]: https://github.com/serenity-rs/serenity/commit/8360f329eae1751a8a413a6f6838486f3a0bba01
[c:83b1d96]: https://github.com/serenity-rs/serenity/commit/83b1d967f4cc2040f94d67dd987302347f227d6a
[c:83b29d5]: https://github.com/serenity-rs/serenity/commit/83b29d5f839cd2ea6cd150aa7b8ccbbc677c1fad
[c:858bbf2]: https://github.com/serenity-rs/serenity/commit/858bbf298d08ada3ae6c5b24105bf751bc938d5e
[c:86a4e00]: https://github.com/serenity-rs/serenity/commit/86a4e008ca7acf23d920e344463df801a774d5ce
[c:86cd00f]: https://github.com/serenity-rs/serenity/commit/86cd00f20d6f218e524deed040d3c209f0361a86
[c:8b504ad]: https://github.com/serenity-rs/serenity/commit/8b504ad7f6e10fecb27583a949262eb61cfd266d
[c:8c04d31]: https://github.com/serenity-rs/serenity/commit/8c04d318e273e9bcb3af6ddd820ad067048e95c6
[c:8c0aeac]: https://github.com/serenity-rs/serenity/commit/8c0aeacadb93d3b56fb98beb882eaef1f79cd652
[c:8c5ee70]: https://github.com/serenity-rs/serenity/commit/8c5ee70b28b42ac92f899932ab2ddafeb9c6f913
[c:8e2c052]: https://github.com/serenity-rs/serenity/commit/8e2c052a55e5e08c6e7ed643b399f1a7f69a2b25
[c:8effc91]: https://github.com/serenity-rs/serenity/commit/8effc918cc3d269b0d4cf34ef4f2053cecad2606
[c:8f24aa3]: https://github.com/serenity-rs/serenity/commit/8f24aa391f6b8a9103a9c105138c7610288acb05
[c:8f88c6b]: https://github.com/serenity-rs/serenity/commit/8f88c6b0613199492ebca8cd9f2bf4dd5c97add7
[c:8f8a059]: https://github.com/serenity-rs/serenity/commit/8f8a05996c5b47ec9401aabb517d96ed2af5c36b
[c:9114963]: https://github.com/serenity-rs/serenity/commit/9114963daf708cfaeaf54d8c788206ccfbae5df8
[c:921f7f4]: https://github.com/serenity-rs/serenity/commit/921f7f42d87e7c727b5a87802d7738f8081b600a
[c:92309b2]: https://github.com/serenity-rs/serenity/commit/92309b2fb8ffd96292fd2edaa7c223a2ba774a56
[c:9268f9c]: https://github.com/serenity-rs/serenity/commit/9268f9c10ef47ffeaeb3d5040e65b1093e04b866
[c:92f4ec2]: https://github.com/serenity-rs/serenity/commit/92f4ec204d10a8d60af9ce3cc7433be8117a711d
[c:933ee89]: https://github.com/serenity-rs/serenity/commit/933ee8914509e52c5119ced9f5d9d8f9644cfa63
[c:93416cd]: https://github.com/serenity-rs/serenity/commit/93416cdebff12a3f85e694c8cb28350a5c14c50f
[c:9392f61]: https://github.com/serenity-rs/serenity/commit/9392f61f8857b6ab2a04781c2d9c92a582a1577b
[c:93f3c60]: https://github.com/serenity-rs/serenity/commit/93f3c60b23cb8ffd16666bdc01b3502ca7ba5f47
[c:9969be6]: https://github.com/serenity-rs/serenity/commit/9969be60cf320797c37b317da24d9a08fd5eafa5
[c:97f9bd1]: https://github.com/serenity-rs/serenity/commit/97f9bd10c16eb24d54a0ab00c52f19eb51a88675
[c:990e611]: https://github.com/serenity-rs/serenity/commit/990e611a56f37f64fbce74fbc487c7dcc4aa4e28
[c:9aa357f]: https://github.com/serenity-rs/serenity/commit/9aa357f0c8f504b53b49824cc20561c8501d2dda
[c:9c04a19]: https://github.com/serenity-rs/serenity/commit/9c04a19015cf579d343d81a7fa50e6f4b18b4a5b
[c:9c1ed6c]: https://github.com/serenity-rs/serenity/commit/9c1ed6ca933f81bc0254d9d52159b9190b50a3ea
[c:9dae9e6]: https://github.com/serenity-rs/serenity/commit/9dae9e67b992cea4c18f1c685f5185abd9428887
[c:9ec05e7]: https://github.com/serenity-rs/serenity/commit/9ec05e701bdbadad39847f0dcc18d5156ecdde02
[c:9ef5522]: https://github.com/serenity-rs/serenity/commit/9ef55224757dff6dec8576bd1ad11db24a10891e
[c:a0bb306]: https://github.com/serenity-rs/serenity/commit/a0bb30686c1a9431aef23c2e8594791f64035194
[c:a2cbeb6]: https://github.com/serenity-rs/serenity/commit/a2cbeb6ece9ef56e2082b6eabbabe5fe536ab3ec
[c:a39647d]: https://github.com/serenity-rs/serenity/commit/a39647d3ba1650a4dd4c92bd40001959828000c7
[c:a8acd61]: https://github.com/serenity-rs/serenity/commit/a8acd6138741a6e5268141ac4ce902561931d353
[c:ab778f8]: https://github.com/serenity-rs/serenity/commit/ab778f8a9cf47c4e27fe688a61effb0caa4f8a6e
[c:ab7f113]: https://github.com/serenity-rs/serenity/commit/ab7f113a9e3acd000dbf69b7c4bd8d2d766b39f1
[c:abd22d2]: https://github.com/serenity-rs/serenity/commit/abd22d289599530cbd1bc9cf1b739420f0d22372
[c:ada07fa]: https://github.com/serenity-rs/serenity/commit/ada07fae09f3521f44d81613f26839d69c1fc7ef
[c:ae352ea]: https://github.com/serenity-rs/serenity/commit/ae352ea3df86eb2d853d5b1af048a95409aafc38
[c:ae395f4]: https://github.com/serenity-rs/serenity/commit/ae395f44361a9a9b488b31d6ac0cb54e0ee9e7a1
[c:aea9885]: https://github.com/serenity-rs/serenity/commit/aea98851e86c0f36be231c0a3b763f769c76e061
[c:afc571f]: https://github.com/serenity-rs/serenity/commit/afc571fd67c294cc10682db5c579d10645aec437
[c:b001234]: https://github.com/serenity-rs/serenity/commit/b0012349cca2a5c7c62bb6d2c99106d245b6c55a
[c:b468cbf]: https://github.com/serenity-rs/serenity/commit/b468cbffa0db341987d1dc397582b3edd3944d09
[c:b4bd771]: https://github.com/serenity-rs/serenity/commit/b4bd7714a155381cc16ece51acb0c4dc6cde96a2
[c:b7cbf75]: https://github.com/serenity-rs/serenity/commit/b7cbf75103939b0b7834c808050b19ba4fbc4b17
[c:b96f85c]: https://github.com/serenity-rs/serenity/commit/b96f85c224b9c0478b7f1b5c5b76761e23ff7edf
[c:bad9ac3]: https://github.com/serenity-rs/serenity/commit/bad9ac3d28bb0417dedcdddf10cf764c08d1d6ae
[c:bb97211]: https://github.com/serenity-rs/serenity/commit/bb97211b2b107943dd6fabb7a0a344d4fe236780
[c:bcb70e8]: https://github.com/serenity-rs/serenity/commit/bcb70e85384a16b2440788a73241f507aaeba4dc
[c:bceb049]: https://github.com/serenity-rs/serenity/commit/bceb049bb2b804dac975567bb7eac6afcfc28574
[c:c00f349]: https://github.com/serenity-rs/serenity/commit/c00f3490f2fb0c045c2da72d850f70da8e2cdb95
[c:c01f238]: https://github.com/serenity-rs/serenity/commit/c01f238a34ad846f8732c8bf97fbbd96fbf6a7ae
[c:c032fbe]: https://github.com/serenity-rs/serenity/commit/c032fbe7a5c65fb6824a5eb36daf327134b854cf
[c:c050c59]: https://github.com/serenity-rs/serenity/commit/c050c59da25b9093a75bda22baa81be3b267c688
[c:c2e8b69]: https://github.com/serenity-rs/serenity/commit/c2e8b69702cf81a1cf149c420aec999124f398e2
[c:c36841d]: https://github.com/serenity-rs/serenity/commit/c36841dd1c3f80141251ba01130333f43ff363d7
[c:c74cc15]: https://github.com/serenity-rs/serenity/commit/c74cc15f8969c8db68119d07a4f273a0d3fc44f4
[c:c8536c1]: https://github.com/serenity-rs/serenity/commit/c8536c111117f26833fb1bceff734ac1abc55479
[c:c8c6b83]: https://github.com/serenity-rs/serenity/commit/c8c6b83ca685a3e503c853d4154a17761790954e
[c:cd914f5]: https://github.com/serenity-rs/serenity/commit/cd914f503c8f0ada7473b5b56e4ad7830370ea45
[c:d033909]: https://github.com/serenity-rs/serenity/commit/d03390968ec7a5e1e93dbcc508c3b8a5f44b792d
[c:d0b64cd]: https://github.com/serenity-rs/serenity/commit/d0b64cd64a18a6116267fa09a837d62c19cced42
[c:d144136]: https://github.com/serenity-rs/serenity/commit/d1441363364970b749d57b8a4863b284239488d1
[c:d3389be]: https://github.com/serenity-rs/serenity/commit/d3389be3042fd7977350a08152d177ac6cdcd37f
[c:d367a70]: https://github.com/serenity-rs/serenity/commit/d367a704985bbb127f410770125c160f90561937
[c:d37461b]: https://github.com/serenity-rs/serenity/commit/d37461b5b705e0cdf802925c59113898a71676df
[c:d4fc8b6]: https://github.com/serenity-rs/serenity/commit/d4fc8b6188627ae8d553cf282b1371e3de7b01f9
[c:d58c544]: https://github.com/serenity-rs/serenity/commit/d58c54425a18bbbdc8e66e8eebfb8191bad06901
[c:d9118c0]: https://github.com/serenity-rs/serenity/commit/d9118c081742d6654dc0a4f60228a7a212ca436e
[c:daf92ed]: https://github.com/serenity-rs/serenity/commit/daf92eda815b8f539f6d759ab48cf7a70513915f
[c:db0f025]: https://github.com/serenity-rs/serenity/commit/db0f025d154e4b6212dd9340c1b789b3c711a24a
[c:dc73d1a]: https://github.com/serenity-rs/serenity/commit/dc73d1a4bad07b453a9d60a6c8f8c187a7e42450
[c:e033ff3]: https://github.com/serenity-rs/serenity/commit/e033ff33b94e024fe5f55a8c93c65c3e885f821b
[c:e1079e9]: https://github.com/serenity-rs/serenity/commit/e1079e9a03473f9ec67414628d5b84e7ea1b5b38
[c:e2557ac]: https://github.com/serenity-rs/serenity/commit/e2557ac794068c1a6a5c4c674ed9f7b7a806068e
[c:e4b484f]: https://github.com/serenity-rs/serenity/commit/e4b484f1c823ccb0aa2be7c54e0def07e5a01806
[c:e5a83dd]: https://github.com/serenity-rs/serenity/commit/e5a83dd1873e5af2df18835d960fe19286c70f1e
[c:e6712c9]: https://github.com/serenity-rs/serenity/commit/e6712c9459c367cf9ba3e5d9bf1c0831357a20b5
[c:e7110ad]: https://github.com/serenity-rs/serenity/commit/e7110adb1e5659b7395588381c2e56c2aa06d1fa
[c:e85e901]: https://github.com/serenity-rs/serenity/commit/e85e901062e8b9ea717ec6c6253c9c7a300448d3
[c:e891ebe]: https://github.com/serenity-rs/serenity/commit/e891ebeba43eb87c985db4e031b8bf76dcaca67b
[c:e8a9086]: https://github.com/serenity-rs/serenity/commit/e8a90860d1e451e21d3bf728178957fe54cf106d
[c:e9282d3]: https://github.com/serenity-rs/serenity/commit/e9282d3373158b6e9792a5484ae3dfb9212eb6f7
[c:e92b667]: https://github.com/serenity-rs/serenity/commit/e92b667058138ffd01587e28e9d8551cd59df160
[c:e9aae9c]: https://github.com/serenity-rs/serenity/commit/e9aae9c043b206b15bd5429126ded62259d6731b
[c:eb09f2d]: https://github.com/serenity-rs/serenity/commit/eb09f2d3389b135978e0671a0e7e4ed299014f94
[c:eb43b9c]: https://github.com/serenity-rs/serenity/commit/eb43b9c4a4e43a8e097ea71fdc7584c8108b52a3
[c:ec9b1c7]: https://github.com/serenity-rs/serenity/commit/ec9b1c79abeb2a4eff9f013ba8f0e430979dbc56
[c:ef6eba3]: https://github.com/serenity-rs/serenity/commit/ef6eba37636a487c0d6f3b93b8e76c94f28abbab
[c:f00e165]: https://github.com/serenity-rs/serenity/commit/f00e1654e8549ec6582c6f3a8fc4af6aadd56015
[c:f0d1157]: https://github.com/serenity-rs/serenity/commit/f0d1157212397ae377e11d4205abfebc849ba9d8
[c:f3f74ce]: https://github.com/serenity-rs/serenity/commit/f3f74ce43f8429c4c9e38ab7b905fb5a24432fd4
[c:f53124e]: https://github.com/serenity-rs/serenity/commit/f53124ec952124f5b742f204cdf7e1dc00a168ab
[c:f57a187]: https://github.com/serenity-rs/serenity/commit/f57a187d564bdcd77f568e77a102d6d261832ee0
[c:f69512b]: https://github.com/serenity-rs/serenity/commit/f69512beaa157775accd4392295dba112adcf1df
[c:f695174]: https://github.com/serenity-rs/serenity/commit/f695174287e3999cbcbabc691a86302fa8269900
[c:f6b27eb]: https://github.com/serenity-rs/serenity/commit/f6b27eb39c042e6779edc2d5d4b6e6c27d133eaf
[c:f847638]: https://github.com/serenity-rs/serenity/commit/f847638859423ffaaecfdb77ee5348a607ad3293
[c:f894cfd]: https://github.com/serenity-rs/serenity/commit/f894cfdc43a708f457273e1afb57ed1c6e8ebc58
[c:f96b6cc]: https://github.com/serenity-rs/serenity/commit/f96b6cc5e1e0383fd2de826c8ffd95565d5ca4fb
[c:fafa363]: https://github.com/serenity-rs/serenity/commit/fafa3637e760f0c72ae5793127bc2f70dcf2d0e2
[c:fb07751]: https://github.com/serenity-rs/serenity/commit/fb07751cfc1efb657cba7005c38ed5ec6b192b4f
[c:fb4d411]: https://github.com/serenity-rs/serenity/commit/fb4d411054fa44928b4fa052b19de19fce69d7cf
[c:ff4437a]: https://github.com/serenity-rs/serenity/commit/ff4437addb01e5c6c3ad8c5b1830db0d0a86396b

[c:f47a0c8]: https://github.com/serenity-rs/serenity/commit/f47a0c831efe5842ca38cb1067de361ae42f6edc
[c:d50b129]: https://github.com/serenity-rs/serenity/commit/d50b12931404946e219d3ff0878f0632445ef35f
[c:41f26b3]: https://github.com/serenity-rs/serenity/commit/41f26b3757c7a5fba1f09f34e3192e2fd9702a4a
[c:f9e5e76]: https://github.com/serenity-rs/serenity/commit/f9e5e76585a1f6317dadb67e440765b0070ca131
[c:9428787]: https://github.com/serenity-rs/serenity/commit/9428787abb6126ba05bfef96cd2b8d2a217fdf5d
[c:a58de97]: https://github.com/serenity-rs/serenity/commit/a58de97e6089aa98f04d2cdc7312ed38a9f72b22
[c:fbd6258]: https://github.com/serenity-rs/serenity/commit/fbd625839e6a2e01b16e6c3814cb9b9f31dc7caa
[c:292ceda]: https://github.com/serenity-rs/serenity/commit/292cedaa3462f7532efda98722354afa8e213b6a
[c:d3015a0ff]: https://github.com/serenity-rs/serenity/commit/d3015a0ff0c0c87888437f991945453b92296875
[c:585ac6e]: https://github.com/serenity-rs/serenity/commit/585ac6e6ca792facf29063776c83262fa849161b
[c:3616585]: https://github.com/serenity-rs/serenity/commit/361658510f3e2eb9aefbe66232b9b1f1a1ebb80f
[c:e694766]: https://github.com/serenity-rs/serenity/commit/e694766bb6c93d5f6a75ad9871cfdefbd0309a17
[c:e02d5fb]: https://github.com/serenity-rs/serenity/commit/e02d5fb8171b11214e1502c6754fef1972bbf1b9
[c:b7cdf15]: https://github.com/serenity-rs/serenity/commit/b7cdf1542cb9199c61c0b17bdd381d4f117f635e
[c:c7aa27d]: https://github.com/serenity-rs/serenity/commit/c7aa27dbb64e64d70c7f13725c79017c4bba1c95
[c:2219bb3]: https://github.com/serenity-rs/serenity/commit/2219bb37a80c4c2b4ff5a24d72b82737eb241195
[c:74ec713]: https://github.com/serenity-rs/serenity/commit/74ec713825b2b4c55382fb76fa57bd967e66b3aa
[c:5829c67]: https://github.com/serenity-rs/serenity/commit/5829c673c13655b86d317ab65d204067a2b1a7a4
[c:ce4f8c2]: https://github.com/serenity-rs/serenity/commit/ce4f8c2ac8dd2c472ab537a60bf92579d078073b
[c:fcc4e2c]: https://github.com/serenity-rs/serenity/commit/fcc4e2ce2e523248ed33c9f4853d3485cbc9b6e6
[c:23ff6f]: https://github.com/serenity-rs/serenity/commit/23ff6f21019bc94f8dc32355fa34691b881bfb69
[c:e57b510]: https://github.com/serenity-rs/serenity/commit/e57b510edd640abb243664337a1c163924313612
[c:c149e36]: https://github.com/serenity-rs/serenity/commit/c149e368ae4bb1be5d0392b9cae282fc530831c5
