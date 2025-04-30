#!/bin/bash
dfx identity use ic-news
# Configuration variables
daily_canister_name="daily_checkin_task"

NETWORK="local"  # Default network is local

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

# Get the absolute path of the project root
PROJECT_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"

cd "$PROJECT_ROOT/canisters/$daily_canister_name"
# Deploy daily_checkin_task canister
./scripts/deploy.sh --network "$NETWORK"

# Get and display canister ID
echo "Getting canister ID information..."
if [ -f "$PROJECT_ROOT/canisters/$daily_canister_name/canister_ids.json" ]; then
    # Get daily_checkin_task canister ID
    DAILY_CHECKIN_TASK_CANISTER_ID=$(jq -r ".$daily_canister_name.$NETWORK" "$PROJECT_ROOT/canisters/$daily_canister_name/canister_ids.json")
    
    if [ "$DAILY_CHECKIN_TASK_CANISTER_ID" != "null" ]; then
        echo "$daily_canister_name canister ID ($NETWORK): $DAILY_CHECKIN_TASK_CANISTER_ID"
        echo "You can use the following command to interact with the canister:"
        echo "dfx canister --network $NETWORK call $DAILY_CHECKIN_TASK_CANISTER_ID <method_name> <args>"
        
        # Also save the ID to an environment variable for other scripts
        export DAILY_CHECKIN_TASK_CANISTER_ID=$DAILY_CHECKIN_TASK_CANISTER_ID
        echo "Saved canister ID to environment variable DAILY_CHECKIN_TASK_CANISTER_ID"
    else
        echo "Warning: Could not find $daily_canister_name canister ID"
    fi
else
    echo "Warning: Could not find canister ID file (.dfx/$NETWORK/canister_ids.json)"
fi

echo "daily_checkin_task canister ID ($NETWORK): $DAILY_CHECKIN_TASK_CANISTER_ID"
# Return to main project directory
cd "$PROJECT_ROOT"

# Deploy main project
canister_name="ic_news_square"

# 构建步骤
# 保留 cargo clean 命令，因为在某些情况下它是必要的
cargo clean
cargo build --target wasm32-unknown-unknown --release
candid-extractor "target/wasm32-unknown-unknown/release/ic_news_square.wasm" > "src/$canister_name.did"

# 使用 --mode upgrade 参数强制执行升级流程而不是重新部署
# 这样可以确保 pre_upgrade 和 post_upgrade 钩子函数被调用，数据得以保存
dfx deploy --network "$NETWORK" --mode upgrade

# Bind daily_checkin_task canister
if [ -n "$DAILY_CHECKIN_TASK_CANISTER_ID" ]; then
    echo "Binding $daily_canister_name canister..."
    # Pass the Principal formatted canister ID to the create_task method
    dfx canister --network "$NETWORK" call $canister_name create_task '(
        record {
            id = "daily_checkin";
            title = "Daily Check-in";
            description = "Check-in every day";
            points_reward = 10;
            task_type = variant { Daily };
            canister_id = principal "'$DAILY_CHECKIN_TASK_CANISTER_ID'";
            start_time = null;
            end_time = null;
            completion_criteria = "Daily check-in";
            requirements = opt record {
                social_interaction = opt record {
                    like_count = opt (0 : nat64);
                    follow_count = opt (0 : nat64);
                };
                required_tokens = opt vec {};
                required_nfts = opt vec {};
                login_streak = opt record { days_required = 1 : nat64 };
                custom_requirements = opt vec {};
                content_creation = opt record {
                    comment_count = opt (0 : nat64);
                    post_count = opt (0 : nat64);
                };
            }
        }
    )'
    
    if [ $? -eq 0 ]; then
        echo "Successfully bound $daily_canister_name canister"
    else
        echo "Failed to bind $daily_canister_name canister"
    fi
fi