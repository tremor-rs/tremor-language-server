#!/usr/bin/env bash

old=$1
new=$2

sed -e "s/^tremor-script = { version = \"${old}\"/tremor-script = { version = \"${new}\"/" -i "Cargo.toml"
