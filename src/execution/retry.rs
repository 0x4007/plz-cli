use bat::PrettyPrinter;
use colored::Colorize;
use question::{Answer, Question};
use spinners::{Spinner, Spinners};
use std::process::Command;

use crate::api::client::generate_code;
use crate::api::perplexity::query_perplexity;
use crate::cli::Cli;
use crate::config::Config;
use crate::execution::system::get_system_context;
use crate::execution::history::{AttemptHistory, format_attempt_history};
use crate::utils::logging::log_to_file;

pub fn execute_with_retry(code: &str, config: &Config, cli: &Cli, prompt: &str, attempt: u8) -> bool {
    execute_with_retry_and_history(code, config, cli, prompt, attempt, &mut Vec::new())
}

fn execute_with_retry_and_history(code: &str, config: &Config, cli: &Cli, prompt: &str, attempt: u8, history: &mut Vec<AttemptHistory>) -> bool {
    let max_retries = std::env::var("PLZ_MAX_RETRIES")
        .ok()
        .and_then(|s| s.parse::<u8>().ok())
        .unwrap_or(2);
    
    if attempt > max_retries {
        println!("{}", format!("Maximum retry attempts ({}) reached. Aborting.", max_retries).red());
        return false;
    }
    
    let mut spinner = Spinner::new(Spinners::BouncingBar, 
        if attempt == 1 { "Executing...".into() } else { "Executing improved solution...".into() }
    );
    
    let output = Command::new("bash")
        .arg("-c")
        .arg(code)
        .output()
        .unwrap_or_else(|_| {
            spinner.stop_and_persist(
                "✖".red().to_string().as_str(),
                "Failed to execute the generated program.".red().to_string(),
            );
            std::process::exit(1);
        });
    
    if output.status.success() {
        spinner.stop_and_persist(
            "✔".green().to_string().as_str(),
            if attempt == 1 { 
                "Command ran successfully".green().to_string() 
            } else { 
                "Solution executed successfully!".green().to_string() 
            },
        );
        println!("{}", String::from_utf8_lossy(&output.stdout));
        true
    } else {
        let error_output = String::from_utf8_lossy(&output.stderr);
        spinner.stop_and_persist(
            "✖".red().to_string().as_str(),
            if attempt == 1 { 
                "Command execution failed".red().to_string() 
            } else { 
                "The improved solution also failed".red().to_string() 
            },
        );
        println!("{}", error_output);
        
        if attempt > max_retries {
            println!("{}", format!("Reached maximum retry attempts ({}). Aborting.", max_retries).red());
            return false;
        }
        
        // Add this attempt to history
        history.push(AttemptHistory {
            attempt_number: attempt,
            generated_code: code.to_string(),
            error_output: error_output.to_string(),
            perplexity_analysis: None,
        });
        
        handle_retry_with_search(code, config, cli, prompt, attempt, &error_output, history)
    }
}

fn handle_retry_with_search(
    code: &str,
    config: &Config,
    cli: &Cli,
    prompt: &str,
    attempt: u8,
    error_output: &str,
    history: &mut Vec<AttemptHistory>,
) -> bool {
    let mut spinner = Spinner::new(Spinners::BouncingBar, "Searching for error solution...".into());
    
    let system_context = get_system_context();
    // Extract just the user's request from the prompt
    let user_request = if let Some(start) = prompt.find("<user_request>") {
        let start_pos = start + "<user_request>".len();
        if let Some(end) = prompt[start_pos..].find("</user_request>") {
            prompt[start_pos..start_pos + end].trim()
        } else {
            prompt.lines().next().unwrap_or(prompt).trim()
        }
    } else {
        prompt.lines().next().unwrap_or(prompt).trim()
    };
    
    let history_section = format_attempt_history(history);
    
    let search_query = format!(
        "<role>You are a debugging assistant helping to fix a failed bash command.</role>\n\n\
        <user_request>{}</user_request>\n\n\
        <system_info>\n{}</system_info>\n\n\
        {}\n\n\
        <current_attempt number=\"{}\">\n\
        <openrouter_generated_code>\n{}\n</openrouter_generated_code>\n\n\
        <bash_error>\n{}\n</bash_error>\n\
        </current_attempt>\n\n\
        <task>Analyze why this command failed. Consider the history of previous attempts and avoid suggesting solutions that have already been tried. Provide a NEW approach to accomplish what the user wanted.</task>",
        user_request,
        system_context.trim(),
        history_section,
        attempt,
        code.trim(),
        error_output.trim()
    );
    
    log_perplexity_query(&search_query, attempt);
    
    // Call Perplexity API directly instead of using bash alias
    let perplexity_result = query_perplexity(&search_query);
    
    match perplexity_result {
        Ok(perplexity_response) => {
            spinner.stop_and_persist(
                "🔍".to_string().as_str(),
                format!("Found potential solution from Perplexity (attempt {}/{})", attempt, 2).bright_blue().to_string(),
            );
            
            // Update the last history entry with Perplexity's analysis
            if let Some(last) = history.last_mut() {
                last.perplexity_analysis = Some(perplexity_response.clone());
            }
            
            generate_and_execute_solution(config, cli, prompt, error_output, &system_context, &perplexity_response, attempt, history)
        }
        Err(e) => {
            spinner.stop_and_persist(
                "⚠".yellow().to_string().as_str(),
                format!("Could not search Perplexity: {}", e).yellow().to_string(),
            );
            false
        }
    }
}

