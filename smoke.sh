#!/usr/bin/env bash
set -euo pipefail

BIN="${1:-cargo run --}"

$BIN -- summary fixtures/clean_hello_witchers.txt
$BIN -- summary fixtures/haunted_hello_witchers.txt || true
$BIN -- decode-zw fixtures/haunted_hello_witchers.txt || true
$BIN -- reveal fixtures/haunted_hello_witchers.txt || true
