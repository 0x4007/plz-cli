# Copilot, for your terminal

A CLI tool that generates shell scripts from a human readable description.

This is a modded version of [plz-cli](https://github.com/m1guelpf/plz-cli) that uses the UbiquityOS AI Gateway.

## Installation

```
cargo build --release
cp ./target/release/plz ~/.bin/plz

```

You may need to close and reopen your terminal after installation. Alternatively, you can download the binary corresponding to your OS from the [latest release](https://github.com/0x4007/plz-cli/releases/latest).

## Usage

`plz` uses the UbiquityOS AI Gateway at `https://ai.ubq.fi/v1/chat/completions`. To use it, set a gateway token in `UOS_AI_TOKEN` (you can also save it in your bash/zsh profile for persistence between sessions). If `UOS_AI_TOKEN` is not set, `plz` falls back to `DENO_DEPLOY_TOKEN`.

```bash
export UOS_AI_TOKEN='...'
```

The CLI sends streaming requests with model `gpt-5.3-codex-spark` and `reasoning_effort: xhigh` by default. Requests include a stable `prompt_cache_key`, and the prompt keeps reusable instruction/environment context before the task. Generated scripts are shown as a live Bash-highlighted preview before the confirmation prompt.

Use `GET https://ai.ubq.fi/v1/models` with the same bearer token to inspect models available through the gateway.

### Examples

```bash
# Basic usage with the default gateway model
plz list all files in current directory

# Run the generated script without an interactive confirmation prompt
plz -y find all large files over 100MB

# Try a different model and reasoning effort for one request
plz --model gpt-5.3-codex --reasoning-effort low summarize this repo

# Short alias for reasoning effort
plz --reasoning high list stale branches
```

Once you have configured your environment, run `plz` followed by whatever it is that you want to do (`plz show me all options for the plz cli`).

To get a full overview of all available options, run `plz --help`

```sh
$ plz --help
Generate bash scripts from the command line using the UbiquityOS AI Gateway

Usage: plz [OPTIONS] [PROMPT]...

Arguments:
  [PROMPT]...  Description of the command to execute

Options:
  -y, --force                      Run the generated program without asking for confirmation
      --model <MODEL>              Override the gateway model for this request
      --reasoning-effort <EFFORT>  Override reasoning effort for this request [aliases: reasoning] [possible values: none, minimal, low, medium, high, xhigh, max]
  -h, --help                       Print help information
  -V, --version                    Print version information
```

## Develop

Make sure you have the latest version of rust installed (use [rustup](https://rustup.rs/)). Then, you can build the project by running `cargo build`, and run it with `cargo run`.

### Releasing

Releases are automated with v prefixed tags. To create a new release, run something similar to the following:

```sh
SEMVER=0.0.0
RELEASE_COMMENT="Default Message. Release 0.0.0"

git tag -s $SEMVER -m "$RELEASE_COMMENT"
git push origin $SEMVER
```

## License

This project is open-sourced under the MIT license. See [the License file](LICENSE) for more information.
