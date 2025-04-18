#!/bin/bash

# Configuration variables
NETWORK="local"  # Default network is local
# Get the absolute path of the project root
PROJECT_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"

# Parse command line arguments
while [[ "$#" -gt 0 ]]; do
    case $1 in
        --network)
            NETWORK="$2"
            shift 2
            ;;
        *)
            echo "Unknown parameter: $1"
            exit 1
            ;;
    esac
done

# Validate network parameter
if [[ "$NETWORK" != "local" && "$NETWORK" != "ic" ]]; then
    echo "Error: Network parameter must be 'local' or 'ic'"
    echo "Usage: ./deploy.sh --network [local|ic]"
    exit 1
fi

echo "Deploying to network: $NETWORK"

daily_canister_name="daily_checkin_task"
# Already in the correct directory, no need to enter the subdirectory
cargo clean
cargo build --target wasm32-unknown-unknown --release
candid-extractor "target/wasm32-unknown-unknown/release/$daily_canister_name.wasm" > "src/$daily_canister_name.did"

# Deploy canister
dfx deploy --network "$NETWORK"
