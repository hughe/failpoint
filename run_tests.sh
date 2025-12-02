#!/bin/bash

set -e

echo "Running all tests"
cargo test -- --test-threads=1

echo "Running example: conditional_comp (enabled)"
cargo run --example conditional_comp

echo "Running example: conditional_comp (disabled)"
cargo run --example conditional_comp --no-default-features


echo "Running example: failpoint"
cargo run --example failpoint

echo "Running example: multiple_error_types"
cargo run --example multiple_error_types

echo "Running example: test_codepath"
cargo run --example test_codepath
