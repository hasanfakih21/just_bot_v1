build:
	cargo rustc --release --bin justbot -- -C target-cpu=native

pgo:
	cargo pgo instrument
	cargo pgo run -- bench
	cargo pgo optimize