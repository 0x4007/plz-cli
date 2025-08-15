#!/bin/bash
# Test script to verify markdown code block extraction fix

echo "Testing plz-cli with a simple command that should generate markdown-wrapped code..."
PLZ_DEBUG=1 ./target/debug/plz "echo hello world"