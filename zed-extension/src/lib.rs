use std::fs;
use zed_extension_api::{self as zed, Result};

struct IdlExtension {
    cached_binary_path: Option<String>,
}

impl zed::Extension for IdlExtension {
    fn new() -> Self {
        Self {
            cached_binary_path: None,
        }
    }

    fn language_server_command(
        &mut self,
        language_server_id: &zed::LanguageServerId,
        worktree: &zed::Worktree,
    ) -> Result<zed::Command> {
        if let Some(path) = worktree.which("idl-language-server") {
            return Ok(zed::Command {
                command: path,
                args: vec![],
                env: Default::default(),
            });
        }

        let path = self.language_server_binary_path(language_server_id)?;
        Ok(zed::Command {
            command: path,
            args: vec![],
            env: Default::default(),
        })
    }
}

impl IdlExtension {
    fn language_server_binary_path(&mut self, language_server_id: &zed::LanguageServerId) -> Result<String> {
        if let Some(path) = &self.cached_binary_path {
            if fs::metadata(path).map_or(false, |stat| stat.is_file()) {
                return Ok(path.clone());
            }
        }

        zed::set_language_server_installation_status(
            language_server_id,
            &zed::LanguageServerInstallationStatus::CheckingForUpdate,
        );

        let release = zed::latest_github_release(
            "xidl/idl-language-server",
            zed::GithubReleaseOptions {
                require_assets: true,
                pre_release: false,
            },
        )?;

        let (os, arch) = zed::current_platform();
        let asset_name = match (os, arch) {
            (zed::Os::Mac, zed::Architecture::Aarch64) => "idl-language-server-aarch64-apple-darwin.tar.gz",
            (zed::Os::Linux, zed::Architecture::X8664) => "idl-language-server-x86_64-unknown-linux-musl.tar.gz",
            (zed::Os::Windows, zed::Architecture::X8664) => "idl-language-server-x86_64-pc-windows-gnu.tar.gz",
            _ => return Err(format!("unsupported platform: {:?} {:?}", os, arch)),
        };

        let asset = release
            .assets
            .iter()
            .find(|asset| asset.name == asset_name)
            .ok_or_else(|| format!("no asset found matching {:?}", asset_name))?;

        let version_dir = format!("idl-language-server-{}", release.version);
        let binary_path = format!("{}/idl-language-server", version_dir);

        if !fs::metadata(&binary_path).map_or(false, |stat| stat.is_file()) {
            zed::set_language_server_installation_status(
                language_server_id,
                &zed::LanguageServerInstallationStatus::Downloading,
            );

            zed::download_file(
                &asset.download_url,
                &version_dir,
                zed::DownloadedFileType::GzipTar,
            )
            .map_err(|e| format!("failed to download language server: {e}"))?;

            let entries = fs::read_dir(&version_dir)
                .map_err(|e| format!("failed to list downloaded directory: {e}"))?;

            for entry in entries {
                let entry = entry.map_err(|e| format!("failed to read entry: {e}"))?;
                if entry.file_name().to_string_lossy().contains("idl-language-server") {
                    if let Some(path_str) = entry.path().to_str() {
                        zed::make_file_executable(path_str)
                            .map_err(|e| format!("failed to make binary executable: {e}"))?;
                    }
                    self.cached_binary_path = Some(entry.path().to_string_lossy().to_string());
                    return Ok(entry.path().to_string_lossy().to_string());
                }
            }

            zed::make_file_executable(&binary_path)
                .map_err(|e| format!("failed to make binary executable: {e}"))?;
        }

        self.cached_binary_path = Some(binary_path.clone());
        Ok(binary_path)
    }
}

zed::register_extension!(IdlExtension);
