#!/bin/bash
# Test script to verify UbiquityOS AI Gateway configuration

echo "=== Testing plz-cli Gateway Configuration ==="
echo

# Check current configuration
if [ -n "${UOS_AI_TOKEN:-}" ]; then
    echo "Auth source: UOS_AI_TOKEN"
elif [ -n "${DENO_DEPLOY_TOKEN:-}" ]; then
    echo "Auth source: DENO_DEPLOY_TOKEN"
else
    echo "Missing auth. Set UOS_AI_TOKEN or DENO_DEPLOY_TOKEN."
    exit 1
fi
echo

# Test with default gateway settings
echo "1. Testing with default gateway settings:"
plz "echo hello world"
echo

# Test with explicit model and reasoning overrides
echo "2. Testing explicit model/reasoning overrides:"
plz --model gpt-5.3-codex-spark --reasoning-effort low "echo hello world"
echo

echo "=== Gateway Configuration Test Complete ==="
echo
echo "Available models can be checked with:"
echo "  curl -sS https://ai.ubq.fi/v1/models -H 'Authorization: Bearer \$UOS_AI_TOKEN'"
