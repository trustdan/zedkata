pub mod config;
pub mod context;
pub mod generator;
pub mod logging;
pub mod prompt;
pub mod provider;
pub mod server;
pub mod storage;
pub mod walkthrough;

pub use config::{default_config_path, AgentConfig, ProviderKind};
pub use context::{
    extract_from_session_state, extract_last_attempt_context, extract_weakness_context,
    AttemptContext, AttemptVerdictContext, ContextConfig, ContextError, ForbidRulesContext,
    MAX_DIFF_BYTES, MAX_EDIT_OPERATIONS,
};
pub use generator::{
    build_candidate_user_prompt, parse_candidate_response, synthesize_default_candidate,
    system_generator_prompt, CandidateExercise, CandidateStatus, WeaknessContext,
    DEFAULT_FALLBACK_CONCEPT,
};
pub use logging::init_stderr_logging;
pub use prompt::{build_attempt_user_prompt, system_prompt};
pub use provider::{
    AttemptExplainer, ConfiguredProvider, ExerciseGenerator, Explanation, FakeProvider,
    ProviderError,
};
pub use server::{AgentServer, SessionState};
pub use storage::{
    store_candidate_assets, validate_candidate, verify_path_within_root, GeneratedAssetError,
    StoredAssetPaths,
};
pub use walkthrough::{
    format_walkthrough_list, mark_completed_in_session, reset_in_session, WalkthroughError,
    WalkthroughRecord,
};
