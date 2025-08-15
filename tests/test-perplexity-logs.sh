#!/bin/bash

echo "=== PLZ Perplexity Debug Test ==="
echo "This will show what gets sent to Perplexity when a command fails"
echo

# Clean up old logs
rm -f logs/plz_debug_*.log

# Enable debug mode
export PLZ_DEBUG=1
export PLZ_MAX_RETRIES=1

# Create a command that will definitely fail
echo "Running a command that will fail and trigger Perplexity search..."
echo

plz -y "use a command called 'magic_unicorn_command' to list files"

echo
echo "=== Generated log files: ==="
ls -la logs/plz_debug_*.log 2>/dev/null

echo
echo "=== Perplexity Query Log Content: ==="
cat logs/plz_debug_*_perplexity.log 2>/dev/null

echo
echo "=== Solution Log Content (includes Perplexity response): ==="
cat logs/plz_debug_*_solution.log 2>/dev/null