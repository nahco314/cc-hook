use clap::{Parser, Subcommand};
use std::path::PathBuf;

#[derive(Parser)]
#[command(author, version, about = "Terminal wrapper with hook-triggered commands", long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Commands>,

    #[arg(short = 'c', long, value_name = "FILE")]
    pub config: Option<PathBuf>,

    #[arg(trailing_var_arg = true)]
    pub args: Vec<String>,
}

#[derive(Subcommand)]
pub enum Commands {
    ConfigPath,

    Run {
        #[arg(trailing_var_arg = true)]
        args: Vec<String>,
    },
}

impl Cli {
    pub fn parse_args() -> (Option<PathBuf>, Vec<String>) {
        let cli = Cli::parse();

        match cli.command {
            Some(Commands::ConfigPath) => {
                println!("{}", crate::config::default_config_path().display());
                std::process::exit(0);
            }
            Some(Commands::Run { args }) => (cli.config, args),
            None => {
                if cli.args.is_empty() {
                    eprintln!("Error: No command provided");
                    std::process::exit(1);
                }
                (cli.config, cli.args)
            }
        }
    }
}
