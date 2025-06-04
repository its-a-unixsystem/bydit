# TODO & Feature Implementation Tracking

This document tracks ongoing tasks, future enhancements, and implementation plans for the `bydit` project.

## Completed Tasks

- [x] Implement overwrite functionality for comments and posts.
- [x] Add a `--yes`/`-y` flag to the delete command to skip confirmation.

## Refactor main.rs

Brief description: Improve the structure, readability, and maintainability of `src/main.rs` by extracting functionalities into separate functions and potentially modules. This effort aims to make the codebase more robust and easier to extend.

### In Progress Tasks (Refactoring `main.rs`)

- [ ] **Overall `main.rs` Structure**
    - [ ] Refactor the main `main()` function in `src/main.rs` to be a high-level coordinator, delegating tasks to newly extracted functions.
    - [ ] Standardize error handling throughout `src/main.rs` and its extracted components using `Result<T, E>` and the `?` operator for more idiomatic Rust error management.

- [ ] **Configuration Management Module**
    - [ ] Design and implement a `load_configuration()` function to encapsulate reading and parsing `config.toml`.
        - *Details*: This function should return a `Result<Config, Box<dyn std::error::Error>>` and handle file reading and TOML parsing errors gracefully.
        - *Relevant Files*: `src/main.rs` (initially), consider moving to `src/config.rs`.

- [ ] **Reddit Client Interaction Module**
    - [ ] Develop an `async fn connect_reddit_client(config: &Config, debug: bool)` to manage Reddit client initialization and login.
        - *Details*: This function will abstract the `roux::Reddit` setup and login sequence, returning a `Result<Reddit, Box<dyn std::error::Error>>`.
        - *Relevant Files*: `src/main.rs` (initially), consider moving to `src/reddit_ops.rs`.

- [ ] **Data Fetching and Processing Logic**
    - [ ] Create an `async fn retrieve_and_filter_items(reddit: &Reddit, username: &str, cli_args: &Cli)` to orchestrate the fetching of posts and comments, apply filters, and sort results.
        - [ ] Implement `async fn fetch_user_submissions(...)` for retrieving user posts, handling pagination and conversion to `UnifiedItem`.
        - [ ] Implement `async fn fetch_user_comments_data(...)` for retrieving user comments, handling pagination and conversion to `UnifiedItem`.
        - [ ] Ensure all fetched items are consistently sorted by `created_utc` (descending) within `retrieve_and_filter_items`.
        - [ ] Integrate score-based filtering logic (from `cli_args.score`) directly into `retrieve_and_filter_items`.
        - *Relevant Files*: `src/main.rs` (initially), consider moving to `src/reddit_ops.rs` and using structs from `src/models.rs`.

- [ ] **Action Dispatching Module**
    - [ ] Implement `async fn perform_overwrite_action(...)` to handle the logic for overwriting item content.
    - [ ] Implement `async fn perform_delete_action(...)` to manage item deletion, including confirmation prompts (unless `-y` is used) and result reporting.
    - [ ] Implement `fn output_items_to_csv(...)` for formatting and printing `UnifiedItem` data to standard output as CSV.
        - *Relevant Files*: `src/main.rs` (initially), consider moving to `src/actions.rs`.

### Future Tasks (Post-Refactoring)

- [ ] **Enhanced Modularity**
    - [ ] Evaluate and implement the separation of concerns into dedicated modules:
        - `src/config.rs`: For `Config` struct and `load_configuration()`.
        - `src/cli.rs`: For `Cli` struct and related command-line parsing logic.
        - `src/models.rs`: For shared data structures like `UnifiedItem`.
        - `src/reddit_ops.rs`: For Reddit API interaction functions (`connect_reddit_client`, `fetch_user_submissions`, `fetch_user_comments_data`).
        - `src/actions.rs`: For user-triggered actions (`perform_overwrite_action`, `perform_delete_action`, `output_items_to_csv`).
        - `src/utils.rs`: For general utility functions like `escape_csv_field`.
- [ ] **Comprehensive Testing**
    - [ ] Develop unit tests for all newly extracted functions to ensure their correctness and facilitate future refactoring.
    - [ ] Explore integration tests for key workflows (e.g., fetching and deleting, fetching and overwriting).

### Implementation Plan for `main.rs` Refactoring

The primary objective of this refactoring effort is to decompose the monolithic `main` function within `src/main.rs`. This will be achieved by systematically extracting distinct blocks of logic into separate, well-defined functions. Each new function will target a specific responsibility, such as loading application configuration, establishing a connection with the Reddit API, fetching and processing user data (posts and comments), or executing specific user-requested actions like overwriting content, deleting items, or exporting data to CSV format.

A key aspect of this refactoring will be the adoption of idiomatic Rust error handling patterns. All functions that can potentially fail will return `Result<T, E>`, typically using `Box<dyn std::error::Error>` for flexible error propagation. The `?` operator will be used extensively to simplify error handling chains.

The refactored `main` function will serve as a high-level orchestrator, parsing command-line arguments and then invoking the appropriate sequence of newly created functions to fulfill the user's request. Debug logging, controlled by the `cli.debug` flag, will be maintained and integrated into the new functions to provide visibility into their operations.

Once the initial functional decomposition within `main.rs` is complete, a subsequent phase will evaluate the benefits of moving these functions and related data structures into separate modules (e.g., `config.rs`, `reddit_ops.rs`, `actions.rs`, `models.rs`, `utils.rs`) to further enhance code organization and maintainability. This modularization will also simplify the process of writing targeted unit tests for individual components.

#### Relevant Files (Initial & Potential Future State)

- **`src/main.rs`**: The central file undergoing refactoring. It will initially house the new functions and the modified `main` orchestrator.
- **`src/config.rs`** (Potential): To house `Config` struct and `load_configuration()` function.
- **`src/cli.rs`** (Potential): For the `Cli` struct and any command-line argument processing utilities.
- **`src/models.rs`** (Potential): For shared data structures like `UnifiedItem` and potentially `Config` and `Cli` if not in their own modules.
- **`src/reddit_ops.rs`** (Potential): For all functions directly interacting with the Reddit API (e.g., `connect_reddit_client`, data fetching functions).
- **`src/actions.rs`** (Potential): For functions that execute specific user commands (e.g., `perform_overwrite_action`, `perform_delete_action`, `output_items_to_csv`).
- **`src/utils.rs`** (Potential): For general utility functions like `escape_csv_field`.
