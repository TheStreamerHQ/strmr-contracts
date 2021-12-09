#!/bin/bash
set -e
cargo build --all --target wasm32-unknown-unknown --release
cp target/wasm32-unknown-unknown/release/*.wasm ./res/
# delete account
near-cli delete account network custom --url http://localhost:3030/ account streamer.test.near beneficiary test.near sign-with-keychain send || true
rm ~/.near-credentials/default/streamer.test.near.json
# create account
near-cli add sub-account network custom --url http://localhost:3030/ owner-account test.near sub-account streamer.test.near sub-account-full-access generate-keypair deposit '100 NEAR' sign-with-keychain send
sleep 3
# deploy contract and initiate
near-cli add contract-code network custom --url http://localhost:3030/ account streamer.test.near contract-file ./res/thestreamer_contract.wasm initialize new '{"owner_id": "test.near", "exchange_price_in_yocto_near": "1000000000000000000000000", "total_supply": "100", "metadata": {"spec": "ft-1.0.0", "name": "Streamer", "symbol": "STRMR", "decimals": 0}}' --attached-deposit '0 NEAR' --prepaid-gas '200.000 TeraGas' sign-with-keychain send
#near-cli add contract-code network custom --url http://localhost:3030/ account streamer2.test.near contract-file ./res/thestreamer_contract.wasm no-initialize sign-with-keychain send
