#!/usr/bin/env bash

old=$1
new=$2

# update dependencies in Cargo.toml
sed -e "s/^tremor-script = { version = \"${old}\"/tremor-script = { version = \"${new}\"/" -i "Cargo.toml"

# update server version is backend tests
sed -e "s/^    const VERSION: \&str = \"${old}\";/    const VERSION: \&str = \"${new}\";/" -i "src/backend.rs"