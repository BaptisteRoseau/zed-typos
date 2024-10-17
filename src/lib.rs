use std::{fs, path::Path};

use zed_extension_api::{
    self as zed, settings::LspSettings, Architecture, Command, LanguageServerId, Os, Result,
    Worktree,
};

struct TyposBinary {
    path: String,
    args: Option<Vec<String>>,
}

struct TyposExtension {
    cached_binary_path: Option<String>,
}

impl TyposExtension {
    #[allow(dead_code)]
    pub const LANGUAGE_SERVER_ID: &'static str = "typos";

    fn language_server_binary(
        &mut self,
        language_server_id: &LanguageServerId,
        worktree: &zed::Worktree,
    ) -> Result<TyposBinary> {
        if let Some(path) = worktree.which("typos-lsp") {
            return Ok(TyposBinary {
                path,
                args: Some(vec![]),
            });
        }

        if let Some(path) = &self.cached_binary_path {
            if fs::metadata(path).map_or(false, |stat| stat.is_file()) {
                return Ok(TyposBinary {
                    path: path.clone(),
                    args: Some(vec![]),
                });
            }
        }

        zed::set_language_server_installation_status(
            language_server_id,
            &zed::LanguageServerInstallationStatus::CheckingForUpdate,
        );
        let release = zed::latest_github_release(
            "tekumara/typos-lsp",
            zed::GithubReleaseOptions {
                require_assets: true,
                pre_release: false,
            },
        )?;

        let (platform, architecture) = zed::current_platform();
        let version = release.version;

        let asset_name = Self::binary_release_name(&version, &platform, &architecture);
        let asset = release
            .assets
            .iter()
            .find(|asset| asset.name == asset_name)
            .ok_or_else(|| format!("no asset found matching {:?}", asset_name))?;

        let version_dir = format!("typos-lsp-{}", version);
        let binary_path = Path::new(&version_dir)
            .join(Self::binary_path_within_archive(&platform, &architecture))
            .to_str()
            .expect("Could not convert binary path to str")
            .to_string();

        if !fs::metadata(&binary_path).map_or(false, |stat| stat.is_file()) {
            zed::set_language_server_installation_status(
                language_server_id,
                &zed::LanguageServerInstallationStatus::Downloading,
            );
            let file_kind = match platform {
                zed::Os::Windows => zed::DownloadedFileType::Zip,
                _ => zed::DownloadedFileType::GzipTar,
            };
            zed::download_file(&asset.download_url, &version_dir, file_kind)
                .map_err(|e| format!("failed to download file: {e}"))?;

            Self::clean_other_installations(&version_dir)?;
        }

        self.cached_binary_path = Some(binary_path.clone());
        Ok(TyposBinary {
            path: binary_path,
            args: Some(vec![]),
        })
    }

    /// The name of the archive found under the "Release" tabs of the GitHub repository,
    /// depending on the version, platform and architecture.
    fn binary_release_name(version: &String, platform: &Os, architecture: &Architecture) -> String {
        format!(
            "typos-lsp-{version}-{arch}-{os}.{ext}",
            version = version,
            arch = match architecture {
                Architecture::Aarch64 => "aarch64",
                Architecture::X86 | Architecture::X8664 => "x86_64",
            },
            os = match platform {
                zed::Os::Mac => "apple-darwin",
                zed::Os::Linux => "unknown-linux-gnu",
                zed::Os::Windows => "pc-windows-msvc",
            },
            ext = match platform {
                zed::Os::Windows => "zip",
                _ => "tar.gz",
            }
        )
    }

    /// The path of the binary inside the archive.
    fn binary_path_within_archive(platform: &Os, architecture: &Architecture) -> String {
        let path = match platform {
            zed::Os::Windows => Path::new("target")
                .join(format!(
                    "{arch}-{os}",
                    arch = match architecture {
                        Architecture::Aarch64 => "aarch64",
                        Architecture::X86 | Architecture::X8664 => "x86_64",
                    },
                    os = match platform {
                        zed::Os::Mac => "apple-darwin",
                        zed::Os::Linux => "unknown-linux-gnu",
                        zed::Os::Windows => "pc-windows-msvc",
                    },
                ))
                .join("release")
                .join("typos-lsp.exe")
                .as_path()
                .to_owned(),
            _ => Path::new("typos-lsp").to_owned(),
        };
        path.to_str()
            .expect("Could not convert binary path to str")
            .to_string()
    }

    /// Remove every typos-lsp version directories within its Zed extension directory,
    /// except for the version specified as [`version_to_keep`].
    fn clean_other_installations(version_to_keep: &String) -> Result<(), String> {
        let entries =
            fs::read_dir(".").map_err(|e| format!("failed to list working directory {e}"))?;
        for entry in entries {
            let entry = entry.map_err(|e| format!("failed to load directory entry {e}"))?;
            if entry.file_name().to_str() != Some(version_to_keep) {
                fs::remove_dir_all(entry.path()).ok();
            }
        }
        Ok(())
    }
}

