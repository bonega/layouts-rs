use std::{collections::HashMap, fs, path::Path};

use clap::{Parser, Subcommand};

use layouts_rs::{
    analyzer::Analyzer,
    config::{Config, LayoutConfig},
    corpus::Corpus,
    layout::Layout,
    optimizer::Optimizer,
    report::{Report, ReportMetrics},
};

#[derive(Parser)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    Analyze(AnalyzeArgs),
    Optimize(OptimizeArgs),
}

#[derive(Parser)]
struct AnalyzeArgs {
    #[command(flatten)]
    common: CommonConfig,
}

#[derive(Parser)]
struct OptimizeArgs {
    #[command(flatten)]
    common: CommonConfig,
    #[arg(long, default_value = "10", help = "Number of optimization iterations")]
    iterations: usize,
    #[arg(long, help = "Random seed for optimization")]
    seed: Option<u64>,
    #[arg(
        long,
        default_value = "",
        help = "Characters to pin in their original positions during optimization"
    )]
    pinned: String,
    #[arg(
        long,
        help = "Maximum number of keys to swap in each optimization iteration"
    )]
    max_swapped: Option<usize>,
}

#[derive(Parser)]
struct CommonConfig {
    #[arg(long)]
    config: String,
    #[arg(
        long,
        default_value = "qwerty",
        value_parser = CommonConfig::parse_layout_string,
        help = "Layout preset or custom layout string"
    )]
    layout: String,
    #[arg(
        long,
        value_name = "PATH",
        num_args = 1..,
        value_delimiter = ',',
        value_parser = CommonConfig::parse_corpus,
        help = "Paths to corpus JSON files"
    )]
    corpus: Vec<Corpus>,
}

impl CommonConfig {
    fn parse_layout_string(name: &str) -> Result<String, String> {
        let content = include_str!("../presets.toml");
        let presets: HashMap<String, String> =
            toml::from_str(content).expect("Failed to parse presets.toml");

        Ok(presets
            .get(name)
            .map(|s| s.to_string())
            .unwrap_or_else(|| name.to_string()))
    }

    fn parse_corpus(path: &str) -> Result<Corpus, String> {
        let content = fs::read_to_string(path)
            .map_err(|e| format!("Failed to read corpus file {}: {e}", path))?;

        let corpus_map: HashMap<String, f64> = serde_json::from_str(&content)
            .map_err(|e| format!("Failed to parse corpus file {}: {e}", path))?;

        Ok(Corpus::new(corpus_map.into_iter()))
    }

    fn corpus(&self) -> Corpus {
        let mut aggregated: HashMap<String, f64> = HashMap::new();

        for corpus in &self.corpus {
            for (word, count) in &corpus.word_items {
                *aggregated.entry(word.clone()).or_insert(0.0) += *count;
            }
        }

        Corpus::new(aggregated.into_iter())
    }
}

impl Command {
    fn run(&self) -> anyhow::Result<()> {
        match self {
            Command::Analyze(args) => {
                let config = Config::load(Path::new(&args.common.config))?;

                let layout = self.load_layout(&args.common.layout, &config.layout)?;

                let corpus = args.common.corpus();
                let analyzer = Analyzer::new(corpus);

                let mut report_metrics = ReportMetrics::default();
                analyzer.analyze(&layout, &mut report_metrics);
                let report = Report::from(report_metrics);

                println!("Initial Layout:");
                println!("{layout}");
                println!("{report}");
            }
            Command::Optimize(args) => {
                let config = Config::load(Path::new(&args.common.config))?;

                let layout = self.load_layout(&args.common.layout, &config.layout)?;

                let corpus = args.common.corpus();
                let analyzer = Analyzer::new(corpus);

                let optimizer =
                    Optimizer::new(analyzer.clone(), config.optimization.weights.into());
                let optimized_layout = optimizer.optimize(
                    &layout,
                    args.iterations,
                    args.seed,
                    &args.pinned.chars().collect(),
                    args.max_swapped,
                );

                let mut report_metrics = ReportMetrics::default();
                analyzer.analyze(&optimized_layout, &mut report_metrics);
                let report = Report::from(report_metrics);

                println!("Optimized Layout:");
                println!("{optimized_layout}");
                println!("{report}");
            }
        }
        Ok(())
    }

    fn load_layout(
        &self,
        layout_str: &str,
        config: &LayoutConfig,
    ) -> anyhow::Result<Layout<3, 12>> {
        Layout::<3, 12>::new(
            layout_str,
            config.finger_assignment.clone(),
            config.finger_effort.clone(),
            config.finger_home_positions.clone(),
        )
        .map_err(|e| anyhow::anyhow!("Failed to load layout: {e}"))
    }
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    cli.command.run()?;
    Ok(())
}
