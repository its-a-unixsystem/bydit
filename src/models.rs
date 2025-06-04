#[derive(Debug, Clone)] // Added Debug and Clone for general utility
pub struct UnifiedItem {
    pub id: String, // Full Reddit ID, e.g., t3_xxxxxx or t1_xxxxxx
    pub item_type: String, // "Post" or "Comment"
    pub subreddit: String,
    pub title: String,       // Post title or link title for comment
    pub content: String,     // Post selftext or comment body
    pub upvotes: i32,
    pub num_comments: i32, // For posts; 0 for comments
    pub permalink: String,
    pub created_utc: f64,  // Timestamp for sorting
}
