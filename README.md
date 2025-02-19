# tty-override

This is a simple software that allows you to override the stdout and stderr of a tty.

## Install

```sh
cargo install --git https://github.com/umutbasal/tty-override
```

## Usage
### Example Config
```toml
[gh-copilot."*"]
rules = [
	["Welcome.*\n", ""],
	["version.*\n", ""],
	["I'm powered.*\n", ""],
	["^\\W\\[[0-9;]*m\\W\\[[0-9;]*m\r\n", ""],
	["^\\W\\[[0-9;]*m\\W.*?\\[2K\r\n", ""]
]

[vi." "]
rules = [
	["VIM - Vi IMproved", "      VSCODE"],
]
```

```sh
#~/tty-override/config/config.toml
curl https://raw.githubusercontent.com/umutbasal/tty-override/master/config/config.toml --create-dirs -o ~/.config/tty-override/config.toml
```

```sh
tty-override gh copilot suggest "list all files in the current directory"
```

```sh
# aliases in .zshrc
alias '??'='tty-override gh copilot suggest -t shell'
alias 'git?'='tty-override gh copilot suggest -t git'
alias 'gh?'='tty-override gh copilot suggest -t gh'
alias 'explain'='tty-override gh copilot explain'
```

![Output](image.png)
