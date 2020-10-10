# Songbird

![](songbird.png)

Songbird is a cross-library compatible voice system for Discord, written in Rust. The library offers:
 * A (standalone) gateway frontend compatible with [serenity] and [twilight] using the `"gateway"` and `"[serenity/twilight]-[rustls/native]"` features.
 * An optionally standalone driver for voice calls.
 * And, by default, a fully featured voice system featuring events, RT(C)P packet handling, seeking on compatible streams, shared multithreaded audio stream caches, and direct Opus data passthrough from DCA files.

## Attribution

Songbird's logo is based upon the copyright-free image ["Black-Capped Chickadee"] by George Gorgas White.

[serenity]: https://github.com/serenity-rs/serenity
[twilight]: https://github.com/twilight-rs/twilight
["Black-Capped Chickadee"]: https://www.oldbookillustrations.com/illustrations/black-capped-chickadee/