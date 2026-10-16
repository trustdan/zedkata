use anyhow::Result;
use clap::{Parser, Subcommand};
use std::path::PathBuf;
use zedkata_agent::{
    init_stderr_logging, AgentConfig, AgentServer, AttemptExplainer, ConfiguredProvider,
    ContextConfig, ProviderKind,
};

#[derive(Parser, Debug)]
#[command(
    name = "zedkata-agent",
    about = "Zedkata ACP Agent for personalized explanations and drills in Zed",
    version
)]
struct Cli {
    /// Speak Agent Client Protocol over stdin/stdout
    #[arg(long)]
    acp: bool,

    /// Explain the last attempt from session.json
    #[arg(long)]
    explain: bool,

    /// Generate a candidate practice exercise targeting weaknesses
    #[arg(long)]
    generate: bool,

    /// Validate and store the generated exercise in the user data directory
    #[arg(long)]
    store: bool,

    /// Optional target output directory for storing generated exercises
    #[arg(long)]
    out_dir: Option<PathBuf>,

    /// Optional target concept for practice generation (defaults to highest weakness or baseline)
    #[arg(long)]
    concept: Option<String>,

    /// Optional explicit path to session.json
    #[arg(long)]
    session: Option<PathBuf>,

    /// Optional explicit provider choice (fake, lmstudio, ollama, anthropic, openai, gemini, degraded)
    #[arg(long)]
    provider: Option<String>,

    /// List all guided walkthroughs
    #[arg(long)]
    walkthrough_list: bool,

    /// Show verified steps for a walkthrough
    #[arg(long)]
    walkthrough_show: Option<String>,

    /// Mark self-reported completion for a walkthrough
    #[arg(long)]
    walkthrough_complete: Option<String>,

    /// Reset self-reported completion for a walkthrough
    #[arg(long)]
    walkthrough_reset: Option<String>,

    /// Optional subcommand
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Explain the last attempt from session.json using the configured provider
    Explain {
        /// Optional explicit path to session.json
        #[arg(long)]
        session: Option<PathBuf>,

        /// Optional explicit provider choice
        #[arg(long)]
        provider: Option<String>,
    },

    /// Generate a candidate practice exercise targeting weaknesses from session.json
    Generate {
        /// Optional target concept (e.g. "dw", "ci", "word-motions")
        #[arg(long)]
        concept: Option<String>,

        /// Validate and store the generated exercise in the user data directory
        #[arg(long)]
        store: bool,

        /// Optional target output directory for storing generated exercises
        #[arg(long)]
        out_dir: Option<PathBuf>,

        /// Optional explicit path to session.json
        #[arg(long)]
        session: Option<PathBuf>,

        /// Optional explicit provider choice
        #[arg(long)]
        provider: Option<String>,
    },

    /// Select, inspect, and track guided walkthroughs (unscored per REQ-25 / REQ-29)
    Walkthrough {
        #[command(subcommand)]
        action: Option<WalkthroughAction>,

        /// Walkthrough ID to show (if no subcommand given)
        #[arg(long)]
        id: Option<String>,

        /// List all guided walkthroughs and progress
        #[arg(long)]
        list: bool,

        /// Mark explicit self-reported completion in session.json (REQ-29)
        #[arg(long)]
        complete: Option<String>,

        /// Reset self-reported completion status in session.json
        #[arg(long)]
        reset: Option<String>,

        /// Optional explicit path to session.json
        #[arg(long)]
        session: Option<PathBuf>,
    },
}

#[derive(Subcommand, Debug)]
enum WalkthroughAction {
    /// List all guided walkthroughs and student progress
    List {
        /// Optional explicit path to session.json
        #[arg(long)]
        session: Option<PathBuf>,
    },

    /// Show verified steps, fixture workspaces, and self-report checklist
    Show {
        /// Walkthrough ID (e.g. "mini-vs-any", "lsp-actions-walkthrough")
        id: String,

        /// Optional explicit path to session.json
        #[arg(long)]
        session: Option<PathBuf>,
    },

    /// Mark explicit self-reported completion in session.json (REQ-29)
    Complete {
        /// Walkthrough ID
        id: String,

        /// Optional explicit path to session.json
        #[arg(long)]
        session: Option<PathBuf>,
    },

