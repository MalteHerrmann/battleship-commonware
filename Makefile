run:
	@RUST_LOG=debug cargo run --bin=player

test:
	@cargo nextest run

