use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

/// Supported model providers according to ADR 0005.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ProviderKind {
    /// In-memory/deterministic fake provider for tests and verified offline runs.
    Fake,
    /// Tier 1: Local / zero-cost LM Studio via OpenAI-compatible endpoint (default).
    LmStudio,
    /// Tier 1: Local / zero-cost Ollama via OpenAI-compatible endpoint.
    Ollama,
    /// Tier 2: Anthropic Claude (claude-3-5-sonnet, claude-3-5-haiku).
    Anthropic,
    /// Tier 2: OpenAI (gpt-4o, gpt-4o-mini).
    OpenAI,
    /// Tier 2: Google Gemini (gemini-2.0-flash, gemini-1.5-pro).
    Gemini,
    /// Degraded mode when no LLM provider or credentials are reachable.
    Degraded,
}

impl std::str::FromStr for ProviderKind {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.trim().to_lowercase().as_str() {
            "fake" | "mock" | "test" => Ok(ProviderKind::Fake),
            "lmstudio" | "lm-studio" | "local" => Ok(ProviderKind::LmStudio),
            "ollama" => Ok(ProviderKind::Ollama),
            "anthropic" | "claude" => Ok(ProviderKind::Anthropic),
            "openai" | "chatgpt" => Ok(ProviderKind::OpenAI),
            "gemini" | "google" => Ok(ProviderKind::Gemini),
            "degraded" | "offline" => Ok(ProviderKind::Degraded),
            other => Err(format!("Unknown provider '{}'. Supported: fake, lmstudio, ollama, anthropic, openai, gemini, degraded", other)),
        }
    }
}

impl std::fmt::Display for ProviderKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ProviderKind::Fake => write!(f, "fake"),
            ProviderKind::LmStudio => write!(f, "lmstudio"),
            ProviderKind::Ollama => write!(f, "ollama"),
            ProviderKind::Anthropic => write!(f, "anthropic"),
            ProviderKind::OpenAI => write!(f, "openai"),
            ProviderKind::Gemini => write!(f, "gemini"),
            ProviderKind::Degraded => write!(f, "degraded"),
        }
    }
}

/// Resolved runtime configuration for `zedkata-agent`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AgentConfig {
    /// Active provider choice.
    pub provider: ProviderKind,
    /// Model identifier requested from provider.
    pub model: String,
    /// Base URL for HTTP endpoint (for LM Studio, Ollama, etc.).
    pub base_url: Option<String>,
    /// Secret API key (kept strictly in memory, never logged).
    pub api_key: Option<String>,
    /// Resolved path to `session.json`.
    pub session_path: Option<PathBuf>,
    /// Resolved path to `config.toml`.
    pub config_path: Option<PathBuf>,
    /// Request timeout in seconds for provider calls (default: 15s).
    pub timeout_seconds: u64,
    /// Configured writable directory for generated lesson assets.
    pub generated_dir: Option<PathBuf>,
}

impl Default for AgentConfig {
    fn default() -> Self {
        Self {
            provider: ProviderKind::LmStudio,
            model: "local-model".to_string(),
            base_url: Some("http://localhost:1234/v1".to_string()),
            api_key: None,
            session_path: None,
            config_path: None,
            timeout_seconds: 15,
            generated_dir: None,
        }
    }
}

/// File representation of user `config.toml` per ADR 0005.
#[derive(Debug, Clone, Default, Deserialize)]
pub struct UserConfigFile {
    pub provider: Option<String>,
    pub model: Option<String>,
    pub timeout_seconds: Option<u64>,
    pub generated_dir: Option<String>,
    pub lmstudio: Option<ProviderEndpointConfig>,
    pub ollama: Option<ProviderEndpointConfig>,
    pub anthropic: Option<CloudProviderConfig>,
    pub openai: Option<CloudProviderConfig>,
    pub gemini: Option<CloudProviderConfig>,
}

#[derive(Debug, Clone, Default, Deserialize)]
pub struct ProviderEndpointConfig {
    pub base_url: Option<String>,
    pub model: Option<String>,
}

