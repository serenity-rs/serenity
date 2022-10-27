# Pull Requests

Please post a comment on an existing issue if you'd like to work on it. Please
post an issue prior to dedicating a large amount of time on a PR so it can be
determined if the change is something that the community wants.

There are going to (usually) be 2 primary branches:

- `current`: Development branch of the _most recent_ major version. For example,
if the largest version is v0.11.1, then the v0.11.x series will be on this branch.
Bugfixes, internal rewrites, documentation updates, new features, etc. go here
so long as they do not introduce breaking changes.

- `next`: Development branch of the _next_ major version. Following the same
example, this would be for the v0.12.x version series. This is where breaking
changes go.

We have a long-term support policy for the previous release series. If the _second
most recent_ major version is v0.10.10, then the v0.10.x branch will be the place
for bugfixes. Fixes from `current` may be backported to this branch if needed.
Occasionally, we might support two past release series for special reasons.

# Testing

Make sure you run tests with the various feature combinations, which you can
find in [our CI pipeline][test_ci]. To run tests with all features, use
`cargo test --features full`. Run and update the examples in the `examples`
directory where applicable. To simplify this procedure, you can use [cargo make][make]
to build and run examples. You can refer to the list of tasks in the [Makefile](Makefile.toml).

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

We leave the responsibility of formatting to the rustfmt tool to mitigate
friction from people's opinions about formatting code in a certain way,
putting more focus into functionality. Formatting is enforced in our CI pipeline,
and pull requests won't be accepted if this is not adhered. Before
committing your changes, always run `cargo fmt --all`.

We have an 80 characters per line soft limit. In case readability would suffer
and to support descriptive naming, 100 characters is our hard limit, enforced
by rustfmt.

Note that our rustfmt configuration uses unstable features! You will have to
install the nightly toolchain of Rust through [`rustup`] in order to format
your code.

## Unsafe

Code that defines or uses `unsafe` functions must be reasoned with comments.
`unsafe` code can pose a potential for undefined behaviour related bugs and other
kinds of bugs to sprout if misused, weakening security. If you commit code containing
`unsafe`, you should confirm that its usage is necessary and correct.

# Comment / Documentation style

Comments, including documentation comments, ought to be written in British English.
They should contain proper English sentences. The first word must be capitalised
and the sentences finished with either a full stop, question mark, or exclamation
mark.

Comments should always appear before the section they talk about. For example, if you
add a comment for some peculiar code at line 235, the comment should be placed at line 234.

When writing categories in documentation comments, prepend the name of the category with `#`.
This will allow to reference the category in links to the API. For example:

```rust
/// # Error
///
/// This function will return an error if the user could not be found in the cache.
```

When referencing other parts of Serenity's API (modules, structs, functions, etc.)
in the documentation, the path must be relative written in the form of a Rust path.
For instance: `[say](crate::model::channel::GuildChannel::say)`.
These are called [intra-doc links][in-links]. For more information, it is recommended
to read the linked RFC. Links to external websites are exempt from this guideline.

# Commit style

When creating a commit summary, use the imperative mood. The summary
should describe the action that is administered by the commit's changes.

Examples of proper commit summaries are:

- "Add tests for checking permissions" -- changes add new tests
- "Fix double sending bug" -- changes fix erroneous behaviour
- "Increase character limit to 2500" -- changes alter existing behaviour

Improper commit summaries are:

- "Removed deprecated items" -- past tense used
- "Changing default data for user objects" -- progressive tense used
- "Misc. changes" -- missing verb

The first letter of the summary must be capitalised. The summary should also
preferably fit into 50 characters, but this is not actively enforced.

# Noisy commits

Set `blame.ignoreRevsFile` to ignore [noise commits][noise-commits] in `git blame`:
```
git config blame.ignoreRevsFile .git-blame-ignore-revs
```

[test_ci]: .github/workflows/ci.yml
[noise-commits]: https://github.com/serenity-rs/serenity/commit/9bbb25aac4d651804286f333eb503a72d41e473b
[make]: https://github.com/sagiegurari/cargo-make
[`rustup`]: https://rustup.rs
[in-links]: https://github.com/rust-lang/rfcs/blob/master/text/1946-intra-rustdoc-links.md
