use clap::Parser; // Cli::parse() is used in main
use std::error::Error;
// std::io::{self, Write}; // Moved to actions.rs
use tokio;

mod config;
use config::load_config;

mod reddit_ops;
use reddit_ops::{connect_reddit, fetch_user_items};

mod actions;
use actions::{handle_overwrite_action, handle_delete_action, handle_csv_export, handle_print_to_console};

mod cli;
use cli::Cli;

mod models;
mod utils;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let cli = Cli::parse();

    let config = load_config("config.toml", cli.debug)?;

    let reddit = connect_reddit(&config, cli.debug).await?;

    // Get the authenticated user's information
    let authenticated_username: &str = reddit.config.username.as_deref().ok_or_else(|| {
        let err_msg = "Username is None in config after successful login and Me retrieval.";
        if cli.debug { eprintln!("{}", err_msg); }
        Box::<dyn Error>::from(err_msg)
    })?;
    if cli.debug { println!("Successfully logged in and using username: {}", authenticated_username); }

    // Fetch additional account metadata using the me() endpoint
    let me_data = reddit.me().await.map_err(|e| {
        if cli.debug {
            eprintln!("\nFailed to fetch your user data (me()): {}", e);
        }
        Box::new(e) as Box<dyn Error>
    })?;
    if cli.debug { println!("Successfully retrieved account metadata (Reddit ID: {}) for user: {}", me_data.id, authenticated_username); }

    // The `reddit` object (type `roux::Reddit` after successful login) can be used for actions
    // like edit, comment, etc., as it holds the authenticated state.
    // The `me_data` variable (type `roux::models::me::MeData`) contains user-specific information.

    // Determine what to fetch
    let fetch_posts = cli.item_type.as_ref().map_or(true, |t| t.eq_ignore_ascii_case("post") || t.eq_ignore_ascii_case("posts") || t.eq_ignore_ascii_case("both"));
    let fetch_comments = cli.item_type.as_ref().map_or(true, |t| t.eq_ignore_ascii_case("comment") || t.eq_ignore_ascii_case("comments") || t.eq_ignore_ascii_case("both"));

    if cli.debug {
        let mut fetching_what = Vec::new();
        if fetch_posts { fetching_what.push("posts"); }
        if fetch_comments { fetching_what.push("comments"); }

        if fetching_what.is_empty() {
            // This case should ideally not be reached if item_type defaults or is "both"
            println!("\nNot fetching any specific item types based on current filters.");
        } else {
            println!("\nPreparing to fetch your {}...", fetching_what.join(" and "));
        }
    }

    // Create user object for fetching
    let user = roux::user::User::new(authenticated_username);

    // Fetch items
    let mut all_items = fetch_user_items(
        &user,
        fetch_posts,
        fetch_comments,
        cli.subreddit.as_ref(),
        cli.score,
        cli.debug,
    )
    .await?;

    // Sort all items by creation date (newest first)
    all_items.sort_by(|a, b| b.created_utc.partial_cmp(&a.created_utc).unwrap_or(std::cmp::Ordering::Equal));

    // If overwrite is requested, perform it first. If delete is also requested, proceed to deletion next.
    if let Some(overwrite_text) = &cli.overwrite {
        handle_overwrite_action(&reddit, &mut all_items, overwrite_text, cli.debug).await?;
    }

    if cli.delete {
        let _deleted_items_count = handle_delete_action(&reddit, &all_items, cli.yes, cli.debug).await?;
        // The function now prints its own summary.
    } else if let Some(csv_file_path) = &cli.csv {
        if all_items.is_empty() {
            if cli.debug {
                println!("No items to export to CSV based on current filters.");
            }
        } else {
            handle_csv_export(&all_items, csv_file_path, cli.debug)?;
        }
    } else {
        handle_print_to_console(&all_items, cli.debug);
        if cli.debug { println!("\nFinished processing and printing data."); }
    }

    if cli.debug { println!("\nApplication finished."); }
    Ok(())
}
