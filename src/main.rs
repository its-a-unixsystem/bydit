use clap::Parser;
use roux::Reddit;
use serde::Deserialize;
use std::fs;
use std::io::{self, Write};

#[derive(Deserialize)]
struct Config {
    user_agent: String,
    client_id: String,
    client_secret: String,
    username: String,
    password: String,
}

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Cli {
    #[clap(short, long, value_parser)]
    subreddit: Option<String>,

    #[clap(short = 'm', long, value_parser, name = "min_score")]
    score: Option<i32>,

    #[clap(short, long, value_parser, name = "type")]
    item_type: Option<String>,

    #[clap(long)]
    debug: bool,

    #[clap(short, long, action)]
    delete: bool,
}

#[derive(Debug)]
struct UnifiedItem {
    id: String, // Full Reddit ID, e.g., t3_xxxxxx or t1_xxxxxx
    item_type: String, // "Post" or "Comment"
    subreddit: String,
    title: String,       // Post title or link title for comment
    content: String,     // Post selftext or comment body
    upvotes: i32,
    num_comments: i32, // For posts; 0 for comments
    permalink: String,
    created_utc: f64,  // Timestamp for sorting
}

fn escape_csv_field(field: &str) -> String {
    field
        .replace("\r\n", "\\n")
        .replace("\n", "\\n")
        .replace("\r", "\\n")
        .replace("\"", "\"\"")
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();

    // Read configuration from config.toml
    let config_str = fs::read_to_string("config.toml")
        .expect("Failed to read config.toml. Make sure it exists in the project root.");
    let config: Config =
        toml::from_str(&config_str).expect("Failed to parse config.toml. Check its format.");

    if cli.debug { println!("Logging in to Reddit..."); }

    // Create a new Reddit client
    let reddit = match Reddit::new(
        &config.user_agent,
        &config.client_id,
        &config.client_secret,
    )
    .username(&config.username)
    .password(&config.password)
    .login()
    .await
    {
        Ok(client) => {
            if cli.debug { println!("Successfully logged in to Reddit."); }
            client
        }
        Err(e) => {
            if cli.debug { eprintln!("Failed to log in to Reddit: {}", e); }
            return;
        }
    };

    // Get the authenticated user's information
    if let Err(e) = reddit.me().await {
        if cli.debug { eprintln!("\nFailed to fetch your user data: {}", e); }
        return;
    }
    if cli.debug { println!("Successfully retrieved account information!"); }

    let mut all_items: Vec<UnifiedItem> = Vec::new();

    if cli.debug {
        println!("\nFetching your submitted posts...");
    }
    let user = roux::user::User::new(&config.username);

    let fetch_posts = cli.item_type.as_ref().map_or(true, |t| t.eq_ignore_ascii_case("posts") || t.eq_ignore_ascii_case("both"));
    let fetch_comments = cli.item_type.as_ref().map_or(true, |t| t.eq_ignore_ascii_case("comments") || t.eq_ignore_ascii_case("both"));

    if fetch_posts {
        match user.submitted(None).await {
        Ok(submitted_feed) => {
            let filtered_posts: Vec<_> = submitted_feed
                .data
                .children
                .into_iter()
                .filter(|post| {
                    let subreddit_match = cli
                        .subreddit
                        .as_ref()
                        .map_or(true, |sr| post.data.subreddit.eq_ignore_ascii_case(sr));
                    let score_match = cli.score.map_or(true, |s| post.data.ups >= s as f64);
                    subreddit_match && score_match
                })
                .collect();

            let num_filtered_posts = filtered_posts.len();
            for post_data in filtered_posts.into_iter() {
                let item = UnifiedItem {
                    id: post_data.data.name.clone(),
                    item_type: "Post".to_string(),
                    subreddit: post_data.data.subreddit,
                    title: post_data.data.title,
                    content: post_data.data.selftext,
                    upvotes: post_data.data.ups as i32,
                    num_comments: post_data.data.num_comments as i32,
                    permalink: post_data.data.permalink,
                    created_utc: post_data.data.created_utc,
                };
                all_items.push(item);
            }
             if cli.debug {
                println!("Collected {} posts.", num_filtered_posts);
            }
        }
        Err(e) => {
            if cli.debug {
                eprintln!("\nFailed to fetch your submitted posts: {}", e);
            }
        }
    }
    } // Closes 'if fetch_posts'

    if fetch_comments {
        if cli.debug {
            println!("\nFetching your comments...");
        }
        let mut all_fetched_comments = Vec::new();
        let mut after_token: Option<String> = None;
        let mut page_count = 0; 

        loop {
            page_count += 1;
            if cli.debug {
                println!("Fetching page {} of comments...", page_count);
            }

            let mut feed_options = roux::util::FeedOption::new().limit(100); // Reddit API typically allows up to 100
            if let Some(token) = &after_token {
                feed_options = feed_options.after(token);
            }

            match user.comments(Some(feed_options)).await {
                Ok(comments_feed) => {
                    let num_fetched_this_page = comments_feed.data.children.len();
                    if cli.debug {
                        println!("Fetched {} comments on page {}.", num_fetched_this_page, page_count);
                    }

                    all_fetched_comments.extend(comments_feed.data.children);
                    after_token = comments_feed.data.after;

                    if after_token.is_none() {
                        if cli.debug {
                            println!("No more comments to fetch (after_token is None).");
                        }
                        break;
                    }
                    // Additional check for robustness as per TODO.md:
                    // If children is empty and it's not the first page, assume we're done.
                    if num_fetched_this_page == 0 && page_count > 1 { 
                        if cli.debug {
                            println!("Fetched 0 comments on page {} (not first page), assuming end of comments.", page_count);
                        }
                        break;
                    }
                }
                Err(e) => {
                    if cli.debug {
                        eprintln!("\nError fetching page {} of comments: {}", page_count, e);
                    }
                    break; 
                }
            }
        }

        if cli.debug {
            println!("Finished fetching all comment pages. Total raw comments fetched: {}", all_fetched_comments.len());
        }

        if all_fetched_comments.is_empty() { 
            if cli.debug {
                 println!("No comments found for this user after attempting to fetch all pages.");
            }
        } else {
            let filtered_comments: Vec<_> = all_fetched_comments
                .into_iter() 
                .filter(|comment| {
                    let subreddit_match = cli.subreddit.as_ref().map_or(true, |sr| {
                        comment
                            .data
                            .subreddit
                            .as_ref()
                            .map_or(false, |cs| cs.eq_ignore_ascii_case(sr))
                    });
                    let score_match =
                        cli.score.map_or(true, |s| comment.data.ups.unwrap_or(0) >= s);
                    subreddit_match && score_match
                })
                .collect();

            let num_filtered_comments = filtered_comments.len();
            for comment_data in filtered_comments.into_iter() {
                let item = UnifiedItem {
                    id: comment_data.data.name.clone().unwrap_or_default(),
                    item_type: "Comment".to_string(),
                    subreddit: comment_data.data.subreddit.unwrap_or_default(),
                    title: comment_data.data.link_title.unwrap_or_default(),
                    content: comment_data.data.body.unwrap_or_default(),
                    upvotes: comment_data.data.ups.unwrap_or(0),
                    num_comments: 0, // Comments don't have a direct num_comments field like posts
                    permalink: comment_data.data.permalink.unwrap_or_default(),
                    created_utc: comment_data.data.created_utc.unwrap_or(0.0),
                };
                all_items.push(item);
            }
            if cli.debug {
                println!("Collected {} comments.", num_filtered_comments);
            }
        }
    } // Closes 'if fetch_comments'

    // Sort all items by creation date (newest first)
    all_items.sort_by(|a, b| b.created_utc.partial_cmp(&a.created_utc).unwrap_or(std::cmp::Ordering::Equal));

    if cli.delete {
        if all_items.is_empty() {
            if cli.debug {
                println!("No items found to delete based on current filters.");
            }
        } else {
            println!("\nStarting deletion process...");
            let mut deleted_count = 0;
            let mut skipped_count = 0;
            let total_items_to_process = all_items.len();

            for (index, item) in all_items.iter().enumerate() {
                println!(
                    "\nItem {}/{} to delete:",
                    index + 1,
                    total_items_to_process
                );
                println!("  Type: {}", item.item_type);
                println!("  Subreddit: r/{}", item.subreddit);
                let display_title = if item.item_type == "Comment" {
                    format!("Comment in post \"{}\"", item.title)
                } else {
                    item.title.clone()
                };
                println!("  Title: {}", display_title);
                println!("  Upvotes: {}", item.upvotes);

                loop {
                    print!("Are you sure you want to delete this item? (y/n/a - yes/no/abort): ");
                    io::stdout().flush().unwrap();
                    let mut input = String::new();
                    io::stdin().read_line(&mut input).expect("Failed to read line");
                    match input.trim().to_lowercase().as_str() {
                        "y" => {
                            if cli.debug {
                                println!("Attempting to delete item with ID: {}", item.id);
                            }
                            let delete_url = "https://oauth.reddit.com/api/del";
                            let params = [("id", item.id.as_str())];
                            match reddit.client.post(delete_url).form(&params).send().await {
                                Ok(response) => {
                                    if response.status().is_success() {
                                        // Consume response body to be a good netizen and allow connection reuse.
                                        let _ = response.text().await; 
                                        println!("Successfully deleted item: {}", item.id);
                                        deleted_count += 1;
                                    } else {
                                        let status = response.status();
                                        let error_body = response.text().await.unwrap_or_else(|e| format!("Could not read error response body: {}", e));
                                        eprintln!(
                                            "Failed to delete item {} - API Error Status: {}. Details: {}",
                                            item.id,
                                            status,
                                            error_body
                                        );
                                    }
                                }
                                Err(e) => { // This is a reqwest::Error (network, DNS, timeout, builder error, etc.)
                                    eprintln!("Error sending delete request for item {}: {}", item.id, e);
                                    if cli.debug {
                                        eprintln!("Debug details for reqwest error: {:?}", e);
                                    }
                                }
                            }
                            break;
                        }
                        "n" => {
                            println!("Skipped item: {}", item.id);
                            skipped_count += 1;
                            break;
                        }
                        "a" => {
                            println!("Aborting deletion process.");
                            println!("\nDeletion Statistics (aborted):");
                            println!("  Total items processed before abort: {}", index);
                            println!("  Items deleted: {}", deleted_count);
                            println!("  Items skipped: {}", skipped_count);
                            if cli.debug { println!("\nApplication finished."); }
                            return;
                        }
                        _ => {
                            println!("Invalid input. Please enter 'y', 'n', or 'a'.");
                        }
                    }
                }
            }

            println!("\nDeletion Statistics:");
            println!("  Total items considered: {}", total_items_to_process);
            println!("  Items deleted: {}", deleted_count);
            println!("  Items skipped: {}", skipped_count);
        }
    } else {
        // Print unified CSV header
        if !all_items.is_empty() {
        println!("Type,Subreddit,Title,Content,Upvotes,NumComments,Permalink,TimestampUTC");
        for item in all_items.iter() {
            let escaped_title = escape_csv_field(&item.title);
            let escaped_content = escape_csv_field(&item.content);
            let subreddit_prefix = if item.subreddit.is_empty() { "" } else { "r/" };

            println!(
                "\"{}\",\"{}{}\",\"{}\",\"{}\",{},{},\"https://reddit.com{}\",{}",
                item.item_type,
                subreddit_prefix,
                item.subreddit,
                escaped_title,
                escaped_content,
                item.upvotes,
                item.num_comments,
                item.permalink,
                item.created_utc
            );
        }
    } else {
        if cli.debug {
            println!("No items to output after filtering.");
        }
    }
        if cli.debug { println!("\nFinished processing and printing data."); }
    }

    if cli.debug { println!("\nApplication finished."); }
}
