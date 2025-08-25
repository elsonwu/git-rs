use std::path::Path;
use crate::application::init::InitCommand;
use crate::application::add::{AddCommand, AddOptions};
use crate::application::status::{StatusCommand, StatusOptions};

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
            return Err("Nothing specified, nothing added.\nMaybe you wanted to say 'git add .'?".into());
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
    
    /// Handle `git commit` command (placeholder)
    pub fn commit(message: &str) -> crate::Result<()> {
        println!("git-rs commit -m \"{}\"", message);
        println!("⚠️  Commit functionality not implemented yet");
        Ok(())
    }
    
    /// Handle `git diff` command (placeholder)
    pub fn diff(cached: bool) -> crate::Result<()> {
        if cached {
            println!("git-rs diff --cached");
        } else {
            println!("git-rs diff");
        }
        println!("⚠️  Diff functionality not implemented yet");
        Ok(())
    }
    
    /// Handle `git clone` command (placeholder)
    pub fn clone(url: &str, directory: Option<&str>) -> crate::Result<()> {
        match directory {
            Some(dir) => println!("git-rs clone {} {}", url, dir),
            None => println!("git-rs clone {}", url),
        }
        println!("⚠️  Clone functionality not implemented yet");
        Ok(())
    }
    
    /// Handle `git log` command (placeholder)  
    pub fn log(count: Option<usize>) -> crate::Result<()> {
        match count {
            Some(n) => println!("git-rs log -n {}", n),
            None => println!("git-rs log"),
        }
        println!("⚠️  Log functionality not implemented yet");
        Ok(())
    }
}
