#!/bin/bash

echo "=== PLZ Debug Test - Showing All Prompts ==="
echo

# Set max retries to 1 for simpler output
export PLZ_MAX_RETRIES=1

# Create a test script that will definitely fail
cat > /tmp/test_fail_debug.sh << 'EOF'
#!/bin/bash
# This will fail with a specific error
somecommandthatdoesnotexist --with-flags
EOF
chmod +x /tmp/test_fail_debug.sh

echo "1. Testing initial prompt to OpenRouter:"
echo "========================================="
echo "Command: plz 'run the test script at /tmp/test_fail_debug.sh'"
echo

# First, let's modify plz to add debug output
cp src/main.rs src/main.rs.backup

# Add debug output to the generate_code function
cat > debug_patch.txt << 'PATCH'
--- a/src/main.rs
+++ b/src/main.rs
@@ -25,6 +25,14 @@
 
 fn generate_code(prompt: &str, config: &Config) -> Result<String, String> {
+    eprintln!("\n=== DEBUG: OpenRouter API Call ===");
+    eprintln!("Model: {}", config.model);
+    eprintln!("System Prompt: {}", config.system_prompt);
+    eprintln!("User Prompt: {}", prompt);
+    eprintln!("Temperature: 1e-10");
+    eprintln!("API Base: {}", config.api_base);
+    eprintln!("================================\n");
+
     let client = Client::builder()
         .timeout(std::time::Duration::from_secs(60))
         .build()
@@ -148,6 +156,10 @@ fn execute_with_retry(code: &str, config: &Config, cli: &Cli, prompt: &str, att
             system_context
         );
         
+        eprintln!("\n=== DEBUG: Perplexity Search Query ===");
+        eprintln!("{}", search_query);
+        eprintln!("====================================\n");
+        
         let perplexity_output = Command::new("bash")
             .arg("-c")
             .arg(format!("? {}", search_query))
@@ -172,6 +184,10 @@ fn execute_with_retry(code: &str, config: &Config, cli: &Cli, prompt: &str, att
                     perplexity_response.trim()
                 );
                 
+                eprintln!("\n=== DEBUG: Solution Prompt to OpenRouter ===");
+                eprintln!("{}", solution_prompt);
+                eprintln!("==========================================\n");
+                
                 match generate_code(&solution_prompt, &config) {
                     Ok(new_code) => {
                         spinner.stop_and_persist(
PATCH

echo "2. Gathering system context..."
echo "=============================="

# Show what system context would be gathered
echo "OS: $(uname -s) $(uname -m)"
echo "Shell: $SHELL"
if [[ "$(uname -s)" == "Darwin" ]]; then
    echo "macOS version: $(sw_vers -productVersion)"
fi

echo
echo "Available package managers:"
for cmd in brew apt yum dnf pacman npm cargo pip pip3; do
    if command -v $cmd &> /dev/null; then
        echo "  - $cmd"
    fi
done

echo
echo "3. Running test with debug output enabled..."
echo "==========================================="
echo "This will show:"
echo "  - Initial prompt to OpenRouter"
echo "  - Full Perplexity search query when command fails"
echo "  - Solution prompt back to OpenRouter"
echo

read -p "Press Enter to run the test with debug output..."