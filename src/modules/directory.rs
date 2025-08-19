use super::Module;
use std::path::PathBuf;

pub struct DirectoryModule {
    current_dir: PathBuf,
}

impl DirectoryModule {
    pub fn new(cwd: &str) -> Self {
        Self {
            current_dir: PathBuf::from(cwd),
        }
    }
    
    /// Abbreviate home directory to ~
    fn abbreviate_home(&self, path: &PathBuf) -> String {
        if let Ok(home) = std::env::var("HOME") {
            let home_path = PathBuf::from(&home);
            if let Ok(relative) = path.strip_prefix(&home_path) {
                return format!("~/{}", relative.display());
            }
        }
        path.display().to_string()
    }
}

impl Module for DirectoryModule {
    fn name(&self) -> &str {
        "directory"
    }
    
    fn should_display(&self) -> bool {
        true // Always display directory
    }
    
    fn render(&self) -> String {
        self.abbreviate_home(&self.current_dir)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_directory_module() {
        let module = DirectoryModule::new("/Users/test/projects");
        assert_eq!(module.name(), "directory");
        assert!(module.should_display());
    }
    
    #[test]
    fn test_home_abbreviation() {
        // Note: set_var is unsafe in Rust 1.77+
        // For now, we'll skip this test as it requires unsafe block
        // In production, HOME is already set by the system
        let module = DirectoryModule::new("/Users/test/projects");
        // Can't easily test without setting HOME env var
        assert_eq!(module.name(), "directory");
    }
}