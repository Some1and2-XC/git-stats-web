#!/usr/bin/sh
# Runs the development script to live reload
# Uses: https://github.com/watchexec/watchexec

watchexec -w src -r cargo run