fn generate_and_execute_solution(
    config: &Config,
    cli: &Cli,
    prompt: &str,
    _error_output: &str,
    system_context: &str,
    perplexity_response: &str,
    attempt: u8,
    history: &mut Vec<AttemptHistory>,
) -> bool {
    let mut spinner = Spinner::new(Spinners::BouncingBar, "Generating solution based on search results...".into());
    
    let history_section = format_attempt_history(history);
    
    // Extract just the user's request from the prompt
    let user_request = if let Some(start) = prompt.find("<user_request>") {
        let start_pos = start + "<user_request>".len();
        if let Some(end) = prompt[start_pos..].find("</user_request>") {
            prompt[start_pos..start_pos + end].trim()
        } else {
            prompt.lines().next().unwrap_or(prompt).trim()
        }
    } else {
        prompt.lines().next().unwrap_or(prompt).trim()
    };
    
    let solution_prompt = format!(
        "<user_request>{}</user_request>\n\n\
        <system_info>\n{}</system_info>\n\n\
        {}\n\n\
        <latest_perplexity_response>\n{}\n</latest_perplexity_response>\n\n\
        <task>Based on the full history of attempts and Perplexity's analysis, provide a NEW bash script that will work. DO NOT repeat any solution that has already been tried.</task>",
        user_request,
        system_context.trim(),
        history_section,
        perplexity_response.trim()
    );
    
    log_solution_prompt(perplexity_response, &solution_prompt, attempt);
    
    match generate_code(&solution_prompt, &config) {
        Ok(new_code) => {
            spinner.stop_and_persist(
                "✨".to_string().as_str(),
                "Generated solution based on search results:".green().to_string(),
            );
            
            PrettyPrinter::new()
                .input_from_bytes(new_code.as_bytes())
                .language("bash")
                .grid(true)
                .print()
                .unwrap();
            
            let should_run = if cli.force {
                true
            } else {
                Question::new(
                    ">> Run the improved solution? [Y/n]"
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
                config.write_to_history(new_code.as_str());
                execute_with_retry_and_history(&new_code, config, cli, prompt, attempt + 1, history)
            } else {
                false
            }
        }
        Err(e) => {
            spinner.stop_and_persist(
                "✖".red().to_string().as_str(),
                format!("Failed to generate solution: {}", e).red().to_string(),
            );
            false
        }
    }
}

fn log_perplexity_query(search_query: &str, attempt: u8) {
    let step = format!("step2_attempt{}", attempt);
    log_to_file(search_query, "perplexity_request", &step);
}

fn log_solution_prompt(perplexity_response: &str, solution_prompt: &str, retry_num: u8) {
    let step = format!("step2_attempt{}", retry_num);
    log_to_file(perplexity_response, "perplexity_response", &step);
    
    let step = format!("step3_attempt{}", retry_num);
    log_to_file(solution_prompt, "openrouter_retry_request", &step);
}