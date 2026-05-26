# Active Context: plz-cli

## Current State
The project is a functional CLI tool that generates shell scripts through the UbiquityOS AI Gateway. It is a modified version of the original plz-cli that now uses the gateway's OpenAI-compatible chat completions endpoint.

## Recent Changes
- Migrated to the UbiquityOS AI Gateway
- Updated the default model to `gpt-5.3-codex-spark`
- Sends `reasoning_effort: xhigh` for chat completions
- Added per-request `--model` and `--reasoning-effort` overrides
- Added a stable `prompt_cache_key` and cache-friendly prompt ordering
- Streams generated scripts into a Bash-highlighted preview
- Uses `UOS_AI_TOKEN`, with `DENO_DEPLOY_TOKEN` as a fallback
- Removed previous-provider request headers and ignored generation parameters

## Active Decisions

### API Integration
- Using UbiquityOS AI Gateway chat completions
- Streaming API responses for immediate preview feedback
- Gateway-specific error handling
- Fixed gateway defaults with per-request model/reasoning CLI overrides
- Stable prompt cache key for repeated shell-generation requests

### Command Generation
- OS-aware prompt construction
- Environment variable context inclusion
- Cache-friendly prompt order with the user task at the end
- Direct bash script output
- No additional explanations in responses

### User Interface
- Command preview with syntax highlighting
- Interactive confirmation process
- Streaming visual feedback during generation
- Color-coded success/error messages

## Current Focus Areas

### Core Functionality
- Reliable command generation
- Safe command execution
- Proper error handling
- Shell integration

### User Experience
- Clear feedback during operations
- Intuitive command previews
- Simple confirmation process
- Helpful error messages

## Next Steps

### Short Term
1. Monitor UbiquityOS AI Gateway performance
2. Gather user feedback on command accuracy
3. Address any API integration issues
4. Fine-tune error messages

### Medium Term
1. Optimize prompt engineering
2. Enhance command preview features
3. Improve error handling coverage
4. Refine shell integration

## Known Issues
- Limited to bash script generation
- Requires manual API key configuration
- No persistent command history management
- Limited to Unix-like environments

## Active Considerations
- API response quality monitoring
- Command generation accuracy
- User interaction patterns
- Error handling completeness

## Integration Status
- UbiquityOS AI Gateway: ✓ Functional
- Shell Integration: ✓ Working
- History Management: ✓ Basic Support
- Error Handling: ✓ Implemented