#[derive(Debug, Clone, Default, Deserialize)]
pub struct CloudProviderConfig {
    pub model: Option<String>,
    pub api_key: Option<String>,
}

impl AgentConfig {
    /// Resolves agent configuration following the 3-layer hierarchy in ADR 0005:
    /// 1. Process environment variables (`ZEDKATA_PROVIDER`, `ZEDKATA_MODEL`, `ANTHROPIC_API_KEY`, etc.)
    /// 2. User configuration file (`config.toml` at standard or override path)
    /// 3. Automatic fallback (single key detection or default offline / LM Studio)
    pub fn resolve() -> Self {
        Self::resolve_with_custom_env_and_file(|k| std::env::var(k).ok(), None)
    }

    /// Resolves configuration with mockable environment lookup and config file path.
    pub fn resolve_with_custom_env_and_file<F>(
        env_lookup: F,
        explicit_config_path: Option<PathBuf>,
    ) -> Self
    where
        F: Fn(&str) -> Option<String>,
    {
        let config_path = explicit_config_path
            .or_else(|| env_lookup("ZEDKATA_CONFIG_PATH").map(PathBuf::from))
            .or_else(default_config_path);

        let file_config = config_path
            .as_ref()
            .and_then(|path| read_user_config_file(path));

        // 1. Resolve Provider
        let env_provider =
            env_lookup("ZEDKATA_PROVIDER").and_then(|s| s.parse::<ProviderKind>().ok());

        let file_provider = file_config
            .as_ref()
            .and_then(|c| c.provider.as_deref())
            .and_then(|s| s.parse::<ProviderKind>().ok());

        // Automatic fallback if no provider explicitly declared
        let resolved_provider = if let Some(p) = env_provider {
            p
        } else if let Some(p) = file_provider {
            p
        } else {
            // Check single API key presence
            let has_anthropic = env_lookup("ANTHROPIC_API_KEY").is_some()
                || file_config
                    .as_ref()
                    .and_then(|c| c.anthropic.as_ref())
                    .and_then(|a| a.api_key.as_ref())
                    .is_some();
            let has_openai = env_lookup("OPENAI_API_KEY").is_some()
                || file_config
                    .as_ref()
                    .and_then(|c| c.openai.as_ref())
                    .and_then(|a| a.api_key.as_ref())
                    .is_some();
            let has_gemini = env_lookup("GEMINI_API_KEY").is_some()
                || file_config
                    .as_ref()
                    .and_then(|c| c.gemini.as_ref())
                    .and_then(|a| a.api_key.as_ref())
                    .is_some();

            match (has_anthropic, has_openai, has_gemini) {
                (true, false, false) => ProviderKind::Anthropic,
                (false, true, false) => ProviderKind::OpenAI,
                (false, false, true) => ProviderKind::Gemini,
                _ => ProviderKind::LmStudio,
            }
        };

        // 2. Resolve Model
        let default_model = match resolved_provider {
            ProviderKind::Fake => "fake-model",
            ProviderKind::LmStudio => "local-model",
            ProviderKind::Ollama => "mistral",
            ProviderKind::Anthropic => "claude-3-5-sonnet-latest",
            ProviderKind::OpenAI => "gpt-4o-mini",
            ProviderKind::Gemini => "gemini-2.0-flash",
            ProviderKind::Degraded => "none",
        };

        let env_model = env_lookup("ZEDKATA_MODEL");
        let file_model = file_config
            .as_ref()
            .and_then(|c| match resolved_provider {
                ProviderKind::Anthropic => c.anthropic.as_ref().and_then(|p| p.model.clone()),
                ProviderKind::OpenAI => c.openai.as_ref().and_then(|p| p.model.clone()),
                ProviderKind::Gemini => c.gemini.as_ref().and_then(|p| p.model.clone()),
                ProviderKind::LmStudio => c.lmstudio.as_ref().and_then(|p| p.model.clone()),
                ProviderKind::Ollama => c.ollama.as_ref().and_then(|p| p.model.clone()),
                _ => c.model.clone(),
            })
            .or_else(|| file_config.as_ref().and_then(|c| c.model.clone()));

        let resolved_model = env_model
            .or(file_model)
            .unwrap_or_else(|| default_model.to_string());

        // 3. Resolve Base URL
        let resolved_base_url = match resolved_provider {
            ProviderKind::LmStudio => env_lookup("LMSTUDIO_BASE_URL")
                .or_else(|| {
                    file_config
                        .as_ref()
                        .and_then(|c| c.lmstudio.as_ref())
                        .and_then(|p| p.base_url.clone())
                })
                .or_else(|| Some("http://localhost:1234/v1".to_string())),
            ProviderKind::Ollama => env_lookup("OLLAMA_BASE_URL")
                .or_else(|| {
                    file_config
                        .as_ref()
                        .and_then(|c| c.ollama.as_ref())
                        .and_then(|p| p.base_url.clone())
                })
                .or_else(|| Some("http://localhost:11434/v1".to_string())),
            _ => None,
        };

        // 4. Resolve API Key
        let resolved_api_key = match resolved_provider {
            ProviderKind::Anthropic => env_lookup("ANTHROPIC_API_KEY").or_else(|| {
                file_config
                    .as_ref()
                    .and_then(|c| c.anthropic.as_ref())
                    .and_then(|p| p.api_key.clone())
            }),
            ProviderKind::OpenAI => env_lookup("OPENAI_API_KEY").or_else(|| {
                file_config
                    .as_ref()
                    .and_then(|c| c.openai.as_ref())
                    .and_then(|p| p.api_key.clone())
            }),
            ProviderKind::Gemini => env_lookup("GEMINI_API_KEY").or_else(|| {
                file_config
                    .as_ref()
                    .and_then(|c| c.gemini.as_ref())
                    .and_then(|p| p.api_key.clone())
            }),
            _ => None,
        };

        // 5. Resolve Session Path
        let session_path = env_lookup("ZEDKATA_SESSION_PATH")
            .or_else(|| env_lookup("ZEDKATA_SESSION"))
            .map(PathBuf::from)
            .or_else(|| zedkata_core::session::default_session_path().ok());

        // 6. Resolve Timeout
        let env_timeout = env_lookup("ZEDKATA_TIMEOUT_SECONDS").and_then(|s| s.parse::<u64>().ok());
        let file_timeout = file_config.as_ref().and_then(|c| c.timeout_seconds);
        let timeout_seconds = env_timeout.or(file_timeout).unwrap_or(15);

        // 7. Resolve Generated Directory
        let env_generated_dir = env_lookup("ZEDKATA_GENERATED_DIR")
            .or_else(|| {
                env_lookup("ZEDKATA_DATA_DIR").map(|d| {
                    PathBuf::from(d)
                        .join("lessons")
                        .join("generated")
                        .to_string_lossy()
                        .to_string()
                })
            })
            .map(PathBuf::from);
        let file_generated_dir = file_config
            .as_ref()
            .and_then(|c| c.generated_dir.as_ref())
            .map(PathBuf::from);
        let generated_dir = env_generated_dir
            .or(file_generated_dir)
            .or_else(|| zedkata_core::catalog::default_generated_dir().ok());

        AgentConfig {
            provider: resolved_provider,
            model: resolved_model,
            base_url: resolved_base_url,
            api_key: resolved_api_key,
            session_path,
            config_path,
            timeout_seconds,
            generated_dir,
        }
    }

