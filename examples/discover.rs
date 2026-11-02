use clap::{CommandFactory, Parser, Subcommand};
use clap_discovery::Discovery;

#[derive(Parser)]
#[command(name = "demo", version, about = "A self-discovering CLI")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Inspect this CLI without running application logic
    #[command(name = "self")]
    Discover {
        #[command(subcommand)]
        format: Discovery,
    },
    /// Manage accounts
    Account {
        #[command(subcommand)]
        action: AccountAction,
    },
}

#[derive(Subcommand)]
enum AccountAction {
    /// List accounts
    #[command(visible_alias = "ls")]
    List,
    /// Fetch an account
    Fetch {
        /// Account identifier
        #[arg(value_name = "ACCOUNT_ID")]
        id: String,
    },
}

fn main() -> std::io::Result<()> {
    match Cli::parse().command {
        Commands::Discover { format } => format.print(&Cli::command()),
        Commands::Account { .. } => Ok(()),
    }
}
