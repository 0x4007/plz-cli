use crate::execution::system::get_system_context;

pub fn build_prompt(prompt: &str) -> String {
    let system_context = get_system_context();
    
    format!(
        r#"<user_request>{}</user_request>

<system_info>
{}
</system_info>

<task>Generate a bash script for the following task. Provide only the script code, no explanations.</task>"#,
        prompt, system_context.trim()
    )
}