    /// Reset self-reported completion status in session.json
    Reset {
        /// Walkthrough ID
        id: String,

        /// Optional explicit path to session.json
        #[arg(long)]
        session: Option<PathBuf>,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    if cli.acp {
        // Initialize logging to write strictly to stderr to preserve stdout framing hygiene.
        init_stderr_logging();
        eprintln!(
            "[zedkata-agent] Starting ACP server (version {}) on stdio",
            env!("CARGO_PKG_VERSION")
        );

        let server = AgentServer::new();
        server.run_stdio().await?;
        return Ok(());
    }

    let is_generate = cli.generate || matches!(cli.command, Some(Commands::Generate { .. }));
    if is_generate {
        let (cli_concept, cli_session, cli_provider, should_store, cli_out_dir) = match cli.command
        {
            Some(Commands::Generate {
                concept,
                store,
                out_dir,
                session,
                provider,
            }) => (
                concept.or(cli.concept),
                session.or(cli.session),
                provider.or(cli.provider),
                store || cli.store,
                out_dir.or(cli.out_dir),
            ),
            _ => (
                cli.concept,
                cli.session,
                cli.provider,
                cli.store,
                cli.out_dir,
            ),
        };

        let mut config = AgentConfig::resolve();
        if let Some(p) = cli_provider {
            if let Ok(kind) = p.parse::<ProviderKind>() {
                config.provider = kind;
            } else {
                eprintln!(
                    "[zedkata-agent] Warning: Unknown provider '{}', using default '{}'.",
                    p, config.provider
                );
            }
        }
        if let Some(s) = cli_session {
            config.session_path = Some(s);
        }

        let session_path = config
            .session_path
            .clone()
            .or_else(|| zedkata_core::session::default_session_path().ok())
            .unwrap_or_else(|| PathBuf::from("session.json"));

        let mut weakness_ctx =
            zedkata_agent::extract_weakness_context(&session_path, cli_concept.as_deref());

        if let Some(c) = cli_concept {
            weakness_ctx.target_concept = c;
        }

        let provider = ConfiguredProvider::new(config.clone());
        match provider.generate_candidate_async(&weakness_ctx, None).await {
            Ok(mut candidate) => {
                println!("{}", candidate.format_preview());
                eprintln!("\nCandidate Lesson TOML:\n{}", candidate.raw_toml);

                if should_store {
                    let out_dir = cli_out_dir.unwrap_or_else(|| config.resolved_generated_dir());
                    match candidate.validate_and_store(&out_dir) {
                        Ok(paths) => {
                            eprintln!(
                                "\n[zedkata-agent] Validated and stored generated exercise in '{}':",
                                out_dir.display()
                            );
                            eprintln!("  TOML:    {}", paths.toml_path.display());
                            eprintln!("  Start:   {}", paths.start_fixture_path.display());
                            eprintln!("  Goal:    {}", paths.goal_fixture_path.display());
                        }
                        Err(e) => {
                            eprintln!(
                                "\n[zedkata-agent] Failed to store generated exercise: {}",
                                e
                            );
                            std::process::exit(1);
                        }
                    }
                }
            }
            Err(e) => {
                eprintln!("{}", e.to_user_message());
                std::process::exit(1);
            }
        }
        return Ok(());
    }

    let is_explain = cli.explain || matches!(cli.command, Some(Commands::Explain { .. }));
    if is_explain {
        let (cli_session, cli_provider) = match cli.command {
            Some(Commands::Explain { session, provider }) => {
                (session.or(cli.session), provider.or(cli.provider))
            }
            _ => (cli.session, cli.provider),
        };

        let mut config = AgentConfig::resolve();
        if let Some(p) = cli_provider {
            if let Ok(kind) = p.parse::<ProviderKind>() {
                config.provider = kind;
            } else {
                eprintln!(
                    "[zedkata-agent] Warning: Unknown provider '{}', using default '{}'.",
                    p, config.provider
                );
            }
        }
        if let Some(s) = cli_session {
            config.session_path = Some(s);
        }

        let session_path = config
            .session_path
            .clone()
            .or_else(|| zedkata_core::session::default_session_path().ok())
            .ok_or_else(|| anyhow::anyhow!("Could not resolve platform-standard session path."))?;

        let provider = ConfiguredProvider::new(config);
        match provider.explain_session(&session_path, None, &ContextConfig::default()) {
            Ok(explanation) => {
                println!("{}", explanation.to_markdown());
            }
            Err(e) => {
                eprintln!("{}", e.to_user_message());
                std::process::exit(1);
            }
        }
        return Ok(());
    }

    let is_walkthrough = cli.walkthrough_list
        || cli.walkthrough_show.is_some()
        || cli.walkthrough_complete.is_some()
        || cli.walkthrough_reset.is_some()
        || matches!(cli.command, Some(Commands::Walkthrough { .. }));

    if is_walkthrough {
        let (action, cli_session) = match cli.command {
            Some(Commands::Walkthrough {
                action,
                id,
                list,
                complete,
                reset,
                session,
            }) => {
                let (resolved_action, inner_session) = match action {
                    Some(WalkthroughAction::List { session: s }) => {
                        (Some(WalkthroughAction::List { session: None }), s)
                    }
                    Some(WalkthroughAction::Show {
                        id: act_id,
                        session: s,
                    }) => (
                        Some(WalkthroughAction::Show {
                            id: act_id,
                            session: None,
                        }),
                        s,
                    ),
                    Some(WalkthroughAction::Complete {
                        id: act_id,
                        session: s,
                    }) => (
                        Some(WalkthroughAction::Complete {
                            id: act_id,
                            session: None,
                        }),
                        s,
                    ),
                    Some(WalkthroughAction::Reset {
                        id: act_id,
                        session: s,
                    }) => (
                        Some(WalkthroughAction::Reset {
                            id: act_id,
                            session: None,
                        }),
                        s,
                    ),
                    None => {
                        if list || cli.walkthrough_list {
                            (Some(WalkthroughAction::List { session: None }), None)
                        } else if let Some(show_id) = id.or(cli.walkthrough_show) {
                            (
                                Some(WalkthroughAction::Show {
                                    id: show_id,
                                    session: None,
                                }),
                                None,
                            )
                        } else if let Some(comp_id) = complete.or(cli.walkthrough_complete) {
                            (
                                Some(WalkthroughAction::Complete {
                                    id: comp_id,
                                    session: None,
                                }),
                                None,
                            )
                        } else if let Some(res_id) = reset.or(cli.walkthrough_reset) {
                            (
                                Some(WalkthroughAction::Reset {
                                    id: res_id,
                                    session: None,
                                }),
                                None,
                            )
                        } else {
                            (Some(WalkthroughAction::List { session: None }), None)
                        }
                    }
                };
                (resolved_action, inner_session.or(session).or(cli.session))
            }
            _ => {
                let resolved_action = if cli.walkthrough_list {
                    Some(WalkthroughAction::List { session: None })
                } else if let Some(show_id) = cli.walkthrough_show {
                    Some(WalkthroughAction::Show {
                        id: show_id,
                        session: None,
                    })
                } else if let Some(comp_id) = cli.walkthrough_complete {
                    Some(WalkthroughAction::Complete {
                        id: comp_id,
                        session: None,
                    })
                } else if let Some(res_id) = cli.walkthrough_reset {
                    Some(WalkthroughAction::Reset {
                        id: res_id,
                        session: None,
                    })
                } else {
                    Some(WalkthroughAction::List { session: None })
                };
                (resolved_action, cli.session)
            }
        };

        let session_path = cli_session
            .or_else(|| zedkata_core::session::default_session_path().ok())
            .unwrap_or_else(|| PathBuf::from("session.json"));

        match action {
            Some(WalkthroughAction::List { .. }) => {
                let session = zedkata_core::session::SessionState::load_from_file(&session_path)
                    .unwrap_or_default();
                let output = zedkata_agent::format_walkthrough_list(
                    &zedkata_agent::WalkthroughRecord::all(),
                    &session,
                );
                print!("{}", output);
            }
            Some(WalkthroughAction::Show { id, .. }) => {
                let record = zedkata_agent::WalkthroughRecord::find(&id).ok_or_else(|| {
                    anyhow::anyhow!(
                        "Walkthrough '{}' not found. Run 'zedkata-agent walkthrough list' for available walkthroughs.",
                        id
                    )
                })?;
                let session = zedkata_core::session::SessionState::load_from_file(&session_path)
                    .unwrap_or_default();
                let is_completed = session.is_walkthrough_completed(&record.id);
                println!("{}", record.format_detail(is_completed));
            }
            Some(WalkthroughAction::Complete { id, .. }) => {
                match zedkata_agent::mark_completed_in_session(&session_path, &id) {
                    Ok((record, newly_completed)) => {
                        let status_note = if newly_completed {
                            "Recorded new self-reported completion."
                        } else {
                            "Refreshed existing completion record."
                        };
                        println!(
                            "Marked walkthrough '{}' ({}) as COMPLETED.\nState updated in '{}': {}",
                            record.id,
                            record.title,
                            session_path.display(),
                            status_note
                        );
                    }
                    Err(e) => {
                        eprintln!("[zedkata-agent] Error completing walkthrough: {}", e);
                        std::process::exit(1);
                    }
                }
            }
            Some(WalkthroughAction::Reset { id, .. }) => {
                match zedkata_agent::reset_in_session(&session_path, &id) {
                    Ok((record, removed)) => {
                        let status_note = if removed {
                            "Completion mark cleared."
                        } else {
                            "Walkthrough was not previously marked complete."
                        };
                        println!(
                            "Reset walkthrough '{}' ({}) completion status.\nState updated in '{}': {}",
                            record.id,
                            record.title,
                            session_path.display(),
                            status_note
                        );
                    }
                    Err(e) => {
                        eprintln!("[zedkata-agent] Error resetting walkthrough: {}", e);
                        std::process::exit(1);
                    }
                }
            }
            None => {
                let session = zedkata_core::session::SessionState::load_from_file(&session_path)
                    .unwrap_or_default();
                let output = zedkata_agent::format_walkthrough_list(
                    &zedkata_agent::WalkthroughRecord::all(),
                    &session,
                );
                print!("{}", output);
            }
        }
        return Ok(());
    }

    println!("zedkata-agent {}", env!("CARGO_PKG_VERSION"));
    println!("Use --acp to launch the Agent Client Protocol server over stdio.");
    println!("Use 'explain' to explain the last attempt from session.json.");
    println!("Use 'generate' to generate a targeted practice drill candidate.");
    println!("Use 'walkthrough' to select, inspect, and track guided walkthroughs.");
    Ok(())
}
