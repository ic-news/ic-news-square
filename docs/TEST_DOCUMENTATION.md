# IC News Square - Test Documentation

## Overview

This document provides a comprehensive guide to the testing infrastructure for the IC News Square project. It covers the test framework, available test cases, and instructions for running tests.

## Test Structure

The testing infrastructure is organized as follows:

```
tests/
├── advanced_tests.sh      # Advanced test scenarios
├── basic_tests.sh         # Basic functionality tests
├── error_test.rs          # Rust-based error testing
├── ic_news_test.sh        # Main test orchestration script
└── test_functions.sh      # Core test functions
```

## Test Framework

The test framework is primarily shell-based, using `dfx` commands to interact with the deployed canisters. The main components are:

1. **ic_news_test.sh**: The entry point for running tests, which parses command-line arguments and orchestrates test execution.
2. **test_functions.sh**: Contains individual test functions for different features.
3. **basic_tests.sh** and **advanced_tests.sh**: Organize tests into basic and advanced categories.

## Available Test Cases

### Basic Tests

- **register_user**: Tests user registration functionality.
- **user_profile**: Tests retrieving user profile information.
- **user_rewards**: Tests the user rewards system.
- **available_tasks**: Tests retrieving available tasks.
- **create_post**: Tests creating posts.
- **social_engagement**: Tests social interactions like liking and commenting on posts.
- **nested_comments**: Tests creating comments on comments (nested comments).

### Advanced Tests

Advanced tests cover more complex scenarios and edge cases. Refer to `advanced_tests.sh` for details.

## Running Tests

### Prerequisites

1. Make sure you have `dfx` installed and configured.
2. Deploy the IC News Square canister locally or on the IC network.

### Basic Usage

```bash
./tests/ic_news_test.sh [options] [test_name]
```

### Command-line Options

- `--help`: Display help information.
- `--network <network>`: Specify the network (local or ic).
- `--canister-id <id>`: Specify the canister ID.
- `--alt-format`: Use alternative format for Candid serialization (useful during refactoring).

### Examples

Run all basic tests:
```bash
./tests/ic_news_test.sh
```

Run a specific test:
```bash
./tests/ic_news_test.sh social_engagement
```

Run with alternative Candid format:
```bash
./tests/ic_news_test.sh --alt-format social_engagement
```

## Candid Serialization

The tests use Candid serialization to format data for API calls. During project refactoring, type definitions may change, which can affect serialization. The test framework provides two approaches:

1. **Standard Format**: The default format used for Candid serialization.
   ```
   record {
       id = opt "comment_id";
       content = "Comment content";
       parent_id = "post_id";
       parent_type = variant { Post };
   }
   ```

2. **Alternative Format**: A fallback format that can be used when type definitions change.
   ```
   record {
       id = opt "comment_id";
       content = "Comment content";
       parent_id = "post_id";
       parent_type = "Post";
   }
   ```

Use the `--alt-format` flag to switch between these formats when needed.

## Testing Nested Comments

The `test_nested_comments` function tests the ability to create comments on other comments. This ensures that the `parent_type = variant { Comment }` format works correctly, similar to the `parent_type = variant { Post }` format used for comments on posts.

## Error Handling

The test framework includes error handling through the `check_result` function, which validates the results of API calls and provides clear error messages when tests fail.

## Best Practices

1. **Isolation**: Each test function should be self-contained and not depend on the state created by other tests.
2. **Cleanup**: Tests should clean up any resources they create.
3. **Descriptive Names**: Use descriptive names for test functions and variables.
4. **Error Messages**: Provide clear error messages when tests fail.
5. **Type Consistency**: Be aware of Candid serialization formats, especially during refactoring.

## Troubleshooting

### Common Issues

1. **"record field parent_type not found" Error**:
   - This error occurs when the Candid serialization format doesn't match the expected format.
   - Try using the alternative format with the `--alt-format` flag.

2. **Authentication Issues**:
   - Ensure that the correct identity is being used for each test.
   - Use the `switch_identity` function to switch between test identities.

3. **Network Issues**:
   - Verify that the specified network is accessible.
   - Check that the canister ID is correct for the specified network.

### Debugging Tips

1. Use the `echo` command to print debug information.
2. Check the output of `dfx canister call` commands for error messages.
3. Inspect the canister logs using `dfx canister log`.

## Extending the Test Framework

To add new tests:

1. Create a new test function in `test_functions.sh`.
2. Add the test to the `run_specific_test` function in `test_functions.sh`.
3. Update the help message in `ic_news_test.sh` to include the new test.

## Conclusion

This test documentation provides a comprehensive guide to the IC News Square testing infrastructure. By following the guidelines and best practices outlined here, you can ensure that your tests are effective, maintainable, and provide good coverage of the application's functionality.
