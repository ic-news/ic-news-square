#!/bin/bash

# IC News Square Unified Test Script
# Comprehensive test suite for all IC News Square functionality
# Including: task system, rewards, content creation, and user management

# Color definitions
GREEN='\033[0;32m'
RED='\033[0;31m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
NC='\033[0m' # No color

# Get the absolute path of the project root
PROJECT_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../" && pwd)"

# Configuration variables
NETWORK="local" # Default network
# Parse command line arguments
BASIC_ONLY=false
ADVANCED_ONLY=false
VERBOSE=false
ALT_FORMAT=false
TEST_NAME=""

while [[ $# -gt 0 ]]; do
    case $1 in
        -h|--help)
            show_help
            exit 0
            ;;
        -n|--network)
            NETWORK="$2"
            shift 2
            ;;
        -c|--canister)
            CANISTER_ID="$2"
            shift 2
            ;;
        -b|--basic)
            BASIC_ONLY=true
            shift
            ;;
        -a|--advanced)
            ADVANCED_ONLY=true
            shift
            ;;
        -u|--user)
            USER1="$2"
            shift 2
            ;;
        -v|--verbose)
            VERBOSE=true
            shift
            ;;
        --alt-format)
            ALT_FORMAT=true
            shift
            ;;
        *)
            TEST_NAME="$1"
            shift
            ;;
    esac
done

# Dynamically get daily_checkin_task canister ID based on network
get_daily_checkin_canister_id() {
    local network=$1
    local canister_ids_file="$PROJECT_ROOT/canisters/daily_checkin_task/.dfx/$network/canister_ids.json"
    
    if [ -f "$canister_ids_file" ]; then
        local canister_id=$(jq -r '.daily_checkin_task.'"$network"'' "$canister_ids_file")
        if [ "$canister_id" != "null" ]; then
            echo "$canister_id"
            return 0
        fi
    fi
    
    # Fallback to checking the main .dfx directory
    local main_canister_ids_file="$PROJECT_ROOT/.dfx/$network/canister_ids.json"
    if [ -f "$main_canister_ids_file" ]; then
        local canister_id=$(jq -r '.daily_checkin_task.'"$network"'' "$main_canister_ids_file")
        if [ "$canister_id" != "null" ]; then
            echo "$canister_id"
            return 0
        fi
    fi
    
    # If we can't find it, return a default or empty
    echo ""
    return 1
}

# Will be set after parsing arguments
# Get daily_checkin_task canister ID
DAILY_CHECKIN_CANISTER_ID=$(get_daily_checkin_canister_id "$NETWORK")
if [ -z "$DAILY_CHECKIN_CANISTER_ID" ]; then
    echo -e "${YELLOW}Warning: Could not find daily_checkin_task canister ID for network $NETWORK${NC}"
else
    echo -e "${GREEN}Found daily_checkin_task canister ID: $DAILY_CHECKIN_CANISTER_ID${NC}"
fi

echo -e "${BLUE}=== IC News Square Unified Test Suite ===${NC}"

# Check if dfx is installed
if ! command -v dfx &> /dev/null; then
    echo -e "${RED}Error: dfx is not installed, please install dfx first${NC}"
    exit 1
fi

get_ic_news_canister_id() {
    local network=$1
    local canister_ids_file="$PROJECT_ROOT/.dfx/$network/canister_ids.json"
    
    if [ -f "$canister_ids_file" ]; then
        local canister_id=$(jq -r '.ic_news_square.'"$network"'' "$canister_ids_file")
        if [ "$canister_id" != "null" ]; then
            echo "$canister_id"
            return 0
        fi
    fi
    
    # Fallback to checking the main .dfx directory
    local main_canister_ids_file="$PROJECT_ROOT/.dfx/$network/canister_ids.json"
    if [ -f "$main_canister_ids_file" ]; then
        local canister_id=$(jq -r '.ic_news_square.'"$network"'' "$main_canister_ids_file")
        if [ "$canister_id" != "null" ]; then
            echo "$canister_id"
            return 0
        fi
    fi
    
    # If we can't find it, return a default or empty
    echo ""
    return 1
}