    /// Returns the resolved generated lessons directory, falling back to platform standard or `lessons/generated`.
    pub fn resolved_generated_dir(&self) -> PathBuf {
        self.generated_dir
            .clone()
            .or_else(|| zedkata_core::catalog::default_generated_dir().ok())
            .unwrap_or_else(|| PathBuf::from("lessons").join("generated"))
    }
}

/// Resolves standard platform path for user `config.toml`.
/// - Windows: `%APPDATA%\zedkata\config.toml`
/// - Linux/macOS: `$XDG_CONFIG_HOME/zedkata/config.toml` (fallback `~/.config/zedkata/config.toml`)
pub fn default_config_path() -> Option<PathBuf> {
    #[cfg(target_os = "windows")]
    {
        if let Ok(app_data) = std::env::var("APPDATA") {
            return Some(PathBuf::from(app_data).join("zedkata").join("config.toml"));
        }
    }

    #[cfg(not(target_os = "windows"))]
    {
        if let Ok(xdg) = std::env::var("XDG_CONFIG_HOME") {
            return Some(PathBuf::from(xdg).join("zedkata").join("config.toml"));
        }
        if let Ok(home) = std::env::var("HOME") {
            return Some(
                PathBuf::from(home)
                    .join(".config")
                    .join("zedkata")
                    .join("config.toml"),
            );
        }
    }

    None
}

