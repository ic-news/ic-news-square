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

canister_name="daily_checkin_task"
# Already in the correct directory, no need to enter the subdirectory
cargo clean
cargo build --target wasm32-unknown-unknown --release
candid-extractor "target/wasm32-unknown-unknown/release/$canister_name.wasm" > "src/$canister_name.did"

# Deploy canister
# 检查是否已经部署过 canister
if [ -f ".dfx/$NETWORK/canister_ids.json" ] && [ "$(jq -r ".$canister_name.$NETWORK" ".dfx/$NETWORK/canister_ids.json")" != "null" ]; then
    echo "Updating existing canister with --mode reinstall..."
    # 更新已存在的 canister，使用 --mode reinstall 保留数据
    dfx deploy --network "$NETWORK" --mode reinstall
else
    echo "First time deployment, using normal deploy..."
    # 首次部署，使用普通模式
    dfx deploy --network "$NETWORK"
fi