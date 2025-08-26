use clap::{Parser, Subcommand};
use git_rs::cli::GitCommand;
use git_rs::domain::repository::GitCompatMode;

#[derive(Parser)]
#[command(name = "git-rs")]
#[command(about = "A minimal Git implementation in Rust for educational purposes")]
#[command(version = "0.1.0")]
struct Cli {
    /// Use .git directory instead of .git-rs (enables real Git compatibility mode)
    #[arg(
        long,
        global = true,
        help = "Use .git directory for Git compatibility (default: .git-rs)"
    )]
    git_compat: bool,

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

    // Convert the boolean flag to GitCompatMode
    let git_compat = if cli.git_compat {
        GitCompatMode::Compatible
    } else {
        GitCompatMode::Educational
    };

    match cli.command {
        Commands::Init => GitCommand::init_with_compat(git_compat)?,
        Commands::Add { files } => GitCommand::add_with_compat(&files, git_compat)?,
        Commands::Commit { message } => GitCommand::commit_with_compat(&message, git_compat)?,
        Commands::Diff { cached } => GitCommand::diff_with_compat(cached, git_compat)?,
        Commands::Clone { url, directory } => {
            GitCommand::clone_with_compat(&url, directory.as_deref(), git_compat)?
        }
        Commands::Status => GitCommand::status_with_compat(git_compat)?,
        Commands::Log { count } => GitCommand::log_with_compat(count, git_compat)?,
    }

    Ok(())
}