/// Reads and parses user config file from disk safely, ignoring errors.
fn read_user_config_file(path: &Path) -> Option<UserConfigFile> {
    let content = std::fs::read_to_string(path).ok()?;
    toml::from_str(&content).ok()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    #[test]
    fn test_provider_from_str() {
        assert_eq!("fake".parse::<ProviderKind>().unwrap(), ProviderKind::Fake);
        assert_eq!(
            "lmstudio".parse::<ProviderKind>().unwrap(),
            ProviderKind::LmStudio
        );
        assert_eq!(
            "local".parse::<ProviderKind>().unwrap(),
            ProviderKind::LmStudio
        );
        assert_eq!(
            "ollama".parse::<ProviderKind>().unwrap(),
            ProviderKind::Ollama
        );
        assert_eq!(
            "anthropic".parse::<ProviderKind>().unwrap(),
            ProviderKind::Anthropic
        );
        assert_eq!(
            "claude".parse::<ProviderKind>().unwrap(),
            ProviderKind::Anthropic
        );
        assert_eq!(
            "openai".parse::<ProviderKind>().unwrap(),
            ProviderKind::OpenAI
        );
        assert_eq!(
            "gemini".parse::<ProviderKind>().unwrap(),
            ProviderKind::Gemini
        );
        assert_eq!(
            "degraded".parse::<ProviderKind>().unwrap(),
            ProviderKind::Degraded
        );
        assert!("unknown".parse::<ProviderKind>().is_err());
    }

    #[test]
    fn test_env_precedence_over_config_file() {
        let mut env_map = HashMap::new();
        env_map.insert("ZEDKATA_PROVIDER".to_string(), "anthropic".to_string());
        env_map.insert(
            "ZEDKATA_MODEL".to_string(),
            "claude-3-5-haiku-custom".to_string(),
        );
        env_map.insert(
            "ANTHROPIC_API_KEY".to_string(),
            "sk-ant-test123".to_string(),
        );

        let config =
            AgentConfig::resolve_with_custom_env_and_file(|k| env_map.get(k).cloned(), None);

        assert_eq!(config.provider, ProviderKind::Anthropic);
        assert_eq!(config.model, "claude-3-5-haiku-custom");
        assert_eq!(config.api_key.as_deref(), Some("sk-ant-test123"));
    }

    #[test]
    fn test_automatic_key_detection_fallback() {
        let mut env_map = HashMap::new();
        env_map.insert("OPENAI_API_KEY".to_string(), "sk-openai-test".to_string());

        let config =
            AgentConfig::resolve_with_custom_env_and_file(|k| env_map.get(k).cloned(), None);

        assert_eq!(config.provider, ProviderKind::OpenAI);
        assert_eq!(config.model, "gpt-4o-mini");
        assert_eq!(config.api_key.as_deref(), Some("sk-openai-test"));
    }

    #[test]
    fn test_default_fallback_to_lmstudio() {
        let env_map: HashMap<String, String> = HashMap::new();
        let config =
            AgentConfig::resolve_with_custom_env_and_file(|k| env_map.get(k).cloned(), None);

        assert_eq!(config.provider, ProviderKind::LmStudio);
        assert_eq!(config.model, "local-model");
        assert_eq!(config.base_url.as_deref(), Some("http://localhost:1234/v1"));
        assert!(config.api_key.is_none());
    }
}
