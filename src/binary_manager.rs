use crate::simple_temp_dir::SimpleTempDir;
use fs_extra::dir;
use std::sync::OnceLock;
use zed_extension_api::{self as zed, DownloadedFileType, GithubReleaseOptions};

/// GitHub release version information
#[derive(Debug, Clone)]
pub struct AdapterVersion {
    /// Release tag name (version)
    pub tag_name: String,
    /// Download URL for the release asset
    pub download_url: String,
}

pub struct BinaryManager {
    /// Cached path to the netcoredbg binary - set once and reused
    cached_binary_path: OnceLock<String>,
}

impl Default for BinaryManager {
    fn default() -> Self {
        Self::new()
    }
}

impl BinaryManager {
    const GITHUB_OWNER: &str = "qwadrox";
    const GITHUB_REPO: &str = "netcoredbg";

    pub fn new() -> Self {
        Self {
            cached_binary_path: OnceLock::new(),
        }
    }

    fn get_executable_name() -> &'static str {
        if zed::current_platform().0 == zed::Os::Windows {
            "netcoredbg.exe"
        } else {
            "netcoredbg"
        }
    }

    /// Determines the appropriate asset name for the current platform
    /// Supported assets:
    /// - netcoredbg-linux-amd64.tar.gz
    /// - netcoredbg-linux-arm64.tar.gz
    /// - netcoredbg-osx-amd64.tar.gz
    /// - netcoredbg-osx-arm64.tar.gz
    /// - netcoredbg-win64.zip
    fn get_platform_asset_name() -> Result<String, String> {
        let (platform, arch) = zed::current_platform();

        let (platform_arch, extension) = match (platform, arch) {
            (zed::Os::Linux, zed::Architecture::X8664) => ("linux-amd64", ".tar.gz"),
            (zed::Os::Linux, zed::Architecture::Aarch64) => ("linux-arm64", ".tar.gz"),
            (zed::Os::Mac, zed::Architecture::X8664) => ("osx-amd64", ".tar.gz"),
            (zed::Os::Mac, zed::Architecture::Aarch64) => ("osx-arm64", ".tar.gz"),
            (zed::Os::Windows, zed::Architecture::X8664) => ("win64", ".zip"),
            (zed::Os::Windows, zed::Architecture::Aarch64) => {
                // Windows ARM64 is not officially supported by netcoredbg,
                // but we can try the x64 version as a fallback
                ("win64", ".zip")
            }
            (_, zed::Architecture::X86) => {
                return Err(
                    "Unsupported architecture: x86 (32-bit). NetCoreDbg only supports 64-bit architectures (amd64/arm64).".to_string(),
                );
            }
        };

        Ok(format!("netcoredbg-{}{}", platform_arch, extension))
    }

    /// Fetches the latest release information from GitHub
    fn fetch_latest_release(&self) -> Result<AdapterVersion, String> {
        let release = zed::latest_github_release(
            &format!("{}/{}", Self::GITHUB_OWNER, Self::GITHUB_REPO),
            GithubReleaseOptions {
                require_assets: true,
                pre_release: false,
            },
        )
        .map_err(|e| format!("Failed to fetch latest release: {}", e))?;

        let asset_name = Self::get_platform_asset_name()?;

        let asset = release
            .assets
            .iter()
            .find(|asset| asset.name == asset_name)
            .ok_or_else(|| {
                format!(
                    "No compatible asset found for platform. Looking for: '{}'. Available assets: [{}]",
                    asset_name,
                    release
                        .assets
                        .iter()
                        .map(|a| a.name.as_str())
                        .collect::<Vec<_>>()
                        .join(", ")
                )
            })?;

        Ok(AdapterVersion {
            tag_name: release.version,
            download_url: asset.download_url.clone(),
        })
    }

    /// Creates a temporary directory for extraction
    fn create_temp_dir(&self, version: &str) -> Result<SimpleTempDir, String> {
        SimpleTempDir::new(&format!("netcoredbg_v{}_", version))
    }

    /// Downloads and extracts the netcoredbg binary, returning the path to the executable
    fn download_and_extract_binary(&self) -> Result<String, String> {
        let version = self.fetch_latest_release()?;
        let asset_name = Self::get_platform_asset_name()?;

        let file_type = if asset_name.ends_with(".zip") {
            DownloadedFileType::Zip
        } else if asset_name.ends_with(".tar.gz") {
            DownloadedFileType::GzipTar
        } else {
            return Err(format!("Unsupported file type for asset: {}", asset_name));
        };

        // Version-specific directory in current working directory
        let version_dir = std::path::PathBuf::from(format!("netcoredbg_v{}", version.tag_name));

        let temp_dir = self.create_temp_dir(&version.tag_name)?;
        zed::download_file(
            &version.download_url,
            &temp_dir.path().to_string_lossy(),
            file_type,
        )
        .map_err(|e| format!("Failed to download netcoredbg: {}", e))?;

        std::fs::create_dir_all(&version_dir)
            .map_err(|e| format!("Failed to create version directory: {}", e))?;

        self.copy_extracted_content(temp_dir.path(), &version_dir)?;

        let exe_name = Self::get_executable_name();

        let binary_path = version_dir.join(exe_name);

        if !binary_path.exists() {
            return Err(format!(
                "netcoredbg executable not found at: {}",
                binary_path.display()
            ));
        }

        zed::make_file_executable(&binary_path.to_string_lossy())
            .map_err(|e| format!("Failed to make file executable: {}", e))?;

        let current_dir = std::env::current_dir()
            .map_err(|e| format!("Failed to get current directory: {}", e))?;
        let absolute_path = current_dir.join(&binary_path);
        Ok(absolute_path.to_string_lossy().to_string())
    }

    /// Copies extracted content from temp_dir into version_dir, handling nested directory structure
    fn copy_extracted_content(
        &self,
        temp_dir: &std::path::Path,
        version_dir: &std::path::Path,
    ) -> Result<(), String> {
        let exe_name = Self::get_executable_name();

        let binary_source_path = self.find_binary_in_extracted_content(temp_dir, exe_name)?;

        let source_dir = binary_source_path
            .parent()
            .ok_or_else(|| "Binary has no parent directory".to_string())?;

        let copy_options = dir::CopyOptions::new().content_only(true);

        dir::copy(source_dir, version_dir, &copy_options).map_err(|e| {
            format!(
                "Failed to copy extracted content from {}: {}",
                source_dir.display(),
                e
            )
        })?;

        Ok(())
    }

    /// Recursively searches for the netcoredbg binary in the extracted content
    fn find_binary_in_extracted_content(
        &self,
        search_dir: &std::path::Path,
        exe_name: &str,
    ) -> Result<std::path::PathBuf, String> {
        fn find_binary_recursive(
            dir: &std::path::Path,
            exe_name: &str,
        ) -> Result<Option<std::path::PathBuf>, String> {
            let entries = std::fs::read_dir(dir)
                .map_err(|e| format!("Failed to read directory {}: {}", dir.display(), e))?;

            for entry in entries {
                let entry = entry.map_err(|e| format!("Failed to read directory entry: {}", e))?;
                let path = entry.path();

                if path.is_file() && path.file_name().is_some_and(|name| name == exe_name) {
                    return Ok(Some(path));
                } else if path.is_dir() {
                    if let Some(found) = find_binary_recursive(&path, exe_name)? {
                        return Ok(Some(found));
                    }
                }
            }
            Ok(None)
        }

        find_binary_recursive(search_dir, exe_name)?.ok_or_else(|| {
            format!(
                "Could not find {} binary in extracted content at {}",
                exe_name,
                search_dir.display()
            )
        })
    }

    /// Gets the netcoredbg binary path, downloading if necessary
    pub fn get_binary_path(&self, user_provided_path: Option<String>) -> Result<String, String> {
        // Priority 1: User-provided path return as is without any validation
        if let Some(user_path) = user_provided_path {
            return Ok(user_path);
        }

        // Priority 2: Check in-memory cache
        if let Some(cached_path) = self.cached_binary_path.get() {
            if std::path::Path::new(cached_path).exists() {
                return Ok(cached_path.clone());
            }
        }

        // Priority 3: Check existing binary on disk before downloading
        let version = self.fetch_latest_release()?;

        // Version-specific directory in current working directory
        let version_dir = std::path::PathBuf::from(format!("netcoredbg_v{}", version.tag_name));
        let exe_name = Self::get_executable_name();
        let existing_binary_path = version_dir.join(exe_name);
        if existing_binary_path.exists() {
            let current_dir = std::env::current_dir()
                .map_err(|e| format!("Failed to get current directory: {}", e))?;
            let absolute_path = current_dir.join(&existing_binary_path);
            let path_str = absolute_path.to_string_lossy().to_string();
            let _ = self.cached_binary_path.set(path_str.clone());
            return Ok(path_str);
        }

        // Priority 4: Download and extract from GitHub releases
        let binary_path = self.download_and_extract_binary()?;

        let _ = self.cached_binary_path.set(binary_path.clone());

        self.validate_binary(&binary_path)?;

        Ok(binary_path)
    }

    /// Validates that the binary exists
    fn validate_binary(&self, binary_path: &str) -> Result<(), String> {
        let path = std::path::Path::new(binary_path);

        if !path.exists() {
            return Err(format!("netcoredbg binary not found at: {}", binary_path));
        }

        if !path.is_file() {
            return Err(format!("netcoredbg path is not a file: {}", binary_path));
        }

        Ok(())
    }
}
