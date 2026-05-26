# Project Brief: plz-cli

## Overview
plz-cli is a command-line interface tool that generates shell scripts from natural language descriptions using the UbiquityOS AI Gateway. It simplifies terminal operations by allowing users to describe what they want to do in plain English.

## Core Requirements
1. Accept natural language prompts from users
2. Generate appropriate shell scripts through the UbiquityOS AI Gateway
3. Execute generated scripts with user confirmation
4. Support environment configuration (`UOS_AI_TOKEN` or `DENO_DEPLOY_TOKEN`)

## Goals
- Simplify complex terminal operations
- Provide intuitive interface for command generation
- Ensure safe execution with user confirmation
- Maintain high reliability and accuracy in script generation

## Target Users
- Developers and system administrators
- Users who need to perform terminal operations but aren't familiar with specific commands
- Power users looking to streamline their workflow

## Key Features
1. Natural language command generation
2. Script preview before execution
3. Force execution option (-y/--force flag)
4. Version information display
5. Help documentation

## Success Metrics
- Accurate translation of natural language to shell commands
- Safe execution with proper error handling
- User satisfaction with generated scripts
- Quick and efficient command generation

## Project Scope
### In Scope
- Natural language processing for command generation
- Shell script generation and execution
- Environment configuration management
- Basic CLI options (help, version, force)

### Out of Scope
- GUI interface
- Persistent command history
- Multi-language support
- Complex script workflows
