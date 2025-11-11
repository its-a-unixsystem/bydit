# RedditAccess - Reddit Data Exporter

RedditAccess is a command-line application that fetches your Reddit posts and/or comments, allows filtering by subreddit and score range, and exports the data in CSV format.

## Setup

1.  **Clone the repository (if applicable) or download the source code.**
2.  **Create a Reddit Application:**
    *   Go to [Reddit's app preferences](https://www.reddit.com/prefs/apps).
    *   Click "are you a developer? create an app..."
    *   Fill in the details:
        *   **name:** (e.g., `RedditAccessApp`)
        *   **type:** select `script`
        *   **description:** (e.g., `App to export my Reddit data`)
        *   **about url:** (can be blank or your profile URL)
        *   **redirect uri:** (e.g., `http://localhost:8080` - this won't be actively used by RedditAccess but is required by Reddit)
    *   Click "create app".
    *   Note down the **client ID** (shown under your app's name) and the **client secret**.
3.  **Configure RedditAccess:**
    *   In the root directory of the RedditAccess project, create a file named `config.toml`.
    *   Add your Reddit API credentials and login information to `config.toml`:

        ```toml
        user_agent = "YOUR_CUSTOM_USER_AGENT_STRING"  # e.g., RedditAccessApp/0.1 by YourUsername
        client_id = "YOUR_REDDIT_APP_CLIENT_ID"      # Found in your Reddit app settings
        client_secret = "YOUR_REDDIT_APP_CLIENT_SECRET"# Found in your Reddit app settings
        username = "YOUR_REDDIT_USERNAME"
        password = "YOUR_REDDIT_PASSWORD"
        ```

        **Important:** Replace the placeholder values with your actual credentials. The `user_agent` should be a unique string that describes your script, including your username if possible (e.g., `RedditAccess/1.0 by u/YourUsername`).

## Building and Running

1.  Navigate to the project's root directory in your terminal.
2.  Build the application:
    ```bash
    cargo build
    ```
3.  Run the application:
    ```bash
    cargo run -- [OPTIONS]
    ```
    Or, after building, run the executable directly:
    ```bash
    ./target/debug/reddit-access [OPTIONS]
    ```
    For a release build (optimized):
    ```bash
    cargo build --release
    ./target/release/reddit-access [OPTIONS]
    ```

## Command-Line Options

RedditAccess accepts the following command-line options:

*   `-s, --subreddit <SUBREDDIT>`: Optional. Filter results by specific subreddit name(s). Accepts comma-separated list (e.g., `rust,programming,coding`). If not provided, items from all subreddits will be fetched.
*   `-x, --exclude-subreddit <SUBREDDIT>`: Optional. Exclude results from specific subreddit name(s). Accepts comma-separated list (e.g., `spam,test,offtopic`). Can be combined with `--subreddit`.
*   `-m, --min-score <MIN_SCORE>`: Optional. Filter results to include only items with at least this many upvotes. If not provided, all items are included regardless of score.
*   `-M, --max-score <MAX_SCORE>`: Optional. Filter results to exclude items with scores at or above this threshold (i.e., keep only items with scores below this value). Can be negative. If not provided, no upper score limit is applied.
*   `--min-age <AGE>`: Optional. Filter results to include only items older than the specified age. Accepts human-readable durations (e.g., `1 week`, `2 years`, `30 days`) or specific dates (e.g., `2024-01-15`, `2024-01-15T10:30:00`). If not provided, no minimum age limit is applied.
*   `--max-age <AGE>`: Optional. Filter results to include only items newer than the specified age. Accepts human-readable durations (e.g., `1 week`, `2 years`, `30 days`) or specific dates (e.g., `2024-01-15`, `2024-01-15T10:30:00`). If not provided, no maximum age limit is applied.
*   `-t, --item-type <ITEM_TYPE>`: Optional. Specify the type of items to fetch. Valid values are:
    *   `posts`: Fetch only submitted posts.
    *   `comments`: Fetch only comments.
    *   `both`: Fetch both posts and comments.
    If not provided, defaults to fetching `both`.
*   `--debug`: Optional. Enable verbose debug logging to the console.
*   `-h, --help`: Display help information.
*   `-V, --version`: Display version information.
*   `--overwrite <TEXT>`: Optional. If provided, the content of filtered posts or comments will be replaced with the specified text. This happens *before* deletion if `--delete` is also used.
*   `-y, --yes`: Optional. If provided with `--delete`, skips the confirmation prompt before deleting items.
*   `--delete`: Optional. Delete the fetched items from Reddit after processing.

## Output Format

The application outputs data in CSV format to standard output. The CSV header is:

`Type,Subreddit,Title,Content,Upvotes,NumComments,Permalink,TimestampUTC`

*   **Type**: "Post" or "Comment".
*   **Subreddit**: The subreddit the item belongs to (e.g., `r/learnrust`).
*   **Title**: The title of the post. For comments, this is the title of the post the comment belongs to.
*   **Content**: The self-text of the post or the body of the comment. Newlines within the content are escaped as `\n`.
*   **Upvotes**: The number of upvotes for the item.
*   **NumComments**: The number of comments on a post. For comments, this field will be 0.
*   **Permalink**: A relative URL to the item on Reddit (e.g., `/r/rust/comments/xxxxxx/title/yyyyyy/`).
*   **TimestampUTC**: The UTC timestamp of when the item was created.

## Example Usage

*   Fetch all posts and comments from the `rust` subreddit with a minimum score of 10:
    ```bash
    ./target/debug/reddit-access -s rust -m 10
    ```
*   Fetch posts and comments from multiple subreddits:
    ```bash
    ./target/debug/reddit-access -s rust,programming,coding
    ```
*   Exclude posts and comments from specific subreddits:
    ```bash
    ./target/debug/reddit-access -x spam,test,offtopic
    ```
*   Combine include and exclude filters (fetch from rust and programming, but exclude rust_gaming):
    ```bash
    ./target/debug/reddit-access -s rust,programming -x rust_gaming
    ```
*   Fetch posts and comments with scores between 5 and 100 (inclusive of 5, exclusive of 100):
    ```bash
    ./target/debug/reddit-access -m 5 -M 100
    ```
*   Fetch only your comments from all subreddits:
    ```bash
    ./target/debug/reddit-access --item-type comments
    ```
*   Fetch all your posts and comments and save to a file:
    ```bash
    ./target/debug/reddit-access > my_reddit_data.csv
    ```
*   Fetch posts from `r/test` and overwrite their content, then delete them:
    ```bash
    ./target/debug/reddit-access -s test --item-type posts --overwrite "This content has been updated." --delete
*   Fetch all your comments and delete them without prompting for confirmation:
    ```bash
    ./target/debug/reddit-access --item-type comments --delete -y
    ```
*   Fetch posts older than 1 year:
    ```bash
    ./target/debug/reddit-access --item-type posts --min-age "1 year"
    ```
*   Fetch comments from the last 30 days:
    ```bash
    ./target/debug/reddit-access --item-type comments --max-age "30 days"
    ```
*   Fetch posts between 6 months and 1 year old:
    ```bash
    ./target/debug/reddit-access --item-type posts --min-age "1 year" --max-age "6 months"
    ```
*   Fetch posts created after a specific date:
    ```bash
    ./target/debug/reddit-access --item-type posts --max-age "2024-01-15"
    ```
*   Fetch posts created before a specific date:
    ```bash
    ./target/debug/reddit-access --item-type posts --min-age "2024-06-01"
    ```

## License

MIT
