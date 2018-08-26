#!/usr/bin/env sh

# Tests only unit tests: does not test integration tests or doctests. Reduces
# test times by over 99%. Doctests are really only needed for PRs anyway.
#
# This is a quick and easy way to check that it compiles and the basic tests
# still validate.
cargo test "::test::" --all-features
