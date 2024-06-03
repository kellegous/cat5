HURDAT2_URL := "https://www.nhc.noaa.gov/data/hurdat/hurdat2-1851-2023-051124.txt"

target/release/cat5: Cargo.toml $(shell find src -type f)
	cargo build --release

hurdat2.csv:
	curl -o $@ $(HURDAT2_URL)