# ZK-029.2 — Model Provider, Credentials & Generated Data Evidence

- **Task:** [ZK-029.2](../BACKLOG.md#task-zk-029-2) (Choose provider, credentials and generated-data configuration)
- **Date:** 2026-10-15
- **Author:** Zedkata Team
- **Status:** Verified

---

## 1. Executive Summary

This specification establishes the model provider architecture, authentication lookup hierarchy, bounded context protocol, and persistent storage layout for `zedkata-agent`. 

In alignment with user preferences proven in sibling projects (`acctg-practice`, `quant-methods-practice`), `zedkata-agent` supports:
1. **Local / Zero-Cost:** LM Studio (default) and Ollama via OpenAI-compatible endpoints (`http://localhost:1234/v1`).
2. **Big Three Cloud Providers:** Anthropic Claude (`ANTHROPIC_API_KEY`), OpenAI GPT (`OPENAI_API_KEY`), and Google Gemini (`GEMINI_API_KEY`).
3. **Personal Subscription Login:** ChatGPT subscription / session token auth option.
4. **Isolated Storage Layout:** Clean separation between development checkout paths and production platform user-data paths.

---

## 2. Configuration & Credential Schema

### User Configuration File (`config.toml`)
Default locations:
- Windows: `%APPDATA%\zedkata\config.toml`
- Linux/macOS: `$XDG_CONFIG_HOME/zedkata/config.toml` (fallback `~/.config/zedkata/config.toml`)

```toml
# Default provider: "lmstudio" | "anthropic" | "openai" | "gemini" | "chatgpt-auth"
provider = "lmstudio"
model = "local-model"

[lmstudio]
base_url = "http://localhost:1234/v1"

[anthropic]
model = "claude-3-5-sonnet-latest"
# api_key can be omitted if ANTHROPIC_API_KEY environment variable is set

[openai]
model = "gpt-4o-mini"
# api_key can be omitted if OPENAI_API_KEY environment variable is set

[gemini]
model = "gemini-2.0-flash"
# api_key can be omitted if GEMINI_API_KEY environment variable is set

[chatgpt_auth]
# Session token / OAuth device profile
```

### Zed `settings.json` Integration
Users can supply environment overrides directly in Zed's configuration:
```json
{
  "agent_servers": {
    "zedkata": {
      "type": "custom",
      "command": "zedkata-agent",
      "args": ["--acp"],
      "env": {
        "ZEDKATA_PROVIDER": "anthropic",
        "ANTHROPIC_API_KEY": "sk-ant-..."
      }
    }
  }
}
```

---

## 3. Bounded Context Contract (REQ-27)

When explaining mistakes, `zedkata-agent` reads `$XDG_STATE_HOME/zedkata/session.json` and constructs a bounded prompt:

```json
{
  "context": {
    "lesson_id": "operators-01-edit",
    "exercise_index": 1,
    "concept": "character-replacement",
    "task_goal": "Replace 'bad' with 'good' using 'r'",
    "par_edits": 1,
    "active_forbid_rules": ["backspace_run"],
    "attempt_verdict": "failed",
    "failure_reason": "Over par: 5 edits (par 1)",
    "recent_edits": [
      {
        "deleted": "bad",
        "inserted": "good",
        "method": "5 backspaces followed by typing"
      }
    ]
  }
}
```

### Invariants:
1. **Edit Boundary:** History is capped at the last 20 edit operations or 2KB of text diff.
2. **Grounding:** The prompt forbids fabricating editor state, cursor coordinates, or alternative file buffers.
3. **No Hallucinated Tools:** The agent answers pedagogical questions directly or generates lessons; it does not claim to execute code or manipulate editor windows outside ACP capabilities.

---

## 4. Writable Generated Data Layout (REQ-12, REQ-28, REQ-31, ZK-006.5)

To prevent cluttering the repository during installed execution, generated practice lessons are written to standard OS user data locations:

| OS | Generated Lessons Directory |
|---|---|
| **Development** | `lessons/generated/` (repo root) |
| **Windows** | `%LOCALAPPDATA%\zedkata\lessons\generated\` |
| **Linux** | `$XDG_DATA_HOME/zedkata/lessons/generated/` (`~/.local/share/zedkata/lessons/generated/`) |
| **macOS** | `~/Library/Application Support/zedkata/lessons/generated/` |

### File Naming & Catalog Inclusion
- Each generated drill produces a TOML lesson file: `gen-<concept>-<timestamp>.toml`.
- Accompanying starting and goal fixtures are stored in a sibling `fixtures/` subdirectory.
- The catalog loader indexes core shipped lessons first, followed by valid files found in the user generated directory.

---

## 5. Security & Verification Checklist

- [x] **No hardcoded secrets:** Zero API keys or tokens are stored in the codebase or git.
- [x] **Isolated network boundary:** Only `crates/zedkata-agent` performs HTTP calls; `zedkata-core`, `zedkata-lsp`, and `zedkata-tui` remain strictly offline.
- [x] **Graceful degradation:** If no API key or local LM Studio server is present, the agent informs the user with setup instructions without crashing.
- [x] **Request cancellation:** Upstream HTTP calls bind to Tokio cancellation tokens driven by ACP client signals.
