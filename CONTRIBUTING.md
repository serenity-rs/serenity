# Pull Requests

Please post a comment on an existing issue if you'd like to work on it. Please
post an issue prior to dedicating a large amount of time on a PR so it can be
determined if the change is something that the community wants.

There are going to (usually) be 3 primary branches:

- `current`: Development branch of the _most recent_ majour version. For example,
if the largest version is v0.5.3, then the v0.5.x series will be on this branch.
Bugfixes, internal rewrites, documentation updates, new features, etc. go here
so long as they do not introduce breaking changes.
- `v0.Y.x`: Development branch of the _second most recent_ majour version. If
the largest version is v0.5.X, then this will be the branch for bugfixes for the
v0.4.x version series. Bugfixes from the `current` branch may be backported here
if applicable.
- `v0.Z.x`: Development branch of the _next_ majour version. Following the same
example, this would be for the v0.6.x version series. This is where breaking
changes go.

### Testing

Make sure you run tests via `cargo test --all-features` prior to submitting a
PR, and updating any of the examples in the `examples` directory where
applicable.

# Issues

For bug reports, please include the following information where applicable:

```
Serenity version:

Rust version (`rustc -V`):

Backtrace (make sure to run `RUST_BACKTRACE=1`):

Minimal test case if possible:
```

For feature requests or other requests, please just be as descriptive as
possible, potentially with a code sample of what it might look like.

# Code Style

We don't follow rustfmt because it often produces unreadable results.

Generally, there are a few rules to note, the rest should be grokable from
existing rules:

Add an empty line before and after logical blocks, but only if there code before
or after it. For example:

```rust
fn foo() {
    let x = true;

    if x {
        println!("x is true");
    }

    let y = 1u64;

    match y {
        1 => println!("y is 1"),
        other => println!("y is not 1, it is {}", other),
    }
}
```

Add an empty line after the subject line in documentation. For example:

```rust
/// This is the subject.
///
/// This is more detailed information.
///
/// Note the empty line after the subject, and between paragraphs.
fn foo() { }
```

We have an 80 characters per line soft limit, in case readability would suffer
and to support descriptive naming, 100 characters is our hard limit.
