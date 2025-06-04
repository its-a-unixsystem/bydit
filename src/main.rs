use roux::Reddit;
use serde::Deserialize;
use std::fs;

#[derive(Deserialize)]
struct Config {
    user_agent: String,
    client_id: String,
    client_secret: String,
}

#[tokio::main]
async fn main() {
    // Read configuration from config.toml
    let config_str = fs::read_to_string("config.toml")
        .expect("Failed to read config.toml. Make sure it exists in the project root.");
    let config: Config = toml::from_str(&config_str)
        .expect("Failed to parse config.toml. Check its format.");

    println!("Logging in to Reddit...");
    
    // Create a new Reddit client
    let reddit = match Reddit::new(
        &config.user_agent,
        &config.client_id,
        &config.client_secret,
    )
    .username("USERNAME")
    .password("PASSWORD")
    .login()
    .await {
        Ok(client) => {
            println!("Successfully logged in to Reddit.");
            client
        },
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

    println!("\nFetching your submitted posts...");
    let user = roux::user::User::new("USERNAME");
    match user.submitted(None).await {
        Ok(submitted_feed) => {
            if submitted_feed.data.children.is_empty() {
                println!("You have not submitted any posts.");
            } else {
                println!("\nSuccessfully fetched {} of your submitted posts:", submitted_feed.data.children.len());
                println!("----------------------------------------");
                for (i, post) in submitted_feed.data.children.iter().enumerate() {
                    // Get post details with proper fallbacks
                    let title = &post.data.title;
                    let subreddit = &post.data.subreddit;
                    
                    println!("{}. {}", i + 1, title);
                    println!("   Subreddit: r/{}", subreddit);
                    println!("   Upvotes: {}, Comments: {}\n", post.data.ups, post.data.num_comments);
                }
            }
        },
        Err(e) => {
            eprintln!("\nFailed to fetch your submitted posts: {}", e);
        }
    }

    println!("\nFetching your comments...");
    match user.comments(None).await {
        Ok(comments_feed) => {
            if comments_feed.data.children.is_empty() {
                println!("You have not made any comments.");
            } else {
                println!("\nSuccessfully fetched {} of your comments:", comments_feed.data.children.len());
                println!("----------------------------------------");
                for (i, comment) in comments_feed.data.children.iter().enumerate() {
                    // Get comment details with proper handling for Option types
                    let link_title = comment.data.link_title.as_ref().map_or("[N/A]", |s| s.as_str());
                    let subreddit = comment.data.subreddit.as_ref().map_or("[N/A]", |s| s.as_str());
                    let body = comment.data.body.as_ref().map_or("[No Body]", |s| s.as_str());
                    
                    println!("{}. Comment on post: '{}' in r/{}", i + 1, link_title, subreddit);
                    println!("   \"{}\"", body);
                    println!("   Upvotes: {}\n", comment.data.ups.unwrap_or(0));
                }
            }
        },
        Err(e) => {
            eprintln!("\nFailed to fetch your comments: {}", e);
        }
    }

    println!("\nFinished fetching your data.");
}
