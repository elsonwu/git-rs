use std::fs;
use std::path::{Path, PathBuf};

use crate::domain::repository::GitCompatMode;
use crate::domain::*;
use crate::infrastructure::*;

/// Git Init Use Case
///
/// This implements the `git init` command functionality.
///
/// ## What `git init` does:
/// 1. Creates the .git directory structure
/// 2. Initializes the object database (.git/objects)
/// 3. Creates the refs directory structure (.git/refs/heads, .git/refs/tags)
/// 4. Sets up initial HEAD to point to main branch
/// 5. Creates initial configuration
///
/// ## Visual Guide - .git Directory Structure After Init:
/// ```text
/// .git/
/// |-- objects/          (Object database - empty initially)
/// |   |-- info/
/// |   `-- pack/
/// |-- refs/             (References)
/// |   |-- heads/        (Branch references - empty initially)
/// |   `-- tags/         (Tag references - empty initially)
/// |-- HEAD              (Current branch: "ref: refs/heads/main")
/// |-- config            (Repository configuration)
/// `-- description       (Repository description)
/// ```
pub struct InitCommand;

impl InitCommand {
    /// Initialize a new Git repository
    ///
    /// # Arguments
    /// * `path` - Directory path where to initialize the repository (default: current directory)
    /// * `bare` - Whether to create a bare repository (not implemented in this educational version)
    ///
    /// # Returns
    /// * `Ok(GitRepository)` - The initialized repository
    /// * `Err(...)` - If initialization failed
    pub fn init<P: AsRef<Path>>(path: Option<P>) -> crate::Result<GitRepository> {
        Self::init_with_compat(path, GitCompatMode::Educational)
    }

    /// Initialize a new Git repository with compatibility mode
    ///
    /// # Arguments
    /// * `path` - Directory path where to initialize the repository (default: current directory)
    /// * `git_compat` - Git compatibility mode (Educational uses .git-rs, Compatible uses .git)
    ///
    /// # Returns
    /// * `Ok(GitRepository)` - The initialized repository
    /// * `Err(...)` - If initialization failed
    pub fn init_with_compat<P: AsRef<Path>>(
        path: Option<P>,
        git_compat: GitCompatMode,
    ) -> crate::Result<GitRepository> {
        let repo_path = match path {
            Some(p) => p.as_ref().to_path_buf(),
            None => std::env::current_dir()?,
        };

        println!("🚀 Initializing Git repository in {:?}", repo_path);

        // Create repository instance with compatibility mode
        let repo = GitRepository::new_with_compat(&repo_path, git_compat);

        // Check if already a git repository
        if repo.is_repository() {
            return Err(format!("Repository already exists at {:?}", repo.git_dir()).into());
        }

        // Create .git directory structure
        Self::create_git_directory_structure(&repo, git_compat)?;

        // Initialize object store
        Self::initialize_object_store(&repo)?;

        // Initialize reference store
        Self::initialize_reference_store(&repo)?;

        // Create initial configuration
        Self::create_initial_config(&repo, git_compat)?;

        // Create repository description
        Self::create_description(&repo, git_compat)?;

        println!(
            "✅ Initialized empty Git repository in {:?}",
            repo.git_dir()
        );

        Ok(repo)
    }

    /// Create the basic .git directory structure
    fn create_git_directory_structure(
        repo: &GitRepository,
        git_compat: GitCompatMode,
    ) -> crate::Result<()> {
        let git_dir_name = match git_compat {
            GitCompatMode::Educational => ".git-rs",
            GitCompatMode::Compatible => ".git",
        };
        println!("📁 Creating {} directory structure...", git_dir_name);

        // Create main .git directory
        fs::create_dir_all(repo.git_dir())?;

        // Create objects directory
        fs::create_dir_all(repo.objects_dir())?;
        fs::create_dir_all(repo.objects_dir().join("info"))?;
        fs::create_dir_all(repo.objects_dir().join("pack"))?;

        // Create refs directory
        fs::create_dir_all(repo.refs_dir())?;
        fs::create_dir_all(repo.heads_dir())?;
        fs::create_dir_all(repo.tags_dir())?;

        println!("   ✓ Created {}/objects/ (object database)", git_dir_name);
        println!(
            "   ✓ Created {}/refs/heads/ (branch references)",
            git_dir_name
        );
        println!("   ✓ Created {}/refs/tags/ (tag references)", git_dir_name);

        Ok(())
    }

    /// Initialize the object store
    fn initialize_object_store(repo: &GitRepository) -> crate::Result<()> {
        println!("🗃️  Initializing object store...");

        let object_store = ObjectStore::new(repo.objects_dir());
        object_store.init()?;

        println!("   ✓ Object store ready for storing blobs, trees, and commits");

        Ok(())
    }

