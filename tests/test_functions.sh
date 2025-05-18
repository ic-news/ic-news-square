#!/bin/bash

# IC News Square Test Functions
# Common functions used by the unified test script

# Switch identity and get principal
switch_identity() {
    local identity=$1
    dfx identity use $identity
    echo -e "${BLUE}Current identity: $(dfx identity whoami)${NC}"
}

get_principal() {
    dfx identity get-principal
}

# Check call result
check_result() {
    local result="$1"
    local step="$2"
    local allow_continue=${3:-false}  # Third parameter: whether to allow continuing execution, default is false
    
    if [[ $result == *"Err"* ]]; then
        echo -e "${RED}Failed: $step${NC}"
        echo -e "${RED}$result${NC}"
        if [[ "$allow_continue" != "true" ]]; then
            exit 1
        fi
    else
        echo -e "${GREEN}Success: $step${NC}"
        if [ "$VERBOSE" = true ]; then
            echo -e "${GREEN}$result${NC}"
        fi
    fi
}

# Setup test environment
setup_test_environment() {
    # Ensure test identities exist
    if ! dfx identity use $USER1 &>/dev/null; then
        echo -e "${YELLOW}Creating primary test identity: $USER1${NC}"
        dfx identity new $USER1 --storage-mode=plaintext || {
            echo -e "${RED}Could not create primary identity${NC}"
            exit 1
        }
    fi
    
    if ! dfx identity use $USER2 &>/dev/null; then
        echo -e "${YELLOW}Creating secondary test identity: $USER2${NC}"
        dfx identity new $USER2 --storage-mode=plaintext || {
            echo -e "${YELLOW}Could not create secondary identity, some tests may fail${NC}"
        }
    fi
    
    # Switch back to primary user
    switch_identity $USER1
}

# Test registering a user
test_register_user() {
    echo -e "\n${BLUE}Test: Registering user${NC}"
    
    switch_identity $USER1
    local principal=$(get_principal)
    
    echo -e "${YELLOW}Registering user: $principal${NC}"
    
    local register_user_request="(
        record {
            username = \"IC News User\";
            handle = \"icnews\";
            bio = \"Test user for IC News Square\";
            interests = opt vec { \"news\"; \"technology\"; \"blockchain\" };
            social_links = opt vec {};
            avatar = \"https://example.com/avatar.png\";
        }
    )"
    
    local result=$($DFX register_user "$register_user_request")
    check_result "$result" "Registering user" true
}

# Test get_user_profile API
test_get_user_profile() {
    echo -e "\n${BLUE}Test: Getting user profile${NC}"
    
    switch_identity $USER1
    local principal=$(get_principal)
    
    echo -e "${YELLOW}Getting profile for user: $principal${NC}"
    
    # Call get_user_profile API
    local result=$($DFX get_user_profile)
    check_result "$result" "Getting user profile" true
    
    # Check if the response is properly structured (data is an object, not an array)
    if [[ $result == *"data = opt record"* && $result != *"data = opt vec"* ]]; then
        echo -e "${GREEN}Response format check: Success - Data is an object (record), not an array${NC}"
    else
        echo -e "${RED}Response format check: Failed - Data format is unexpected${NC}"
        echo -e "${RED}$result${NC}"
    fi
    
    # Additional check to ensure data is not returned as an array
    if [[ $result == *"data = opt vec"* ]]; then
        echo -e "${RED}ERROR: Data is returned as an array (vec) instead of an object (record)${NC}"
        echo -e "${RED}$result${NC}"
    fi
    
    # Verify specific fields exist in the response
    if [[ $result == *"username"* && $result == *"handle"* && $result == *"bio"* ]]; then
        echo -e "${GREEN}Field verification: Success - Required fields found in response${NC}"
    else
        echo -e "${RED}Field verification: Failed - Some required fields are missing${NC}"
        echo -e "${RED}$result${NC}"
    fi
}

# Test getting current user points
test_get_user_rewards() {
    echo -e "\n${BLUE}Test: Getting user points${NC}"
    
    switch_identity $USER1
    local principal=$(get_principal)
    
    echo -e "${YELLOW}Getting user points: $principal${NC}"
    
    local result=$($DFX get_user_rewards "(principal \"$principal\")")
    
    # Allow continuing execution even if the result contains errors
    check_result "$result" "Getting user points" true
    
    # Return result for subsequent analysis
    echo "$result"
}

# Test getting available tasks
test_get_available_tasks() {
    echo -e "\n${BLUE}Test: Getting available tasks${NC}"
    
    switch_identity $USER1
    local principal=$(get_principal)
    
    echo -e "${YELLOW}Getting available tasks for user: $principal${NC}"
    
    local result=$($DFX get_available_tasks "(principal \"$principal\")")
    check_result "$result" "Getting available tasks"
    
    # Check if default tasks are included
    if [[ $result == *"daily_post"* ]]; then
        echo -e "${GREEN}Found default task: daily_post${NC}"
    else
        echo -e "${RED}Default task 'daily_post' not found${NC}"
    fi
    
    if [[ $result == *"social_engagement"* ]]; then
        echo -e "${GREEN}Found default task: social_engagement${NC}"
    else
        echo -e "${RED}Default task 'social_engagement' not found${NC}"
    fi
    
    # Return the result for further processing
    echo "$result"
}

