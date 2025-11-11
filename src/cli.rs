use clap::Parser;

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None, allow_negative_numbers = true)]
pub struct Cli {
    #[clap(short, long, value_parser, help = "Filter by subreddit(s). Comma-separated list: --subreddit one,two,three")]
    pub subreddit: Option<String>,

    #[clap(short = 'x', long, value_parser, help = "Exclude subreddit(s). Comma-separated list: --exclude-subreddit one,two,three")]
    pub exclude_subreddit: Option<String>,

    #[clap(short = 'm', long = "min-score", value_parser, name = "min_score")]
    pub score: Option<i32>,

    #[clap(short = 'M', long, value_parser, name = "max_score")]
    pub max_score: Option<i32>,

    #[clap(short, long, value_parser)] // Long flag will be --item-type
    pub item_type: Option<String>,

    #[clap(long, help = "Enable debug mode for verbose output")]
    pub debug: bool,

    #[clap(short, long, action)]
    pub delete: bool,

    #[clap(short, long, help = "Skip confirmation prompts when deleting items")]
    pub yes: bool,

    #[clap(long, value_parser, name = "overwrite_text")]
    pub overwrite: Option<String>,

    #[clap(long, value_parser, help = "Export items to a CSV file at the specified path")]
    pub csv: Option<String>,

    #[clap(long, value_parser, help = "Minimum age of items (e.g., '1 week', '2 years', or '2024-01-15')")]
    pub min_age: Option<String>,

    #[clap(long, value_parser, help = "Maximum age of items (e.g., '1 week', '2 years', or '2024-01-15')")]
    pub max_age: Option<String>,
}
