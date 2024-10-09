#!/usr/bin/sh
# Runs the development script to live reload
# Uses: https://github.com/watchexec/watchexec
# This can be installed through cargo with
# ```sh
# cargo install --locked watchexec-cli
# ```

watchexec -w src -r cargo run -- --log DEBUG
