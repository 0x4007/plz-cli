#!/bin/bash

echo "=== PLZ Debug Test - Showing All Prompts ==="
echo
echo "This test will show exactly what is sent to OpenRouter and Perplexity"
echo

# Enable debug mode
export PLZ_DEBUG=1
export PLZ_MAX_RETRIES=1

# Create a failing test script
cat > /tmp/fail_test.sh << 'EOF'
#!/bin/bash
nonexistentcommand --help
EOF
chmod +x /tmp/fail_test.sh

echo "Running: plz -y 'run the script /tmp/fail_test.sh'"
echo "=================================================="
echo

plz -y "run the script /tmp/fail_test.sh" 2>&1