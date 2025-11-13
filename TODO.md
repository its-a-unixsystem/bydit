# TODO & Feature Implementation Tracking

This document tracks ongoing tasks, future enhancements, and implementation plans for the `bydit` project.

## Completed Tasks

- [x] Implement overwrite functionality for comments and posts.
- [x] Add a `--yes`/`-y` flag to the delete command to skip confirmation.

## Refactor main.rs

Brief description: Improve the structure, readability, and maintainability of `src/main.rs` by extracting functionalities into separate functions and potentially modules. This effort aims to make the codebase more robust and easier to extend.

### In Progress Tasks (Refactoring `main.rs`)

- [x] **Overall `main.rs` Structure**
    - [x] Refactor the main `main()` function in `src/main.rs` to be a high-level coordinator, delegating tasks to newly extracted functions.
    - [x] Standardize error handling throughout `src/main.rs` and its extracted components using `Result<T, E>` and the `?` operator for more idiomatic Rust error management.

- [x] **Configuration Management Module**
    - [x] Design and implement a `load_configuration()` function to encapsulate reading and parsing `config.toml` (now `load_config` in `src/config.rs`).
        - *Details*: This function returns a `Result<Config, Box<dyn std::error::Error>>` and handles file reading and TOML parsing errors gracefully.
        - *Relevant Files*: `src/config.rs`.

- [x] **Reddit Client Interaction Module**
    - [x] Develop an `async fn connect_reddit_client(config: &Config, debug: bool)` to manage Reddit client initialization and login (now `connect_reddit` in `src/reddit_ops.rs`).
        - *Details*: This function abstracts the `roux::Reddit` setup and login sequence, returning a `Result<Me, Box<dyn std::error::Error>>`.
        - *Relevant Files*: `src/reddit_ops.rs`.

- [x] **Data Fetching and Processing Logic**
    - [x] Create an `async fn retrieve_and_filter_items(...)` to orchestrate the fetching of posts and comments, apply filters, and sort results (now `fetch_user_items` in `src/reddit_ops.rs`).
        - [x] Implement logic for retrieving user posts, handling pagination and conversion to `UnifiedItem` (within `fetch_user_items`).
        - [x] Implement logic for retrieving user comments, handling pagination and conversion to `UnifiedItem` (within `fetch_user_items`).
        - [x] Ensure all fetched items are consistently sorted by `created_utc` (descending).
        - [x] Integrate score-based and subreddit filtering logic.
        - *Relevant Files*: `src/reddit_ops.rs`, uses `src/models.rs`.

- [x] **Action Dispatching Module**
    - [x] Implement `async fn perform_overwrite_action(...)` (now `handle_overwrite_action` in `src/actions.rs`).
    - [x] Implement `async fn perform_delete_action(...)` (now `handle_delete_action` in `src/actions.rs`).
    - [x] Implement `fn output_items_to_csv(...)` (now `handle_csv_export` and `handle_print_to_console` in `src/actions.rs`).
        - *Relevant Files*: `src/actions.rs`.

### Future Tasks

- [x] **Enhanced Modularity**
    - [x] Separation of concerns into dedicated modules has been completed:
        - `src/config.rs`: For `Config` struct and `load_config()`.
        - `src/cli.rs`: For `Cli` struct and related command-line parsing logic.
        - `src/models.rs`: For shared data structures like `UnifiedItem`.
        - `src/reddit_ops.rs`: For Reddit API interaction functions (`connect_reddit`, `fetch_user_items`).
        - `src/actions.rs`: For user-triggered actions (`handle_overwrite_action`, `handle_delete_action`, `handle_csv_export`, `handle_print_to_console`).
        - `src/utils.rs`: For general utility functions like `escape_csv_field`.

### Refactoring Summary

The refactoring effort aimed to decompose the monolithic `main` function into distinct modules, each with specific responsibilities. This has been achieved, resulting in the following structure:

- **`src/main.rs`**: Serves as the application entry point and high-level orchestrator.
- **`src/cli.rs`**: Manages command-line argument parsing (`Cli` struct).
- **`src/config.rs`**: Handles configuration loading (`Config` struct, `load_config` function).
- **`src/models.rs`**: Defines shared data structures (`UnifiedItem`).
- **`src/reddit_ops.rs`**: Encapsulates Reddit API interactions (`connect_reddit`, `fetch_user_items`).
- **`src/actions.rs`**: Contains functions for user-triggered actions on data (`handle_overwrite_action`, `handle_delete_action`, `handle_csv_export`, `handle_print_to_console`).
- **`src/utils.rs`**: Provides utility functions (`escape_csv_field`).

This modular design enhances code organization, maintainability, and testability. Idiomatic Rust error handling (`Result<T, E>`, `?` operator) has been applied throughout.
