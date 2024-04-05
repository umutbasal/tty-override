# tty-override

This is a simple script that allows you to override the stdout and stderr of a tty.

## Install

```sh
cargo install --git https://github.com/umutbasal/tty-override
```

## Usage

```sh
#~/tty-override/config/config.toml
curl https://raw.githubusercontent.com/umutbasal/tty-override/master/config/config.toml -o ~/tty-override/config/config.toml
```

```sh
tty-override gh copilot suggest "list all files in the current directory"
```

![Output](image.png)
