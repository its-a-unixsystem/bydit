use clap::Parser;
use roux::Reddit;
use serde::Deserialize;
use std::fs;

#[derive(Deserialize)]
struct Config {
    user_agent: String,
    client_id: String,
    client_secret: String,
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
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();

    // Read configuration from config.toml
    let config_str = fs::read_to_string("config.toml")
        .expect("Failed to read config.toml. Make sure it exists in the project root.");
    let config: Config =
        toml::from_str(&config_str).expect("Failed to parse config.toml. Check its format.");

    println!("Logging in to Reddit...");

    // Create a new Reddit client
    let reddit = match Reddit::new(
        &config.user_agent,
        &config.client_id,
        &config.client_secret,
    )
    .username("USERNAME") // Consider making username/password configurable or CLI args
    .password("PASSWORD") // Store credentials securely
    .login()
    .await
    {
        Ok(client) => {
            println!("Successfully logged in to Reddit.");
            client
        }
        Err(e) => {
            eprintln!("Failed to log in to Reddit: {}", e);
            return;
        }
    };

    // Get the authenticated user's information
    if let Err(e) = reddit.me().await {
        eprintln!("\nFailed to fetch your user data: {}", e);
        return;
    }
    println!("Successfully retrieved account information!");

    if cli.debug {
        println!("\nFetching your submitted posts...");
    }
    let user = roux::user::User::new("USERNAME"); // Consider making username configurable

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

            if filtered_posts.is_empty() {
                println!("No submitted posts match your criteria.");
            } else {
                println!(
                    "\nSuccessfully fetched {} of your submitted posts matching criteria:",
                    filtered_posts.len()
                );
                println!("----------------------------------------");
                for (i, post) in filtered_posts.iter().enumerate() {
                    let title = &post.data.title;
                    let subreddit_name = &post.data.subreddit;

                    println!("{}. {}", i + 1, title);
                    println!("   Subreddit: r/{}", subreddit_name);
                    println!(
                        "   Upvotes: {}, Comments: {}\n",
                        post.data.ups, post.data.num_comments
                    );
                }
            }
        }
        Err(e) => {
            eprintln!("\nFailed to fetch your submitted posts: {}", e);
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
                    eprintln!("\nError fetching page {} of comments: {}", page_count, e);
                    break; 
                }
            }
        }

        if cli.debug {
            println!("Finished fetching all comment pages. Total raw comments fetched: {}", all_fetched_comments.len());
        }

        if all_fetched_comments.is_empty() { 
             println!("No comments found for this user after attempting to fetch all pages.");
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

            if filtered_comments.is_empty() {
                println!("No comments match your criteria after filtering.");
            } else {
                println!(
                    "\nSuccessfully fetched and filtered {} comments matching criteria:",
                    filtered_comments.len()
                );
                println!("----------------------------------------");
                for (i, comment) in filtered_comments.iter().enumerate() {
                    let link_title = comment.data.link_title.as_ref().map_or("[N/A]", |s| s.as_str());
                    let subreddit_name = comment.data.subreddit.as_ref().map_or("[N/A]", |s| s.as_str());
                    let body = comment.data.body.as_ref().map_or("[No Body]", |s| s.as_str());

                    println!(
                        "{}. Comment on post: '{}' in r/{}",
                        i + 1,
                        link_title,
                        subreddit_name
                    );
                    println!("   \"{}\"", body);
                    println!("   Upvotes: {}\n", comment.data.ups.unwrap_or(0));
                }
            }
        }
    } // Closes 'if fetch_comments'

    println!("\nFinished fetching your data.");
}
