fix:
	@rustup component add rustfmt --toolchain stable 2> /dev/null
	cargo +stable clippy --fix --allow-dirty --all-features --all --tests --examples -- -D clippy::all && cargo +stable fmt --all
.PHONY: fix