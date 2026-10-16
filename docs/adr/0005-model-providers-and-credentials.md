# ADR 0005 — Model providers, credential resolution, and generated data layout

**Status:** Accepted
**Date:** 2026-10-15

## Context

Under [REQ-26](../REQUIREMENTS.md#L64), `zedkata-agent` serves as the conversational pedagogical assistant within Zed's native Agent Panel. It handles:
1. Grounded mistake explanations derived from the student's active edit log in `session.json`.
2. Weakness-targeted practice generation written to disk for catalog consumption ([REQ-28](../REQUIREMENTS.md#L66)).
3. Step-by-step guidance through unobservable editor walkthroughs (Multibuffer, Tasks, Debugger, ACP).

We must establish:
- Which LLM providers and authentication schemes are supported.
- The configuration and credential lookup precedence.
- Bounded context constraints to guarantee token efficiency and eliminate hallucinations.
- The writable storage layout for generated lessons across platforms ([REQ-12](../REQUIREMENTS.md#L50), [REQ-31](../REQUIREMENTS.md#L69), [ZK-006.5](../BACKLOG.md#task-zk-006-5)).

## Decision

### 1. Provider Support Hierarchy

We adopt the proven multi-provider tier from sibling practice projects (`acctg-practice`, `quant-methods-practice`):

1. **Tier 1 — Local / Zero-Cost (Default / Offline):**
   * **LM Studio** via OpenAI-compatible endpoint at `http://localhost:1234/v1`.
   * **Ollama** via OpenAI-compatible endpoint at `http://localhost:11434/v1`.
   * Requires no API keys; operates completely offline with zero usage costs.
2. **Tier 2 — The "Big Three" Cloud Providers (API Keys):**
   * **Anthropic** (`claude-3-5-sonnet`, `claude-3-5-haiku`) via `ANTHROPIC_API_KEY`.
   * **OpenAI** (`gpt-4o`, `gpt-4o-mini`) via `OPENAI_API_KEY`.
   * **Google Gemini** (`gemini-2.0-flash`, `gemini-1.5-pro`) via `GEMINI_API_KEY`.
3. **Tier 3 — Personal Subscription Auth:**
   * **ChatGPT Login / Session Token** option allowing users with existing ChatGPT Plus/Team subscriptions to authenticate without per-token API billing.

### 2. Configuration & Credential Lookup Precedence

Credentials and provider settings are resolved using a 3-layer hierarchy (highest to lowest priority):

1. **Process Environment Variables / Zed Settings:**
   * `ZEDKATA_PROVIDER` (`lmstudio`, `anthropic`, `openai`, `gemini`, `chatgpt-auth`).
   * `ZEDKATA_MODEL` (e.g. `claude-3-5-sonnet-latest`, `gpt-4o-mini`, `local-model`).
   * `ANTHROPIC_API_KEY`, `OPENAI_API_KEY`, `GEMINI_API_KEY`.
   * `LMSTUDIO_BASE_URL` (defaults to `http://localhost:1234/v1`).
   * When launched by Zed, these can be set in `settings.json` under `agent_servers.zedkata.env`.
2. **User Configuration File (`config.toml`):**
   * Windows: `%APPDATA%\zedkata\config.toml`
   * Linux/macOS: `$XDG_CONFIG_HOME/zedkata/config.toml` (fallback `~/.config/zedkata/config.toml`)
   * Holds persistent provider and model defaults.
3. **Automatic Fallback:**
   * If an API key is present in the environment for only one provider, `zedkata-agent` automatically selects that provider.
   * If no keys are present, it defaults to checking LM Studio at `http://localhost:1234/v1`.
   * If no provider is reachable, the agent operates in **degraded mode**: it politely explains that an LLM provider is not configured and provides exact setup steps, while continuing to fulfill non-LLM walkthrough requests.
   * **Security Invariant:** Zero credentials are ever logged to `stderr`, written to `session.json`, or committed to git.

### 3. Bounded Context Contract (REQ-27)

To ensure fast responses, prevent token exhaustion, and eliminate hallucinations:
- Prompts extract only bounded state from `session.json`:
  - Active lesson ID, title, concept, and task goal text.
  - Par edit budget and any active forbidden pattern rules.
  - Rolling edit delta history: capped at the last 20 edit operations or 2 KB of text diff.
  - Specific failure trigger (e.g. `Over par: 5 edits (par 1)` or `Violated forbid rule: backspace_run`).
- The system prompt strictly bounds model behavior: it acts as a focused Vim tutor, analyzes the student's actual edit delta against the target motion, and never fabricates buffer lines outside the provided excerpt.

### 4. Generated Data Layout & Catalog Precedence (REQ-28, REQ-31, ZK-006.5)

Generated practice drills must be stored in writable user data paths outside source checkouts:
* **Development / Source checkout:** `lessons/generated/` relative to repository root.
* **Production Installed Paths:**
  * Windows: `%LOCALAPPDATA%\zedkata\lessons\generated\`
  * Linux/macOS: `$XDG_DATA_HOME/zedkata/lessons/generated/` (fallback `~/.local/share/zedkata/lessons/generated/`)
* **Catalog Precedence:** Shipped core lessons take precedence. Generated lessons are loaded with prefixed IDs (`gen-<topic>-<timestamp>`), marked `draft = false`, and merged into the available curriculum.

### 5. Error Handling & Cancellation

- **Request Cancellation:** When a student cancels a response or navigates away in Zed, Zed sends an ACP cancellation signal. `zedkata-agent` must cancel pending upstream HTTP requests immediately using Tokio `CancellationToken`.
- **Network Failures:** Upstream HTTP timeouts and 4xx/5xx API errors are formatted cleanly into readable pedagogical messages rather than crashing the agent process.

## Consequences

- **Positives:**
  - High accessibility: Students can run zero-cost local models (LM Studio) or leverage existing Big Three keys / ChatGPT subscriptions.
  - Proven architecture: Mirrors successful patterns from `acctg-practice` and `quant-methods-practice`.
  - Robust bounds: Strict context capping prevents costly token runaways and hallucinations.
  - Clean separation: Core drills and TUI remain 100% offline and runtime-agnostic.
- **Trade-offs:**
  - `zedkata-agent` requires HTTP client dependencies (e.g. `reqwest` with rustls) inside its crate, adding build time to the agent binary (isolated from `zedkata-core`).
