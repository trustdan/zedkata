use zed_extension_api::{self as zed, Command, LanguageServerId, Worktree};

/// Formats an actionable error message when the native language server binary cannot be located.
pub fn binary_not_found_message(is_windows: bool) -> String {
    let binary_name = if is_windows {
        "zedkata-lsp.exe"
    } else {
        "zedkata-lsp"
    };
    format!(
        "Zedkata LSP server binary ('{binary_name}') was not found on PATH.\n\
         To install it from the repository, run:\n\
           cargo install --path crates/zedkata-lsp\n\
         Or build it and ensure the binary directory is included in your PATH:\n\
           cargo build -p zedkata-lsp\n\
         Then restart Zed or reload the extension."
    )
}

/// Constructs the `Command` to launch the language server over stdio.
/// Note that paths with spaces are preserved safely as a single argument.
pub fn build_command(binary_path: String) -> Command {
    Command {
        command: binary_path,
        args: vec!["--stdio".to_string()],
        env: vec![],
    }
}

/// The Zedkata production extension for Zed.
struct ZedkataExtension;

impl zed::Extension for ZedkataExtension {
    fn new() -> Self {
        Self
    }

    fn language_server_command(
        &mut self,
        _language_server_id: &LanguageServerId,
        worktree: &Worktree,
    ) -> Result<Command, String> {
        let is_windows = zed::current_platform().0 == zed::Os::Windows;
        let binary = worktree
            .which("zedkata-lsp")
            .or_else(|| {
                if is_windows {
                    worktree.which("zedkata-lsp.exe")
                } else {
                    None
                }
            })
            .ok_or_else(|| binary_not_found_message(is_windows))?;

        Ok(build_command(binary))
    }
}

zed::register_extension!(ZedkataExtension);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build_command_preserves_path_with_spaces() {
        let path = "C:\\Program Files\\Zedkata Tools\\zedkata-lsp.exe".to_string();
        let cmd = build_command(path.clone());
        assert_eq!(cmd.command, path);
        assert_eq!(cmd.args, vec!["--stdio".to_string()]);
        assert!(cmd.env.is_empty());
    }

    #[test]
    fn test_binary_not_found_message_content() {
        let win_msg = binary_not_found_message(true);
        assert!(win_msg.contains("zedkata-lsp.exe"));
        assert!(win_msg.contains("cargo install --path crates/zedkata-lsp"));

        let unix_msg = binary_not_found_message(false);
        assert!(unix_msg.contains("zedkata-lsp"));
        assert!(unix_msg.contains("cargo install --path crates/zedkata-lsp"));
    }
}