# Test creating a post
test_create_post() {
    echo -e "\n${BLUE}Test: Creating a post${NC}"
    
    switch_identity $USER1
    local principal=$(get_principal)
    
    local content="Test post - $(date)"
    local post_id="post_$(date +%s)"
    
    echo -e "${YELLOW}Creating post ID: $post_id${NC}"
    
    local create_post_request="(
        record {
            id = opt \"$post_id\";
            content = \"$content\";
            hashtags = vec { \"test\" };
            mentions = opt vec {};
            media_urls = vec {};
            is_nsfw = opt false;
            tags = opt vec {};
            token_mentions = opt vec {};
            visibility = opt variant { Public };
        }
    )"
    
    local result=$($DFX create_post "$create_post_request")
    check_result "$result" "Creating post"
    
    # Return post ID for later use
    echo "$post_id"
}

# Test social engagement (likes and comments)
test_social_engagement() {
    echo -e "\n${BLUE}Test: Social engagement${NC}"
    
    switch_identity $USER1
    local principal=$(get_principal)
    
    # First create a post for interaction
    local post_id=$(test_create_post)
    
    # Like the post
    echo -e "${YELLOW}Liking post: $post_id${NC}"
    # Use simple ID format to avoid serialization issues
    local simple_post_id="post_simple_123"
    local like_request="(record { content_id = \"$simple_post_id\"; content_type = variant { Post }; })"
    
    # Create a simple post for liking
    local create_simple_post_request="(
        record {
            id = opt \"$simple_post_id\";
            content = \"Simple test post for liking\";
            hashtags = vec { \"test\" };
            mentions = opt vec {};
            media_urls = vec {};
            is_nsfw = opt false;
            tags = opt vec {};
            token_mentions = opt vec {};
            visibility = opt variant { Public };
        }
    )"
    
    $DFX create_post "$create_simple_post_request" > /dev/null
    
    # Like the post
    local result=$($DFX like_content "$like_request")
    check_result "$result" "Liking post" true
    
    # Comment on the post
    echo -e "${YELLOW}Commenting on post: $simple_post_id${NC}"
    local comment_id="comment_simple_123"
    local comment_request="(
        record {
            id = opt \"$comment_id\";
            content = \"This is a test comment\";
            parent_id = \"$simple_post_id\";
            parent_type = variant { Post };
        }
    )"
    
    # Alternative format is provided as a fallback in case the standard format doesn't work
    # This might be useful during project refactoring when type definitions change
    if [ "$1" == "alternative" ]; then
        comment_request="(
            record {
                id = opt \"$comment_id\";
                content = \"This is a test comment\";
                parent_id = \"$simple_post_id\";
                parent_type = \"Post\";
            }
        )"
    fi
    
    local result=$($DFX create_comment "$comment_request")
    check_result "$result" "Commenting on post" true
    
    # Return comment ID for later use
    echo "$comment_id"
}

# Test nested comments (comments on comments)
test_nested_comments() {
    echo -e "\n${BLUE}Test: Nested comments${NC}"
    
    switch_identity $USER1
    local principal=$(get_principal)
    
    # First create a post
    local post_id="post_nested_test"
    local create_post_request="(
        record {
            id = opt \"$post_id\";
            content = \"Test post for nested comments\";
            hashtags = vec { \"test\" };
            mentions = opt vec {};
            media_urls = vec {};
            is_nsfw = opt false;
            tags = opt vec {};
            token_mentions = opt vec {};
            visibility = opt variant { Public };
        }
    )"
    
    $DFX create_post "$create_post_request" > /dev/null
    
    # Create a parent comment
    echo -e "${YELLOW}Creating parent comment on post: $post_id${NC}"
    local parent_comment_id="comment_parent_123"
    local parent_comment_request="(
        record {
            id = opt \"$parent_comment_id\";
            content = \"This is a parent comment\";
            parent_id = \"$post_id\";
            parent_type = variant { Post };
        }
    )"
    
    local result=$($DFX create_comment "$parent_comment_request")
    check_result "$result" "Creating parent comment" true
    
    # Create a child comment (comment on comment)
    echo -e "${YELLOW}Creating child comment on comment: $parent_comment_id${NC}"
    local child_comment_id="comment_child_123"
    local child_comment_request="(
        record {
            id = opt \"$child_comment_id\";
            content = \"This is a reply to another comment\";
            parent_id = \"$parent_comment_id\";
            parent_type = variant { Comment };
        }
    )"
    
    local result=$($DFX create_comment "$child_comment_request")
    check_result "$result" "Creating child comment" true
    
    # Return child comment ID for later use
    echo "$child_comment_id"
}

# Include basic test functions
source "$source_dir/basic_tests.sh"

# Include advanced test functions
source "$source_dir/advanced_tests.sh"

# Run all tests
run_all_tests() {
    echo -e "\n${BLUE}=== Running all tests ===${NC}"
    
    run_basic_tests
    run_advanced_tests
    
    echo -e "\n${GREEN}=== All tests completed successfully ===${NC}"
}

# Run specific test
run_specific_test() {
    local test_name=$1
    local format=${2:-"standard"}
    echo -e "\n${GREEN}Running test: $test_name (format: $format)${NC}"
    
    case $test_name in
        register_user)
            test_register_user
            ;;
        user_profile)
            test_get_user_profile
            ;;
        user_rewards)
            test_get_user_rewards
            ;;
        available_tasks)
            test_get_available_tasks
            ;;
        create_post)
            test_create_post
            ;;
        social_engagement)
            if [ "$format" == "alternative" ]; then
                test_social_engagement "alternative"
            else
                test_social_engagement
            fi
            ;;
        nested_comments)
            test_nested_comments
            ;;
        *)
            echo -e "${RED}Unknown test: $test_name${NC}"
            exit 1
            ;;
    esac
}
