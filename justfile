layout := "colemak-dh"
corpus := "./corpus/english.json"
iterations := "10000"
pinned := ""
config := "config/example.toml"
algorithm := "simulated_annealing"

analyze:
    cargo run --release -- analyze --layout {{layout}} --corpus {{corpus}} --config {{config}}

optimize:
    cargo run --release -- optimize --layout {{layout}} --corpus {{corpus}} --config {{config}} --algorithm {{algorithm}} --iterations {{iterations}} --pinned "{{pinned}}" --level debug

run: analyze
