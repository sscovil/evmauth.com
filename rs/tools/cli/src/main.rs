mod commands;

use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "evmauth-cli", about = "EVMAuth internal CLI tool")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Send ETH from Anvil account #0 to an address (local dev only)
    Fund {
        /// Target address to fund
        address: String,
        /// Amount of ETH to send
        #[arg(long)]
        amount: String,
    },
    /// Deploy contracts
    Deploy {
        #[command(subcommand)]
        command: DeployCommands,
    },
}

#[derive(Subcommand)]
enum DeployCommands {
    /// Deploy EVMAuth beacon implementation contract
    Beacon,
    /// Deploy platform proxy contract
    Platform {
        /// Beacon contract address
        #[arg(long)]
        beacon: String,
    },
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Fund { address, amount } => {
            commands::fund::execute(&address, &amount).await?;
        }
        Commands::Deploy { command } => match command {
            DeployCommands::Beacon => {
                commands::deploy_beacon::execute().await?;
            }
            DeployCommands::Platform { beacon } => {
                commands::deploy_platform::execute(&beacon).await?;
            }
        },
    }

    Ok(())
}
