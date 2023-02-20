#!/usr/bin/env bash

cargo watch -s "trunk build --release; cp -R assets dist"
