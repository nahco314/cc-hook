mod cli;
mod config;
mod frame;
mod hook;
mod runner;
mod screen;

use std::process;

#[tokio::main]
async fn main() {
    let (config_path, args) = cli::Cli::parse_args();
    
    let config = match config::load_config(config_path) {
        Ok(cfg) => cfg,
        Err(e) => {
            eprintln!("Failed to load config: {}", e);
            process::exit(1);
        }
    };
    
    match runner::run_with_hooks(args, config).await {
        Ok(exit_code) => process::exit(exit_code),
        Err(e) => {
            eprintln!("Error: {}", e);
            process::exit(1);
        }
    }
}