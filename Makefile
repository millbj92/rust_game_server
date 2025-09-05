run:
	cargo run -p moonhold-zone-server
fmt:
	cargo fmt --all
clippy:
	cargo clippy --all-targets --all-features -D warnings
release:
	cargo build --release -p moonhold-zone-server