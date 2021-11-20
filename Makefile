.PHONY: all build run release static clean test darwin

all:
	cargo build

run:
	cargo run

release:
	cargo build --release

darwin: export CC=o64-clang
darwin: export CXX=o64-clang++
darwin: export LIBZ_SYS_STATIC=1
darwin:
	PATH=/usr/local/darwin-ndk-x86_64/bin:$$PATH \
		 cargo build --target=x86_64-apple-darwin --release

windows:
	cargo build --release --target x86_64-pc-windows-gnu

static:
	cargo build --release --target x86_64-unknown-linux-musl

clean:
	cargo clean

check:
	cargo check

test:
	cargo test

