use crate::application::add::{AddCommand, AddOptions};
use crate::application::clone::{CloneCommand, CloneOptions};
use crate::application::commit::{CommitCommand, CommitOptions};
use crate::application::diff::{DiffCommand, DiffOptions};
use crate::application::init::InitCommand;
use crate::application::log::{LogCommand, LogOptions};
use crate::application::status::{StatusCommand, StatusOptions};
use crate::domain::repository::GitCompatMode;
use std::path::Path;

/// CLI Command Handler
///
/// This module provides the interface between the CLI and the application layer.
/// Each method corresponds to a Git command and handles the user interface aspects
/// while delegating the actual work to the application use cases.
pub struct GitCommand;

impl GitCommand {
    /// Handle `git init` command
    pub fn init() -> crate::Result<()> {
        println!("git-rs init");
        println!("============");

        let repo = InitCommand::init::<&Path>(None)?;
        let info = InitCommand::get_repository_info(&repo);

        println!("\n📊 Repository Summary:");
        println!("{}", info);

        Ok(())
    }

    /// Handle `git add` command
    pub fn add(files: &[String]) -> crate::Result<()> {
        println!("git-rs add {:?}", files);
        println!("==================");

        if files.is_empty() {
            return Err(
                "Nothing specified, nothing added.\nMaybe you wanted to say 'git add .'?".into(),
            );
        }

        let current_dir = std::env::current_dir()?;
        let options = AddOptions::default();

        let result = AddCommand::add(&current_dir, files, options)?;

        if result.has_failures() {
            for (path, error) in &result.failed_files {
                println!("⚠️  Failed to add {}: {}", path.display(), error);
            }
        }

        if result.total_staged() > 0 {
            println!("\n🎯 Successfully staged {} file(s)", result.total_staged());
        }

        Ok(())
    }

    /// Handle `git status` command
    pub fn status() -> crate::Result<()> {
        println!("git-rs status");
        println!("=============");

        let current_dir = std::env::current_dir()?;
        let options = StatusOptions::default();

        let _result = StatusCommand::status(&current_dir, options)?;

        Ok(())
    }

    /// Handle `git commit` command
    pub fn commit(message: &str) -> crate::Result<()> {
        println!("git-rs commit -m \"{}\"", message);
        println!("=======================");

        // Validate commit message
        CommitCommand::validate_message(message)?;

        let current_dir = std::env::current_dir()?;
        let options = CommitOptions::default();

        let result = CommitCommand::commit(&current_dir, message, options)?;

        println!("\n{}", result.summary());
        println!("📁 Tree: {}", result.tree_hash);
        println!("💬 Message: {}", result.message);

        if result.is_root_commit {
            println!("\n🌱 This is your first commit! Your git-rs journey begins.");
        }

        Ok(())
    }

    /// Handle `git diff` command
    pub fn diff(cached: bool) -> crate::Result<()> {
        if cached {
            println!("git-rs diff --cached");
        } else {
            println!("git-rs diff");
        }
        println!("=================");

        let current_dir = std::env::current_dir()?;
        let options = DiffOptions {
            cached,
            ..Default::default()
        };

        let result = DiffCommand::diff(&current_dir, options)?;

        if result.files_changed == 0 {
            if cached {
                println!("No changes between index and HEAD");
            } else {
                println!("No changes between working directory and index");
            }
        } else {
            result.print_unified();
        }

        Ok(())
    }

    /// Handle `git clone` command
    pub fn clone(url: &str, directory: Option<&str>) -> crate::Result<()> {
        match directory {
            Some(dir) => println!("git-rs clone {} {}", url, dir),
            None => println!("git-rs clone {}", url),
        }
        println!("====================");

        let options = CloneOptions::default();
        let result = CloneCommand::clone(url, directory, options)?;

        println!("\n📊 Clone Summary:");
        println!("{}", result.summary());

        Ok(())
    }

    /// Handle `git log` command
    pub fn log(count: Option<usize>) -> crate::Result<()> {
        println!("git-rs log");
        println!("==========");

        let options = LogOptions { max_count: count };

        let result = LogCommand::log(".", options)?;

        if result.entries.is_empty() {
            println!("📭 No commits found in this repository");
            return Ok(());
        }

        // Display the log entries
        print!("{}", LogCommand::format_log_result(&result));

        if result.has_more && count.is_some() {
            println!("💡 Use 'git-rs log' (without -n) to see all commits");
        }

        Ok(())
    }

    // Git compatibility methods

    /// Handle `git init` command with compatibility mode
    pub fn init_with_compat(git_compat: GitCompatMode) -> crate::Result<()> {
        println!("git-rs init");
        println!("============");

        let repo = InitCommand::init_with_compat::<&Path>(None, git_compat)?;
        let info = InitCommand::get_repository_info(&repo);

        println!("\n📊 Repository Summary:");
        println!("{}", info);

        Ok(())
    }

    /// Handle `git add` command with compatibility mode
    pub fn add_with_compat(files: &[String], _git_compat: GitCompatMode) -> crate::Result<()> {
        // For now, just delegate to the original add method
        // TODO: Pass git_compat to AddCommand when it supports it
        Self::add(files)
    }

    /// Handle `git commit` command with compatibility mode
    pub fn commit_with_compat(message: &str, _git_compat: GitCompatMode) -> crate::Result<()> {
        // For now, just delegate to the original commit method
        // TODO: Pass git_compat to CommitCommand when it supports it
        Self::commit(message)
    }

    /// Handle `git status` command with compatibility mode
    pub fn status_with_compat(_git_compat: GitCompatMode) -> crate::Result<()> {
        // For now, just delegate to the original status method
        // TODO: Pass git_compat to StatusCommand when it supports it
        Self::status()
    }

    /// Handle `git diff` command with compatibility mode
    pub fn diff_with_compat(staged: bool, _git_compat: GitCompatMode) -> crate::Result<()> {
        // For now, just delegate to the original diff method
        // TODO: Pass git_compat to DiffCommand when it supports it
        Self::diff(staged)
    }

    /// Handle `git clone` command with compatibility mode
    pub fn clone_with_compat(
        url: &str,
        directory: Option<&str>,
        _git_compat: GitCompatMode,
    ) -> crate::Result<()> {
        // For now, just delegate to the original clone method
        // TODO: Pass git_compat to CloneCommand when it supports it
        Self::clone(url, directory)
    }

    /// Handle `git log` command with compatibility mode
    pub fn log_with_compat(count: Option<usize>, git_compat: GitCompatMode) -> crate::Result<()> {
        println!("git-rs log");
        println!("==========");

        let options = LogOptions { max_count: count };

        let result = LogCommand::log_with_compat(".", options, git_compat)?;

        if result.entries.is_empty() {
            println!("📭 No commits found in this repository");
            return Ok(());
        }

        // Display the log entries
        print!("{}", LogCommand::format_log_result(&result));

        if result.has_more && count.is_some() {
            println!("💡 Use 'git-rs log' (without -n) to see all commits");
        }

        Ok(())
    }
}