    /// Initialize the reference store with default HEAD
    fn initialize_reference_store(repo: &GitRepository) -> crate::Result<()> {
        println!("🔗 Initializing references...");

        let ref_store = RefStore::new(repo.git_dir().to_path_buf());
        ref_store.init()?;

        // Set HEAD to point to main branch (even though main doesn't exist yet)
        // This is what real Git does - HEAD points to a branch that will be created on first commit
        ref_store.set_head_to_branch("main")?;

        println!("   ✓ Created HEAD pointing to refs/heads/main");

        Ok(())
    }

    /// Create initial repository configuration
    fn create_initial_config(repo: &GitRepository, git_compat: GitCompatMode) -> crate::Result<()> {
        println!("⚙️  Creating initial configuration...");

        let config_content = r#"[core]
	repositoryformatversion = 0
	filemode = true
	bare = false
	logallrefupdates = true
[user]
	name = Git User
	email = user@example.com
"#;

        fs::write(repo.config_path(), config_content)?;

        let git_dir_name = match git_compat {
            GitCompatMode::Educational => ".git-rs",
            GitCompatMode::Compatible => ".git",
        };
        println!("   ✓ Created {}/config with default settings", git_dir_name);

        Ok(())
    }

    /// Create repository description
    fn create_description(repo: &GitRepository, git_compat: GitCompatMode) -> crate::Result<()> {
        let description_path = repo.git_dir().join("description");
        let description_content =
            "Unnamed repository; edit this file 'description' to name the repository.\n";

        fs::write(description_path, description_content)?;

        let git_dir_name = match git_compat {
            GitCompatMode::Educational => ".git-rs",
            GitCompatMode::Compatible => ".git",
        };
        println!("   ✓ Created {}/description", git_dir_name);

        Ok(())
    }

    /// Check if a directory is already a Git repository
    pub fn is_git_repository<P: AsRef<Path>>(path: P) -> bool {
        let repo = GitRepository::new(path);
        repo.is_repository()
    }

    /// Get repository information after initialization
    pub fn get_repository_info(repo: &GitRepository) -> RepositoryInfo {
        RepositoryInfo {
            root_path: repo.root_path().to_path_buf(),
            git_dir: repo.git_dir().to_path_buf(),
            is_bare: false, // We don't support bare repos in this educational version
            current_branch: None, // No branches exist yet
            head_commit: None, // No commits exist yet
        }
    }
}

/// Repository information structure
#[derive(Debug, Clone)]
pub struct RepositoryInfo {
    pub root_path: PathBuf,
    pub git_dir: PathBuf,
    pub is_bare: bool,
    pub current_branch: Option<String>,
    pub head_commit: Option<ObjectHash>,
}

impl std::fmt::Display for RepositoryInfo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "Repository Information:")?;
        writeln!(f, "  Root: {:?}", self.root_path)?;
        writeln!(f, "  .git: {:?}", self.git_dir)?;
        writeln!(f, "  Bare: {}", self.is_bare)?;
        writeln!(f, "  Current Branch: {:?}", self.current_branch)?;
        writeln!(f, "  HEAD Commit: {:?}", self.head_commit)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_init_new_repository() {
        let temp_dir = tempdir().unwrap();
        let repo_path = temp_dir.path();

        // Initialize repository
        let repo = InitCommand::init(Some(repo_path)).unwrap();

        // Verify repository structure
        assert!(repo.is_repository());
        assert!(repo.git_dir().exists());
        assert!(repo.objects_dir().exists());
        assert!(repo.refs_dir().exists());
        assert!(repo.heads_dir().exists());
        assert!(repo.tags_dir().exists());
        assert!(repo.head_path().exists());
        assert!(repo.config_path().exists());

        // Verify HEAD points to main branch
        let ref_store = RefStore::new(repo.git_dir().to_path_buf());
        let head = ref_store.load_head().unwrap().unwrap();
        assert_eq!(head, HeadRef::symbolic("main"));
    }

    #[test]
    fn test_init_already_exists() {
        let temp_dir = tempdir().unwrap();
        let repo_path = temp_dir.path();

        // Initialize repository first time
        InitCommand::init(Some(repo_path)).unwrap();

        // Try to initialize again - should fail
        let result = InitCommand::init(Some(repo_path));
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("already exists"));
    }

    #[test]
    fn test_is_git_repository() {
        let temp_dir = tempdir().unwrap();
        let repo_path = temp_dir.path();

        // Should not be a git repo initially
        assert!(!InitCommand::is_git_repository(repo_path));

        // Initialize repository
        InitCommand::init(Some(repo_path)).unwrap();

        // Should be a git repo now
        assert!(InitCommand::is_git_repository(repo_path));
    }

    #[test]
    fn test_repository_info() {
        let temp_dir = tempdir().unwrap();
        let repo_path = temp_dir.path();

        let repo = InitCommand::init(Some(repo_path)).unwrap();
        let info = InitCommand::get_repository_info(&repo);

        assert_eq!(info.root_path, repo_path);
        assert_eq!(info.git_dir, repo_path.join(".git-rs"));
        assert!(!info.is_bare);
        assert_eq!(info.current_branch, None);
        assert_eq!(info.head_commit, None);
    }
}
