#![warn(clippy::all, clippy::pedantic, clippy::nursery)]

use bat::PrettyPrinter;
use clap::Parser;
use colored::Colorize;
use question::{Answer, Question};
use spinners::{Spinner, Spinners};

mod api;
mod cli;
mod config;
mod execution;
mod utils;

use api::client::generate_code;
use cli::Cli;
use config::Config;
use execution::retry::execute_with_retry;
use utils::prompt::build_prompt;

fn main() {
    let cli = Cli::parse();
    let config = Config::new();
    let prompt = build_prompt(&cli.prompt.join(" "));
    let mut spinner = Spinner::new(Spinners::BouncingBar, "Generating your command...".into());

    let code_result = generate_code(&prompt, &config);

        match code_result {
            Ok(code) => {
                spinner.stop_and_persist(
                    "✔".green().to_string().as_str(),
                    "Got some code!".green().to_string(),
                );

                PrettyPrinter::new()
                    .input_from_bytes(code.as_bytes())
                    .language("bash")
                    .grid(true)
                    .print()
                    .unwrap();

                let should_run = if cli.force {
                    true
                } else {
                    Question::new(
                        ">> Run the generated program? [Y/n]"
                            .bright_black()
                            .to_string()
                            .as_str(),
                    )
                    .yes_no()
                    .until_acceptable()
                    .default(Answer::YES)
                    .ask()
                    .expect("Couldn't ask question.")
                        == Answer::YES
                };

                if should_run {
                    config.write_to_history(code.as_str());
                    if !execute_with_retry(&code, &config, &cli, &prompt, 1) {
                        std::process::exit(1);
                    }
                }
            }
            Err(e) => {
                spinner.stop_and_persist(
                    "✖".red().to_string().as_str(),
                    format!("{}", e).red().to_string(),
                );
                std::process::exit(1);
            }
        }
}