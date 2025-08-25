use clap::{Parser, Subcommand};
use git_rs::cli::GitCommand;

#[derive(Parser)]
#[command(name = "git-rs")]
#[command(about = "A minimal Git implementation in Rust for educational purposes")]
#[command(version = "0.1.0")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Initialize a new Git repository
    Init,
    /// Add files to the staging area
    Add {
        /// Files to add
        files: Vec<String>,
    },
    /// Create a new commit
    Commit {
        /// Commit message
        #[arg(short, long)]
        message: String,
    },
    /// Show changes between commits, working tree, etc.
    Diff {
        /// Show staged changes instead of unstaged
        #[arg(long)]
        cached: bool,
    },
    /// Clone a repository
    Clone {
        /// Repository URL or path
        url: String,
        /// Directory name
        directory: Option<String>,
    },
    /// Show repository status
    Status,
    /// Show commit logs
    Log {
        /// Number of commits to show
        #[arg(short = 'n', long)]
        count: Option<usize>,
    },
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();
    
    match cli.command {
        Commands::Init => GitCommand::init()?,
        Commands::Add { files } => GitCommand::add(&files)?,
        Commands::Commit { message } => GitCommand::commit(&message)?,
        Commands::Diff { cached } => GitCommand::diff(cached)?,
        Commands::Clone { url, directory } => GitCommand::clone(&url, directory.as_deref())?,
        Commands::Status => GitCommand::status()?,
        Commands::Log { count } => GitCommand::log(count)?,
    }
    
    Ok(())
}
