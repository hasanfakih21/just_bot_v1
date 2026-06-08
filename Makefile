EXE = justbot

build:
	cargo rustc --release --bin justbot -- -C target-cpu=native --emit link=$(EXE)

pgo:
	cargo pgo instrument
	cargo pgo run -- bench
	cargo pgo optimize