impl zed::Extension for TyposExtension {
    fn new() -> Self {
        Self {
            cached_binary_path: None,
        }
    }

    fn language_server_command(
        &mut self,
        language_server_id: &LanguageServerId,
        worktree: &Worktree,
    ) -> Result<Command> {
        let typos_binary = self.language_server_binary(language_server_id, worktree)?;

        Ok(zed::Command {
            command: typos_binary.path,
            args: typos_binary.args.unwrap(),
            env: Default::default(),
        })
    }

    fn language_server_initialization_options(
        &mut self,
        server_id: &LanguageServerId,
        worktree: &zed_extension_api::Worktree,
    ) -> Result<Option<zed_extension_api::serde_json::Value>> {
        let settings = LspSettings::for_worktree(server_id.as_ref(), worktree)
            .ok()
            .and_then(|lsp_settings| lsp_settings.initialization_options.clone())
            .unwrap_or_default();
        Ok(Some(settings))
    }

    fn language_server_workspace_configuration(
        &mut self,
        server_id: &LanguageServerId,
        worktree: &zed_extension_api::Worktree,
    ) -> Result<Option<zed_extension_api::serde_json::Value>> {
        let settings = LspSettings::for_worktree(server_id.as_ref(), worktree)
            .ok()
            .and_then(|lsp_settings| lsp_settings.settings.clone())
            .unwrap_or_default();
        Ok(Some(settings))
    }
}

zed::register_extension!(TyposExtension);

#[cfg(test)]
mod tests {
    use zed_extension_api::{Architecture, Os};

    use crate::TyposExtension;

    #[test]
    fn release_name() {
        assert_eq!(
            TyposExtension::binary_release_name(
                &"v0.1.23".to_string(),
                &Os::Mac,
                &Architecture::Aarch64
            ),
            "typos-lsp-v0.1.23-aarch64-apple-darwin.tar.gz".to_string()
        );
        assert_eq!(
            TyposExtension::binary_release_name(
                &"v0.1.23".to_string(),
                &Os::Windows,
                &Architecture::Aarch64
            ),
            "typos-lsp-v0.1.23-aarch64-pc-windows-msvc.zip".to_string()
        );
        assert_eq!(
            TyposExtension::binary_release_name(
                &"v0.1.23".to_string(),
                &Os::Linux,
                &Architecture::Aarch64
            ),
            "typos-lsp-v0.1.23-aarch64-unknown-linux-gnu.tar.gz".to_string()
        );
        assert_eq!(
            TyposExtension::binary_release_name(
                &"v0.1.23".to_string(),
                &Os::Mac,
                &Architecture::X86
            ),
            "typos-lsp-v0.1.23-x86_64-apple-darwin.tar.gz".to_string()
        );
        assert_eq!(
            TyposExtension::binary_release_name(
                &"v0.1.23".to_string(),
                &Os::Windows,
                &Architecture::X86
            ),
            "typos-lsp-v0.1.23-x86_64-pc-windows-msvc.zip".to_string()
        );
        assert_eq!(
            TyposExtension::binary_release_name(
                &"v0.1.23".to_string(),
                &Os::Linux,
                &Architecture::X86
            ),
            "typos-lsp-v0.1.23-x86_64-unknown-linux-gnu.tar.gz".to_string()
        );
        assert_eq!(
            TyposExtension::binary_release_name(
                &"v0.1.23".to_string(),
                &Os::Mac,
                &Architecture::X8664
            ),
            "typos-lsp-v0.1.23-x86_64-apple-darwin.tar.gz".to_string()
        );
        assert_eq!(
            TyposExtension::binary_release_name(
                &"v0.1.23".to_string(),
                &Os::Windows,
                &Architecture::X8664
            ),
            "typos-lsp-v0.1.23-x86_64-pc-windows-msvc.zip".to_string()
        );
        assert_eq!(
            TyposExtension::binary_release_name(
                &"v0.1.23".to_string(),
                &Os::Linux,
                &Architecture::X8664
            ),
            "typos-lsp-v0.1.23-x86_64-unknown-linux-gnu.tar.gz".to_string()
        );
    }

    #[test]
    #[cfg(target_os = "linux")]
    fn binary_name_within_extension() {
        assert_eq!(
            TyposExtension::binary_path_within_archive(&Os::Mac, &Architecture::X8664),
            "typos-lsp".to_string()
        );
        assert_eq!(
            TyposExtension::binary_path_within_archive(&Os::Windows, &Architecture::X8664),
            "target/x86_64-pc-windows-msvc/release/typos-lsp.exe".to_string()
        );
    }
}
