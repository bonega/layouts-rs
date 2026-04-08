layout := "colemak-dh"
corpus := "./corpus/english.json"
iterations := "6"
pinned := ""
config := "config/example.toml"

analyze:
    cargo run --release -- analyze --layout {{layout}} --corpus {{corpus}} --config {{config}}

optimize:
    cargo run --release -- optimize --layout {{layout}} --corpus {{corpus}} --config {{config}} --iterations {{iterations}} --pinned "{{pinned}}"

run: analyze
