mod commands;
mod config;
mod error;
mod output;

use anyhow::Result;
use clap::{Parser, Subcommand};
use commands::*;
use error::{handle_error, EXIT_SUCCESS};
use output::OutputMode;
use std::process::ExitCode;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[derive(Parser)]
#[command(name = "vllama")]
#[command(about = "High-performance local LLM inference server", long_about = None)]
#[command(version)]
struct Cli {
    #[command(subcommand)]
    command: Commands,

    #[arg(long, global = true, help = "Enable verbose logging")]
    verbose: bool,

    #[arg(long, global = true, help = "Minimal output")]
    quiet: bool,

    #[arg(long, global = true, help = "JSON output for scripting")]
    json: bool,
}

#[derive(Subcommand)]
enum Commands {
    #[command(about = "Start the vLLama server")]
    Serve {
        #[arg(long, default_value = "127.0.0.1", help = "Server host address")]
        host: String,

        #[arg(short, long, default_value = "11434", help = "Server port")]
        port: u16,

        #[arg(long, help = "Model to load in vLLM (e.g., meta-llama/Llama-3.2-1B-Instruct)")]
        model: Option<String>,

        #[arg(long, default_value = "8100", help = "vLLM OpenAI server port")]
        vllm_port: u16,

        #[arg(long, help = "Skip auto-starting vLLM server (use existing instance)")]
        no_vllm: bool,

        #[arg(long, default_value = "256", help = "vLLM max concurrent sequences")]
        max_num_seqs: usize,

        #[arg(long, default_value = "0.9", help = "vLLM GPU memory utilization (0.0-1.0)")]
        gpu_memory_utilization: f32,
    },

    #[command(about = "Run a model and chat interactively")]
    Run {
        #[arg(help = "Model name to run")]
        model: String,

        #[arg(help = "Optional prompt to send")]
        prompt: Option<String>,
    },

    #[command(about = "Generate text from a model")]
    Generate {
        #[arg(help = "Model name")]
        model: String,

        #[arg(help = "Prompt text")]
        prompt: String,

        #[arg(long, help = "Stream the response")]
        stream: bool,
    },

    #[command(about = "List locally available models")]
    List,

    #[command(about = "Download a model from a registry")]
    Pull {
        #[arg(help = "Model name to download")]
        model: String,
    },

    #[command(about = "Remove a local model")]
    Rm {
        #[arg(help = "Model name to remove")]
        model: String,
    },

    #[command(about = "Show information about a model")]
    Show {
        #[arg(help = "Model name")]
        model: String,

        #[arg(long, help = "Show modelfile")]
        modelfile: bool,

        #[arg(long, help = "Show parameters")]
        parameters: bool,
    },

    #[command(about = "List currently running models")]
    Ps,

    #[command(about = "Show system hardware information")]
    Info,

    #[command(about = "Benchmark inference engine performance (experimental)")]
    Bench {
        #[arg(help = "Model name")]
        model: String,

        #[arg(help = "Prompt for benchmarking", default_value = "Once upon a time")]
        prompt: String,

        #[arg(short, long, help = "Number of requests", default_value = "10")]
        iterations: usize,

        #[arg(short, long, help = "Concurrent requests (1 = sequential)", default_value = "1")]
        concurrency: usize,
    },

    #[command(about = "Generate example configuration file")]
    Config {
        #[arg(long, help = "Show current configuration")]
        show: bool,
    },
}

#[tokio::main]
async fn main() -> ExitCode {
    let cli = Cli::parse();

    // Load configuration files
    let config = match config::Config::load() {
        Ok(c) => c,
        Err(e) => {
            eprintln!("Failed to load configuration: {}", e);
            return ExitCode::from(error::EXIT_ERROR);
        }
    };

    init_tracing(cli.verbose || config.logging.level == "debug");

    // Determine output mode (CLI flags override config)
    let output_mode = if cli.json || config.output.json {
        OutputMode::Json
    } else if cli.quiet || config.output.quiet {
        OutputMode::Quiet
    } else {
        OutputMode::Normal
    };

    // Run command and handle errors gracefully
    if let Err(err) = run_command(cli.command, output_mode, config).await {
        let user_error = handle_error(err);
        eprintln!("{}", user_error);
        return user_error.exit_code();
    }

    ExitCode::from(EXIT_SUCCESS)
}

