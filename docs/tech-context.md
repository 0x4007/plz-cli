# Technical Context: plz-cli

## Technology Stack

### Core Technologies
1. **Rust**
   - Primary development language
   - Used for system-level interactions
   - Provides strong safety guarantees
   - Efficient binary output

2. **OpenRouter API (with Claude 3 Sonnet)**
   - Managed API access to Claude
   - Natural language processing
   - Command generation
   - High accuracy results

### Key Dependencies
- **clap**: Command line argument parsing
- **reqwest**: HTTP client for API calls
- **serde_json**: JSON serialization/deserialization
- **colored**: Terminal text coloring
- **spinners**: Terminal progress indicators
- **bat**: Syntax highlighting for command preview
- **question**: User input handling

## Development Setup

### Requirements
1. **Rust Installation**
   - Latest version via rustup
   - Cargo package manager
   - Build tools and dependencies

2. **Environment Configuration**
   ```bash
   OPENROUTER_API_KEY=sk-xxxx    # Required: OpenRouter API key
   OPENROUTER_API_BASE=https://openrouter.ai/api    # Optional: API endpoint
   ```

### Build Process
1. Development Build:
   ```bash
   cargo build
   ```

2. Production Release:
   ```bash
   cargo build --release
   ```

3. Installation:
   ```bash
   cp ./target/release/plz ~/.bin/plz
   ```

## Technical Constraints

### API Limitations
- Requires valid OpenRouter API key
- Subject to API rate limits
- Dependent on API availability
- Limited to bash script generation

### System Requirements
- Unix-like environment (Linux/macOS)
- Bash or Zsh shell
- Write access for history files
- Network connectivity

## Configuration Details

### API Configuration
- API key from environment
- Configurable API base URL
- Configurable token limits (4000 default, 32000 extended)
- Error handling for API issues

### Shell Integration
- Supports Bash and Zsh
- History file management
- Environment variable access
- Command execution handling

## Testing & Quality Assurance

### Code Quality
- Clippy lints enabled
  - all
  - pedantic
  - nursery
- Strong type safety
- Error handling patterns
- Code documentation

### Release Process
1. Version tagging
2. Automated releases
3. Binary distribution
4. Platform-specific builds

## Deployment & Distribution

### Binary Distribution
- GitHub releases
- Platform-specific binaries
- Manual installation option
- Version management

### Installation Methods
1. From source (cargo build)
2. Pre-built binaries
3. Manual binary placement
4. Shell configuration

## Maintenance Considerations

### Update Process
- Version bumping
- Tag-based releases
- Dependency updates
- API compatibility checks

### Monitoring Points
- API response times
- Error rates
- Command success rates
- User feedback
