# IC News Square - Project Documentation

## Overview

IC News Square is a decentralized social media platform built on the Internet Computer (IC) blockchain. The platform enables users to create posts, engage with content through likes and comments (including nested comments), earn rewards through daily check-ins and task completion, and participate in a social community.

## Architecture

### Canister Structure

The project is organized as a Rust-based canister with the following structure:

```
ic-news-square/
├── src/
│   ├── auth/           # Authentication-related functionality
│   ├── models/         # Data models and structures
│   ├── services/       # Business logic services
│   ├── storage/        # Data storage and persistence
│   ├── utils/          # Utility functions
│   ├── tests/          # Internal unit tests
│   ├── lib.rs          # Main canister code
│   └── ic_news_square.did  # Candid interface definition
├── tests/              # Integration tests
├── canisters/          # Additional canisters (e.g., daily_checkin_task)
└── scripts/            # Deployment and utility scripts
```

### Key Components

1. **Authentication System**: Manages user identities and permissions.
2. **Content Management**: Handles creation, retrieval, and management of posts and comments.
3. **Social Engagement**: Manages likes, comments, and other social interactions.
4. **Reward System**: Manages user points, tasks, and rewards.

## Data Models

### Core Models

- **User**: Represents a user profile with personal information and social stats.
- **Post**: Represents user-generated content with text, media, hashtags, and engagement metrics.
- **Comment**: Represents user comments on posts or other comments.
- **Like**: Represents a user's like on a post or comment.
- **Task**: Represents activities users can complete to earn rewards.

### Parent Types

Content can have different parent types, which determine the relationship between content items:

- **Post**: A top-level content item.
- **Comment**: A response to a post or another comment.

## API Endpoints

The canister exposes various endpoints for interacting with the platform:

### User Management

- `register_user`: Register a new user.
- `get_user_profile`: Retrieve a user's profile information.
- `update_user_profile`: Update a user's profile information.

### Content Management

- `create_post`: Create a new post.
- `get_post`: Retrieve a specific post.
- `get_posts`: Retrieve multiple posts based on criteria.
- `delete_post`: Delete a post.

### Social Engagement

- `create_comment`: Create a comment on a post or another comment.
- `get_comments`: Retrieve comments for a post or comment.
- `like_content`: Like a post or comment.
- `unlike_content`: Remove a like from a post or comment.

### Rewards and Tasks

- `get_available_tasks`: Retrieve available tasks for a user.
- `complete_task`: Mark a task as completed.
- `get_user_rewards`: Retrieve a user's reward information.

## Candid Serialization

The project uses Candid for serializing data between the frontend and backend. The Candid interface is defined in `src/ic_news_square.did`.

### Important Types

- **ParentType**: A variant type that can be either `Post` or `Comment`.
- **ContentStatus**: Represents the status of content (e.g., active, deleted).
- **ContentVisibility**: Determines who can see the content.

### Serialization Formats

During project refactoring, type definitions may change, affecting serialization. The project supports two formats:

1. **Variant Format**:

   ```
   parent_type = variant { Post };
   ```

2. **String Format**:
   ```
   parent_type = "Post";
   ```

## Testing

The project includes a comprehensive testing framework:

- **Unit Tests**: Located in `src/tests/`.
- **Integration Tests**: Located in `tests/`.

See the [Test Documentation](./TEST_DOCUMENTATION.md) for detailed information on running tests.

## Development Guide

### Prerequisites

1. Install the DFINITY Canister SDK (dfx).
2. Install Rust and Cargo.
3. Install pnpm for package management.

### Local Development

1. Clone the repository:

   ```bash
   git clone <repository-url>
   cd ic-news-square
   ```

2. Install dependencies:

   ```bash
   pnpm install
   ```

3. Start the local IC replica:

   ```bash
   dfx start --background
   ```

4. Deploy the canister locally:
   ```bash
   dfx deploy
   ```

### Running Tests

Run the test suite:

```bash
./tests/ic_news_test.sh
```

For more detailed testing instructions, see the [Test Documentation](./TEST_DOCUMENTATION.md).

### Code Style and Best Practices

1. **Error Handling**: Use the `ApiError` and `SquareError` types for consistent error reporting.
2. **Type Safety**: Leverage Rust's type system and Candid's serialization to ensure type safety.
3. **Documentation**: Document functions and types with clear comments.
4. **Testing**: Write tests for new functionality and ensure existing tests pass.

### Deployment

Deploy to the IC mainnet:

```bash
dfx deploy --network ic
```

## Social Engagement Features

### Posts

Users can create posts with text content, hashtags, mentions, and media attachments. Posts can be liked and commented on by other users.

### Comments

Comments can be added to posts or to other comments (nested comments). The `parent_type` field in the `CreateCommentRequest` determines whether the comment is on a post or another comment.

#### Creating a Comment on a Post

```
record {
    id = opt "comment_id";
    content = "Comment content";
    parent_id = "post_id";
    parent_type = variant { Post };
}
```

#### Creating a Comment on a Comment

```
record {
    id = opt "child_comment_id";
    content = "Reply to comment";
    parent_id = "parent_comment_id";
    parent_type = variant { Comment };
}
```

### Likes

Users can like posts and comments. The system tracks like counts and whether a specific user has liked a piece of content.

## Troubleshooting

### Common Issues

1. **Candid Serialization Errors**:

   - If you encounter "record field not found" errors, check the Candid serialization format.
   - During refactoring, you may need to use alternative serialization formats.

2. **Authentication Issues**:

   - Ensure you're using the correct identity for API calls.
   - Check that the user is registered before attempting other operations.

3. **Storage Issues**:
   - If data isn't being stored or retrieved correctly, check the sharding logic.
   - Verify that the correct shard is being accessed for the given content.

## Future Development

Potential areas for future development include:

1. **Enhanced Media Support**: Improved handling of images, videos, and other media types.
2. **Advanced Search**: More sophisticated search capabilities for content discovery.
3. **Moderation Tools**: Tools for community moderation and content filtering.
4. **Integration with Other Services**: Integration with other IC canisters and services.

## Conclusion

IC News Square provides a robust platform for decentralized social media on the Internet Computer. By following the guidelines in this documentation, developers can effectively contribute to and extend the platform's functionality.