async fn run_command(command: Commands, output_mode: OutputMode, config: config::Config) -> Result<()> {
    match command {
        Commands::Serve {
            host,
            port,
            model,
            vllm_port,
            no_vllm,
            max_num_seqs,
            gpu_memory_utilization,
        } => {
            // Apply config defaults when CLI flags not provided
            let host = if host == "127.0.0.1" { config.server.host } else { host };
            let port = if port == 11434 { config.server.port } else { port };
            let vllm_port = if vllm_port == 8100 { config.server.vllm_port } else { vllm_port };
            let model = model.or(config.model.default_model);
            let max_num_seqs = if max_num_seqs == 256 { config.model.max_num_seqs } else { max_num_seqs };
            let gpu_memory_utilization = if (gpu_memory_utilization - 0.9).abs() < 0.001 {
                config.model.gpu_memory_utilization
            } else {
                gpu_memory_utilization
            };

            serve::run(
                host,
                port,
                model,
                vllm_port,
                no_vllm,
                max_num_seqs,
                gpu_memory_utilization,
                output_mode,
            )
            .await?;
        }
        Commands::Run { model, prompt } => {
            run::execute(model, prompt).await?;
        }
        Commands::Generate {
            model,
            prompt,
            stream,
        } => {
            generate::execute(model, prompt, stream).await?;
        }
        Commands::List => {
            list::execute(output_mode).await?;
        }
        Commands::Pull { model } => {
            pull::execute(model, output_mode).await?;
        }
        Commands::Rm { model } => {
            rm::execute(model, output_mode).await?;
        }
        Commands::Show {
            model,
            modelfile,
            parameters,
        } => {
            show::execute(model, modelfile, parameters).await?;
        }
        Commands::Ps => {
            ps::execute().await?;
        }
        Commands::Info => {
            info::execute().await?;
        }
        Commands::Bench {
            model,
            prompt,
            iterations,
            concurrency,
        } => {
            bench::execute(model, prompt, iterations, concurrency, output_mode).await?;
        }
        Commands::Config { show } => {
            if show {
                // Show current configuration
                match output_mode {
                    OutputMode::Json => {
                        output::json(&config);
                    }
                    _ => {
                        println!("{}", output::section("Current Configuration"));
                        println!();
                        let toml = toml::to_string_pretty(&config)?;
                        println!("{}", toml);
                    }
                }
            } else {
                // Generate example configuration
                println!("{}", output::section("Example Configuration"));
                println!();
                println!("Save this to ~/.config/vllama/config.toml or ./vllama.toml:\n");
                println!("{}", config::Config::example());
            }
        }
    }

    Ok(())
}

fn init_tracing(verbose: bool) {
    let filter = if verbose {
        "vllama=debug,info"
    } else {
        "vllama=info,warn"
    };

    let env_filter = tracing_subscriber::EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| filter.into());

    // Check if JSON logging is requested via environment variable
    let use_json = std::env::var("VLLAMA_LOG_FORMAT")
        .map(|v| v.to_lowercase() == "json")
        .unwrap_or(false);

    if use_json {
        // Structured JSON logging for production
        tracing_subscriber::registry()
            .with(env_filter)
            .with(
                tracing_subscriber::fmt::layer()
                    .json()
                    .with_current_span(true)
                    .with_span_list(false)
                    .with_target(true)
                    .with_thread_ids(false)
                    .with_file(false)
                    .with_line_number(false),
            )
            .init();
    } else {
        // Human-readable logging for development
        tracing_subscriber::registry()
            .with(env_filter)
            .with(tracing_subscriber::fmt::layer())
            .init();
    }
}
