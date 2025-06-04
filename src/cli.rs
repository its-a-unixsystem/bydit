use clap::Parser;

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
pub struct Cli {
    #[clap(short, long, value_parser)]
    pub subreddit: Option<String>,

    #[clap(short = 'm', long, value_parser, name = "min_score")]
    pub score: Option<i32>,

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
}
