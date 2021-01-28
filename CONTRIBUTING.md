# Pull Requests

Please post a comment on an existing issue if you'd like to work on it. Please
post an issue prior to dedicating a large amount of time on a PR so it can be
determined if the change is something that the community wants.

There are going to (usually) be 2 primary branches:

- `current`: Development branch of the _most recent_ major version. For example,
if the largest version is v0.10.2, then the v0.10.x series will be on this branch.
Bugfixes, internal rewrites, documentation updates, new features, etc. go here
so long as they do not introduce breaking changes.

- `next`: Development branch of the _next_ major version. Following the same
example, this would be for the v0.11.x version series. This is where breaking
changes go.

We have a long-term support policy for the previous release series. If the _second
most recent_ major version is v0.9.4, then the v0.9.x branch will be the place
for bugfixes. Fixes from `current` may be backported to this branch if needed.
Occasionally, we might support two past release series for special reasons.

# Testing

Make sure you run tests with the various feature combinations, which you can
find in [our CI pipeline][test_ci]. To run tests with all features, use
`cargo test --all-features`. Run and update the examples in the `examples`
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

# Commit style

When creating a commit summary, use the imperative mood. The summary
should describe the action that is administered by the commit's changes.

Examples of proper commit summaries are:

- "Add tests for checking permissions" -- changes add new tests
- "Fix double sending bug" -- changes fix erroneous behaviour
- "Increase character limit to 2500" -- changes alter existing behaviour

Improper commit summaries are:

- "Removed deprecated items"
- "Changing default data for user objects"
- "Misc. changes"

The first letter of the summary must be capitalised. The summary should also
preferably fit into 50 characters, but this is not actively enforced.

[test_ci]: .github/workflows/ci.yml
[make]: https://github.com/sagiegurari/cargo-make
