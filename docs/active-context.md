# Active Context: plz-cli

## Current State
The project is a functional CLI tool that generates shell scripts using Claude 3.5 Sonnet. It is a modified version of the original plz-cli that now uses Claude 3.5 Sonnet instead of GPT-3.5 Turbo Instruct.

## Recent Changes
- Migrated from GPT-3.5 Turbo Instruct to Claude 3.5 Sonnet
- Updated API integration for Anthropic's Claude API
- Adjusted prompt formatting for improved command generation
- Added environment variable collection for context

## Active Decisions

### API Integration
- Using Claude 3.5 Sonnet model
- Synchronous API calls for simplicity
- Error handling for both client and server errors
- Maximum token limit set to 1000

### Command Generation
- OS-aware prompt construction
- Environment variable context inclusion
- Direct bash script output
- No additional explanations in responses

### User Interface
- Command preview with syntax highlighting
- Interactive confirmation process
- Progress indicators during generation
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
1. Monitor Claude 3.5 Sonnet performance
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
- Claude API: ✓ Functional
- Shell Integration: ✓ Working
- History Management: ✓ Basic Support
- Error Handling: ✓ Implemented
