#!/bin/bash
# Test script to verify LLM model configuration

echo "=== Testing plz-cli LLM Configuration ==="
echo

# Check current configuration
echo "Current OPENROUTER_MODEL: ${OPENROUTER_MODEL:-Not set}"
echo

# Test with default model
echo "1. Testing with default model (should use Claude Opus):"
plz "echo hello world"
echo

# Test with a different model
echo "2. Testing with different model (Claude 3.5 Sonnet):"
OPENROUTER_MODEL="anthropic/claude-3.5-sonnet-20241022" plz "echo testing with sonnet"
echo

# Test with invalid model (should still work but might get different results)
echo "3. Testing with custom model specification:"
OPENROUTER_MODEL="openai/gpt-4-turbo" plz "echo testing with gpt-4"
echo

echo "=== Configuration Test Complete ==="
echo
echo "To permanently change the model, update your .bashrc:"
echo "  export OPENROUTER_MODEL='model-name-here'"
echo
echo "Available models can be found at: https://openrouter.ai/models"
