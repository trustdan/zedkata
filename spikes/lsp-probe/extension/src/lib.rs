use zed_extension_api::{self as zed, Command, LanguageServerId, Worktree};

struct ProbeExtension;

impl zed::Extension for ProbeExtension {
    fn new() -> Self {
        Self
    }

    fn language_server_command(
        &mut self,
        _language_server_id: &LanguageServerId,
        worktree: &Worktree,
    ) -> Result<Command, String> {
        worktree
            .read_text_file("spikes/lsp-probe/package.json")
            .map_err(|_| {
                "Open the zedkata repository root in Zed to run this probe.".to_string()
            })?;
        let node = worktree
            .which("node")
            .ok_or("Node.js is not on Zed's PATH. Install Node 22+ and restart Zed.")?;
        let root = worktree.root_path();
        Ok(Command {
            command: node,
            args: vec![
                format!("{root}/spikes/lsp-probe/launch.cjs"),
                "--stdio".into(),
            ],
            env: vec![],
        })
    }
}

zed::register_extension!(ProbeExtension);
