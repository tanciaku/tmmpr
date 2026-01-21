use std::{fs, path::PathBuf};
use tempfile::NamedTempFile;

/// Trait for filesystem operations to enable testing with mocks
pub trait FileSystem {
    /// Get the user's home directory
    fn get_home_dir(&self) -> Option<PathBuf>;

    /// Create directory and all parent directories
    fn create_dir_all(&self, path: &PathBuf) -> Result<(), std::io::Error>;

    /// Check if a path exists
    fn path_exists(&self, path: &PathBuf) -> bool;

    /// Test if we can write to a directory by creating a temp file
    fn test_write_to_dir(&self, path: &PathBuf) -> Result<(), std::io::Error>;
}

/// Production implementation that uses the real filesystem
#[derive(Debug, Clone, Copy)]
pub struct RealFileSystem;


impl FileSystem for RealFileSystem {
    fn get_home_dir(&self) -> Option<PathBuf> {
        home::home_dir()
    }

    fn create_dir_all(&self, path: &PathBuf) -> Result<(), std::io::Error> {
        fs::create_dir_all(path)
    }

    fn path_exists(&self, path: &PathBuf) -> bool {
        path.exists()
    }

    fn test_write_to_dir(&self, path: &PathBuf) -> Result<(), std::io::Error> {
        NamedTempFile::new_in(path)?;
        Ok(())
    }
}

#[cfg(test)]
pub mod test_utils {
    use super::{FileSystem, PathBuf};
    use std::collections::HashSet;
    use std::io;

    /// Mock filesystem for testing
    pub struct MockFileSystem {
        pub existing_paths: HashSet<PathBuf>,
        pub home_dir: Option<PathBuf>,
        pub fail_dir_create: bool,
        pub fail_write_test: bool,
    }

    impl MockFileSystem {
        /// Create a new mock filesystem with default success behavior
        pub fn new() -> Self {
            MockFileSystem {
                existing_paths: HashSet::new(),
                home_dir: Some(PathBuf::from("/mock/home")),
                fail_dir_create: false,
                fail_write_test: false,
            }
        }

        /// Builder: Add a path that should exist
        pub fn with_existing_path(mut self, path: PathBuf) -> Self {
            self.existing_paths.insert(path);
            self
        }

        /// Builder: Set the home directory (None simulates missing home)
        pub fn with_home_dir(mut self, home: Option<PathBuf>) -> Self {
            self.home_dir = home;
            self
        }

        /// Builder: Make directory creation fail
        pub fn with_dir_create_failure(mut self) -> Self {
            self.fail_dir_create = true;
            self
        }

        /// Builder: Make write test fail
        pub fn with_write_failure(mut self) -> Self {
            self.fail_write_test = true;
            self
        }
    }

    impl FileSystem for MockFileSystem {
        fn get_home_dir(&self) -> Option<PathBuf> {
            self.home_dir.clone()
        }

        fn create_dir_all(&self, _path: &PathBuf) -> Result<(), io::Error> {
            if self.fail_dir_create {
                Err(io::Error::new(io::ErrorKind::PermissionDenied, "mock error"))
            } else {
                Ok(())
            }
        }

        fn path_exists(&self, path: &PathBuf) -> bool {
            self.existing_paths.contains(path)
        }

        fn test_write_to_dir(&self, _path: &PathBuf) -> Result<(), io::Error> {
            if self.fail_write_test {
                Err(io::Error::new(io::ErrorKind::PermissionDenied, "mock error"))
            } else {
                Ok(())
            }
        }
    }
}