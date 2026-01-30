mod commands;

use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "porter", about = "Porter - Personal Assistant", version)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Start the Porter server
    Serve {
        /// Path to the config file
        #[arg(short, long, default_value = "config/home.toml")]
        config: String,
    },
    /// Manage tasks
    Task {
        #[command(subcommand)]
        command: TaskCommands,
    },
    /// Manage Claude agent sessions
    Agent {
        #[command(subcommand)]
        command: AgentCommands,
    },
    /// Show server status
    Status {
        /// Server URL
        #[arg(short, long, default_value = "http://localhost:3101")]
        server: String,
    },
}

#[derive(Subcommand)]
enum TaskCommands {
    /// Create a new task
    New {
        /// Task title
        title: String,
        /// Task priority
        #[arg(short, long, default_value = "medium")]
        priority: String,
    },
    /// List tasks
    List {
        /// Filter by status
        #[arg(short, long)]
        status: Option<String>,
    },
}

#[derive(Subcommand)]
enum AgentCommands {
    /// Start a new Claude agent session
    Start {
        /// Prompt for the agent
        prompt: String,
    },
    /// List agent sessions
    List {
        /// Filter by status
        #[arg(short, long)]
        status: Option<String>,
    },
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Serve { config } => {
            commands::serve::run(&config).await?;
        }
        Commands::Task { command } => match command {
            TaskCommands::New { title, priority } => {
                commands::task::create("http://localhost:3101", &title, &priority).await?;
            }
            TaskCommands::List { status } => {
                commands::task::list("http://localhost:3101", status.as_deref()).await?;
            }
        },
        Commands::Agent { command } => match command {
            AgentCommands::Start { prompt } => {
                commands::agent::start("http://localhost:3101", &prompt).await?;
            }
            AgentCommands::List { status } => {
                commands::agent::list("http://localhost:3101", status.as_deref()).await?;
            }
        },
        Commands::Status { server } => {
            commands::status::run(&server).await?;
        }
    }

    Ok(())
}
