#!/bin/bash

echo "=== PLZ Debug Logging Test ==="
echo "All debug output will be saved to the logs/ directory"
echo

# Enable debug mode
export PLZ_DEBUG=1
export PLZ_MAX_RETRIES=1

# Create a failing test script
cat > /tmp/fail_logging_test.sh << 'EOF'
#!/bin/bash
# This will trigger Perplexity search
ffmpeg_that_doesnt_exist -i video.mp4 -o audio.mp3
EOF
chmod +x /tmp/fail_logging_test.sh

echo "Running test command that will fail and trigger Perplexity search..."
echo "Check the logs/ directory for detailed output"
echo

plz -y "convert a video file to audio using ffmpeg but use a command that doesn't exist"

echo
echo "Debug logs have been written to:"
ls -la logs/plz_debug_*.log 2>/dev/null | tail -5