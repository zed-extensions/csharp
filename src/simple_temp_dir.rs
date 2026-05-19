pub struct SimpleTempDir {
    path: std::path::PathBuf,
}

impl SimpleTempDir {
    pub fn new(prefix: &str) -> Result<Self, String> {
        let temp_base = std::env::current_dir()
            .map_err(|e| format!("Failed to get current directory: {}", e))?;
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_err(|e| format!("Failed to get timestamp: {}", e))?
            .as_nanos();

        let dir_name = format!("{}{}", prefix, timestamp);
        let temp_path = temp_base.join(dir_name);

        std::fs::create_dir_all(&temp_path)
            .map_err(|e| format!("Failed to create temp directory: {}", e))?;

        Ok(SimpleTempDir { path: temp_path })
    }

    pub fn path(&self) -> &std::path::Path {
        &self.path
    }
}

impl Drop for SimpleTempDir {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.path);
    }
}
