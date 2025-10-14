mod commands;
mod config;

use anyhow::Result;
use clap::{Parser, Subcommand};
use commands::*;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[derive(Parser)]
#[command(name = "hyperllama")]
#[command(about = "High-performance local LLM inference server", long_about = None)]
#[command(version)]
struct Cli {
    #[command(subcommand)]
    command: Commands,

    #[arg(long, global = true, help = "Enable verbose logging")]
    verbose: bool,

    #[arg(long, global = true, help = "Enable JSON output")]
    json: bool,
}

#[derive(Subcommand)]
enum Commands {
    #[command(about = "Start the HyperLlama server")]
    Serve {
        #[arg(long, default_value = "127.0.0.1", help = "Server host address")]
        host: String,

        #[arg(short, long, default_value = "11434", help = "Server port")]
        port: u16,
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

    #[command(about = "Benchmark MAX Engine vs Ollama")]
    Bench {
        #[arg(help = "Model name")]
        model: String,

        #[arg(help = "Prompt for benchmarking", default_value = "Once upon a time")]
        prompt: String,

        #[arg(short, long, help = "Number of iterations", default_value = "5")]
        iterations: usize,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    init_tracing(cli.verbose);

    match cli.command {
        Commands::Serve { host, port } => {
            serve::run(host, port).await?;
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
            list::execute().await?;
        }
        Commands::Pull { model } => {
            pull::execute(model).await?;
        }
        Commands::Rm { model } => {
            rm::execute(model).await?;
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
        } => {
            bench::execute(model, prompt, iterations).await?;
        }
    }

    Ok(())
}

fn init_tracing(verbose: bool) {
    let filter = if verbose {
        "hyperllama=debug,info"
    } else {
        "hyperllama=info,warn"
    };

    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| filter.into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();
}
