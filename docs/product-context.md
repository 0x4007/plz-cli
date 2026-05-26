# Product Context: plz-cli

## Problem Statement
Terminal operations often require specific command knowledge and syntax memorization, creating barriers for users who know what they want to do but aren't familiar with the exact commands needed. Even experienced users may need to frequently consult documentation or search for complex command combinations.

## Solution
plz-cli bridges this gap by allowing users to describe their intended actions in natural language. By using the UbiquityOS AI Gateway, it translates these descriptions into appropriate shell commands, making terminal operations more accessible and efficient.

## User Experience Goals

### Primary Goals
1. **Intuitive Input**
   - Users should be able to describe their needs in plain English
   - No need to memorize specific command syntax
   - Support for various command types and operations

2. **Safe Execution**
   - Preview generated commands before execution
   - Clear confirmation process
   - Option to force execution for automation needs

3. **Reliable Output**
   - Accurate command generation
   - Proper error handling
   - Clear feedback on execution status

### User Workflows

#### Basic Usage
1. User provides natural language description
2. System generates appropriate shell command
3. User reviews generated command
4. User confirms or modifies execution
5. System executes command and provides feedback

#### Automated Usage
1. User provides description with --force flag
2. System generates and executes command directly
3. System provides execution feedback

## Success Criteria
- Users can get desired results without knowing specific commands
- Generated commands are safe and accurate
- Execution process is transparent and controlled
- Error handling prevents dangerous operations
- Quick and efficient command generation

## Key Differentiators
- Uses the UbiquityOS AI Gateway for command generation
- Focus on safety with command preview
- Simple, straightforward interface
- Fast and efficient operation
- No persistent state or complex configuration needed
