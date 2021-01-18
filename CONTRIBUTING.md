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

# Testing

Make sure you run tests with the various feature combinations, which you can
find in [our CI pipeline][test_ci]. To run tests with all features, use
`cargo test --all-features`. Run and update the examples in the `examples`
directory where applicable.

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

We always follow rustfmt, and it is enforced in our CI pipeline. Before
committing your changes, always run `cargo fmt --all`.

We have an 80 characters per line soft limit. In case readability would suffer
and to support descriptive naming, 100 characters is our hard limit.

# Commit style

When creating a commit summary, use the imperative mood. The summary
should describe the action that is administered by the commit's changes.

Proper examples of a commit summary are:

- "Add tests for checking permissions" -- changes add new tests
- "Fix double sending bug" -- changes fix erroneous behaviour
- "Increase character limit to 2500" -- changes alter existing behaviour

Improper commit summary are:

- "Removed deprecated items"
- "Changing default data for user objects"
- "Misc. changes"

The summary should preferably fit into 50 characters. The first letter must
also be capitalized.

[test_ci]: .github/workflows/ci.yml