# Get ic_news_square canister ID
CANISTER_ID=$(get_ic_news_canister_id "$NETWORK")
if [ -z "$CANISTER_ID" ]; then
    echo -e "${RED}Error: Could not find ic_news_square canister ID for network $NETWORK${NC}"
    echo -e "${YELLOW}Tests will fail without a valid canister ID${NC}"
    exit 1
else
    echo -e "${GREEN}Found ic_news_square canister ID: $CANISTER_ID${NC}"
fi

DFX="dfx canister --network $NETWORK call $CANISTER_ID" # Simplified call command

# User identities
USER1="ic-news" # Default user
USER2="ic-news-2" # Second user for multi-user tests
ADMIN_IDENTITY="ic-news" # Admin identity (using default user for testing)

# Constants for testing
SECONDS_IN_DAY=86400 # Seconds in a day for testing task expiration

# Source common functions
source_dir=$(dirname "$0")
source "$source_dir/test_functions.sh"

# Display help information
show_help() {
    echo -e "${BLUE}IC News Square Test Suite Usage:${NC}"
    echo -e "  $0 [options] [test_name]"
    echo -e "\nOptions:"
    echo -e "  -h, --help                 Show this help message"
    echo -e "  -n, --network NETWORK      Set network (local, ic)"
    echo -e "  -c, --canister CANISTER_ID Set canister ID"
    echo -e "  -b, --basic                Run basic tests only"
    echo -e "  -a, --advanced             Run advanced tests only"
    echo -e "  -u, --user USER            Set primary test user identity"
    echo -e "  -v, --verbose              Enable verbose output"
    echo -e "\nAvailable basic tests:"
    echo -e "  register, user_profile, user_rewards, available_tasks, daily_post,"
    echo -e "  social_engagement, nested_comments, task_repetition, admin_reward,"
    echo -e "  points_accumulation, error_handling, multi_user"
    echo -e "\nAvailable advanced tests:"
    echo -e "  custom_task, expiration, chaining, reset, bulk, leaderboard, checkin"
    echo -e "\nExamples:"
    echo -e "  $0                         Run all tests"
    echo -e "  $0 -b                      Run basic tests only"
    echo -e "  $0 -a                      Run advanced tests only"
    echo -e "  $0 daily_post              Run daily_post test only"
    echo -e "  $0 -n ic -c gkx4d-myaaa-aaaag-at72q-cai  Run on IC mainnet"
}



# Update DFX command with selected network and canister
DFX="dfx canister --network $NETWORK call $CANISTER_ID"
export DFX_NETWORK=$NETWORK

# PROJECT_ROOT is already set at the beginning of the script

# Get daily_checkin_task canister ID based on network
if [ -z "$DAILY_CHECKIN_CANISTER_ID" ]; then
    echo -e "${YELLOW}Warning: Could not find daily_checkin_task canister ID for network $NETWORK${NC}"
    echo -e "${YELLOW}Daily check-in tests may fail${NC}"
else
    echo -e "${BLUE}Using daily_checkin_task canister ID: ${DAILY_CHECKIN_CANISTER_ID} on network: ${NETWORK}${NC}"
fi

echo -e "${BLUE}Using canister ID: ${CANISTER_ID} on network: ${NETWORK}${NC}"

# Main function
main() {
    # Setup test environment
    setup_test_environment
    
    # Run specific test if provided
    if [ -n "$TEST_NAME" ]; then
        if [ "$ALT_FORMAT" = true ]; then
            run_specific_test "$TEST_NAME" "alternative"
        else
            run_specific_test "$TEST_NAME"
        fi
        exit 0
    fi
    
    # Run all tests based on options
    if [ "$BASIC_ONLY" = true ] && [ "$ADVANCED_ONLY" = true ]; then
        echo -e "${RED}Error: Cannot specify both --basic and --advanced${NC}"
        exit 1
    elif [ "$BASIC_ONLY" = true ]; then
        run_basic_tests
    elif [ "$ADVANCED_ONLY" = true ]; then
        run_advanced_tests
    else
        run_all_tests
    fi
}

# Execute main function
main
