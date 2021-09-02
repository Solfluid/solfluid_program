#!/bin/sh
cargo build-bpf --manifest-path=Cargo.toml --bpf-out-dir=dist/program

#solana deploy --keypair ../solana-wallet/keypair.json dist/program/solfluid.so