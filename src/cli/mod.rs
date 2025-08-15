use clap::Parser;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct Cli {
    /// Description of the command to execute
    pub prompt: Vec<String>,

    /// Run the generated program without asking for confirmation
    #[clap(short = 'y', long)]
    pub force: bool,
}