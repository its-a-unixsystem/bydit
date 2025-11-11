use roux::Reddit;
use roux::Me;
use roux::util::FeedOption;
use roux::{Submissions, Comments};
use std::error::Error;
use crate::config::Config; 
use crate::models::UnifiedItem;

pub async fn connect_reddit(config: &Config, debug_mode: bool) -> Result<Me, Box<dyn Error>> {
    if debug_mode { println!("Connecting to Reddit and logging in..."); }
    let reddit_client = Reddit::new(
        &config.user_agent,
        &config.client_id,
        &config.client_secret,
    )
    .username(&config.username)
    .password(&config.password)
    .login()
    .await
    .map_err(|e| {
        if debug_mode { eprintln!("Failed to log in to Reddit: {}", e); }
        Box::new(e) as Box<dyn Error>
    })?;
    if debug_mode { println!("Successfully logged in to Reddit."); }
    Ok(reddit_client)
}

pub async fn fetch_user_items(
    reddit: &Me,
    username: &str,
    do_fetch_posts: bool,
    do_fetch_comments: bool,
    filter_subreddit: Option<&String>,
    exclude_subreddit: Option<&String>,
    filter_score: Option<i32>,
    filter_max_score: Option<i32>,
    min_age_timestamp: Option<f64>,
    max_age_timestamp: Option<f64>,
    filter_post_title: Option<&String>,
    debug_mode: bool,
) -> Result<Vec<UnifiedItem>, Box<dyn Error>> {
    let mut all_items: Vec<UnifiedItem> = Vec::new();

    // Parse comma-separated subreddit lists
    let include_subreddits: Option<Vec<String>> = filter_subreddit.map(|s| {
        s.split(',')
            .map(|sr| sr.trim().to_lowercase())
            .filter(|sr| !sr.is_empty())
            .collect()
    });

    let exclude_subreddits: Option<Vec<String>> = exclude_subreddit.map(|s| {
        s.split(',')
            .map(|sr| sr.trim().to_lowercase())
            .filter(|sr| !sr.is_empty())
            .collect()
    });

    if do_fetch_posts {
        if debug_mode {
            println!("Fetching your posts...");
        }
        // Use authenticated OAuth client to fetch user's submitted posts
        let mut url = format!("user/{}/submitted/.json?", username);
        let feed_options = FeedOption::new().limit(100);
        feed_options.build_url(&mut url);
        let oauth_url = roux::util::url::build_oauth(&url);

        if debug_mode { println!("GET {}", oauth_url); }
        let submitted_feed_res = reddit.client.get(&oauth_url).send().await;
        match submitted_feed_res {
            Ok(response) => {
                if debug_mode { println!("Status: {}", response.status()); }
                let submitted_feed: Submissions = response.json().await.map_err(|e| {
                    if debug_mode {
                        eprintln!("Failed to parse submitted posts JSON: {}", e);
                    }
                    Box::new(e) as Box<dyn Error>
                })?;

                let filtered_posts: Vec<_> = submitted_feed
                    .data
                    .children
                    .into_iter()
                    .filter(|post| {
                        let post_subreddit = post.data.subreddit.to_lowercase();
                        
                        // Check if subreddit is in include list (if specified)
                        let include_match = include_subreddits.as_ref().map_or(true, |list| {
                            list.iter().any(|sr| sr == &post_subreddit)
                        });
                        
                        // Check if subreddit is NOT in exclude list (if specified)
                        let exclude_match = exclude_subreddits.as_ref().map_or(true, |list| {
                            !list.iter().any(|sr| sr == &post_subreddit)
                        });
                        
                        let score_match = filter_score.map_or(true, |s| post.data.ups >= s as f64);
                        let max_score_match = filter_max_score.map_or(true, |s| post.data.ups < s as f64);
                        
                        // Age filtering: created_utc is the timestamp when the post was created
                        // min_age_timestamp: items must be OLDER than this (created_utc <= min_age_timestamp)
                        // max_age_timestamp: items must be NEWER than this (created_utc >= max_age_timestamp)
                        let min_age_match = min_age_timestamp.map_or(true, |min_ts| post.data.created_utc <= min_ts);
                        let max_age_match = max_age_timestamp.map_or(true, |max_ts| post.data.created_utc >= max_ts);
                        
                        include_match && exclude_match && score_match && max_score_match && min_age_match && max_age_match
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
                if debug_mode {
                    println!("Collected {} posts.", num_filtered_posts);
                }
            }
            Err(e) => {
                if debug_mode {
                    eprintln!("\nFailed to fetch your submitted posts: {}", e);
                }
                return Err(Box::new(e) as Box<dyn Error>);
            }
        }
    }

    if do_fetch_comments {
        if debug_mode {
            println!("\nFetching your comments...");
        }
        let mut all_fetched_comments = Vec::new();
        let mut after_token: Option<String> = None;
        let mut page_count = 0;

        loop {
            page_count += 1;
            if debug_mode {
                println!("Fetching page {} of comments...", page_count);
            }

            // Build OAuth URL with pagination options
            let mut url = format!("user/{}/comments/.json?", username);
            let mut feed_options = FeedOption::new().limit(100);
            if let Some(token) = &after_token {
                feed_options = feed_options.after(token);
            }
            feed_options.build_url(&mut url);
            let oauth_url = roux::util::url::build_oauth(&url);

            if debug_mode { println!("GET {}", oauth_url); }
            match reddit.client.get(&oauth_url).send().await {
                Ok(response) => {
                    if debug_mode { println!("Status: {}", response.status()); }
                    let comments_feed: Comments = response.json().await.map_err(|e| {
                        if debug_mode {
                            eprintln!("Failed to parse comments JSON on page {}: {}", page_count, e);
                        }
                        Box::new(e) as Box<dyn Error>
                    })?;

                    let num_fetched_this_page = comments_feed.data.children.len();
                    if debug_mode {
                        println!("Fetched {} comments on page {}.", num_fetched_this_page, page_count);
                    }

                    all_fetched_comments.extend(comments_feed.data.children);
                    after_token = comments_feed.data.after;

                    if after_token.is_none() {
                        if debug_mode {
                            println!("No more comments to fetch (after_token is None).");
                        }
                        break;
                    }
                    if num_fetched_this_page == 0 && page_count > 1 {
                        if debug_mode {
                            println!("Fetched 0 comments on page {} (not first page), assuming end of comments.", page_count);
                        }
                        break;
                    }
                }
                Err(e) => {
                    if debug_mode {
                        eprintln!("\nError fetching page {} of comments: {}", page_count, e);
                    }
                    return Err(Box::new(e) as Box<dyn Error>);
                }
            }
        }

        if debug_mode {
            println!("Finished fetching all comment pages. Total raw comments fetched: {}", all_fetched_comments.len());
        }

        if all_fetched_comments.is_empty() {
            if debug_mode {
                println!("No comments found for this user after attempting to fetch all pages.");
            }
        } else {
            let filtered_comments: Vec<_> = all_fetched_comments
                .into_iter()
                .filter(|comment| {
                    let comment_subreddit = comment.data.subreddit.as_ref().map(|s| s.to_lowercase());
                    
                    // Check if subreddit is in include list (if specified)
                    let include_match = include_subreddits.as_ref().map_or(true, |list| {
                        comment_subreddit.as_ref().map_or(false, |cs| {
                            list.iter().any(|sr| sr == cs)
                        })
                    });
                    
                    // Check if subreddit is NOT in exclude list (if specified)
                    let exclude_match = exclude_subreddits.as_ref().map_or(true, |list| {
                        comment_subreddit.as_ref().map_or(false, |cs| {
                            !list.iter().any(|sr| sr == cs)
                        })
                    });
                    
                    let score_match = filter_score.map_or(true, |s_filter| {
                        comment.data.score.map_or(true, |s_comment| s_comment >= s_filter)
                    });
                    let max_score_match = filter_max_score.map_or(true, |s_filter| {
                        comment.data.score.map_or(true, |s_comment| s_comment < s_filter)
                    });
                    
                    // Age filtering for comments (min_age => older than threshold, max_age => newer than threshold)
                    let older_than_match = min_age_timestamp.map_or(true, |min_ts| {
                        comment.data.created_utc.map_or(true, |created| created <= min_ts)
                    });
                    let newer_than_match = max_age_timestamp.map_or(true, |max_ts| {
                        comment.data.created_utc.map_or(true, |created| created >= max_ts)
                    });
                    
                    // Post title filtering for comments (case-insensitive substring match)
                    let post_title_match = filter_post_title.as_ref().map_or(true, |filter_title| {
                        comment.data.link_title.as_ref().map_or(false, |link_title| {
                            link_title.to_lowercase().contains(&filter_title.to_lowercase())
                        })
                    });
                    
                    include_match && exclude_match && score_match && max_score_match && older_than_match && newer_than_match && post_title_match
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
                    upvotes: comment_data.data.score.unwrap_or(0),
                    num_comments: 0, // Comments don't have a direct num_comments field in this context
                    permalink: comment_data.data.permalink.unwrap_or_default(),
                    created_utc: comment_data.data.created_utc.unwrap_or(0.0),
                };
                all_items.push(item);
            }
            if debug_mode {
                println!("Collected {} comments after filtering.", num_filtered_comments);
            }
        }
    }

    Ok(all_items)
}
