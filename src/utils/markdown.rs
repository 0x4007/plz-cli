/// Extract code from markdown code blocks
pub fn extract_code_from_markdown(content: &str) -> String {
    // Check if the content contains markdown code blocks
    if content.contains("```") {
        // Find the first code block
        if let Some(start) = content.find("```") {
            let after_start = &content[start + 3..];
            
            // Skip the language identifier (e.g., bash, sh, shell)
            let code_start = if let Some(newline_pos) = after_start.find('\n') {
                start + 3 + newline_pos + 1
            } else {
                return content.to_string(); // No newline after ```, return as-is
            };
            
            // Find the ending ```
            if let Some(end_pos) = content[code_start..].find("```") {
                return content[code_start..code_start + end_pos].trim().to_string();
            }
        }
    }
    
    // If no markdown code blocks found, return content as-is
    content.trim().to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_bash_code_block() {
        let markdown = r#"```bash
#!/bin/bash
echo "Hello World"
```"#;
        assert_eq!(extract_code_from_markdown(markdown), "#!/bin/bash\necho \"Hello World\"");
    }

    #[test]
    fn test_extract_code_block_with_language() {
        let markdown = r#"Here's the code:
```sh
ls -la
```
Done!"#;
        assert_eq!(extract_code_from_markdown(markdown), "ls -la");
    }

    #[test]
    fn test_no_code_block() {
        let plain = "echo 'Hello World'";
        assert_eq!(extract_code_from_markdown(plain), "echo 'Hello World'");
    }

    #[test]
    fn test_multiple_code_blocks_extracts_first() {
        let markdown = r#"```bash
first block
```
Some text
```bash
second block
```"#;
        assert_eq!(extract_code_from_markdown(markdown), "first block");
    }
}