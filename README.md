# Copilot, for your terminal

A CLI tool that generates shell scripts from a human readable description.

This is a modded version of [plz-cli](https://github.com/m1guelpf/plz-cli) that now uses Claude 3.5 Sonnet instead of GPT-3.5 Turbo Instruct.

## Installation

```
cargo build --release
cp ./target/release/plz ~/.bin/plz
```

You may need to close and reopen your terminal after installation. Alternatively, you can download the binary corresponding to your OS from the [latest release](https://github.com/0x4007/plz-cli/releases/latest).

## Usage

`plz` uses [Claude 3.5 Sonnet](https://console.anthropic.com/). To use it, you'll need to grab an API key from [your dashboard](https://console.anthropic.com/settings/keys), and save it to `ANTHROPIC_API_KEY` as follows (you can also save it in your bash/zsh profile for persistance between sessions).

```bash
export ANTHROPIC_API_KEY='sk-ant-XXXXXXXX'
```

Once you have configured your environment, run `plz` followed by whatever it is that you want to do (`plz show me all options for the plz cli`).

To get a full overview of all available options, run `plz --help`

```sh
$ plz --help
Generates bash scripts from the command line

Usage: plz [OPTIONS] <PROMPT>

Arguments:
  <PROMPT>  Description of the command to execute

Options:
  -y, --force    Run the generated program without asking for confirmation
  -h, --help     Print help information
  -V, --version  Print version information
```

## Develop

Make sure you have the latest version of rust installed (use [rustup](https://rustup.rs/)). Then, you can build the project by running `cargo build`, and run it with `cargo run`.

## License

This project is open-sourced under the MIT license. See [the License file](LICENSE) for more information.
