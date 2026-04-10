use std::{collections::HashMap, fs, path::Path};

use clap::{Parser, Subcommand};

use layouts_rs::{
    analyzer::Analyzer,
    config::{Config, OptimizationConfig},
    corpus::Corpus,
    layout::Layout,
    optimizer::{self, Algorithm, HillClimbOptimizer, Optimizer, SimulatedAnnealingOptimizer},
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
    #[command(flatten)]
    run_options: RunOptions,
}

#[derive(Parser, Clone)]
struct RunOptions {
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
    #[arg(
        long,
        default_value = "false",
        help = "Whether to shuffle the layout before optimization"
    )]
    shuffle: bool,
}

impl From<RunOptions> for optimizer::RunOptions {
    fn from(options: RunOptions) -> Self {
        Self {
            iterations: options.iterations,
            seed: options.seed,
            pinned: options.pinned.chars().collect(),
            max_swapped: options.max_swapped,
            shuffle: options.shuffle,
        }
    }
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

        Ok(Corpus::new(corpus_map))
    }

    fn corpus(&self) -> Corpus {
        let mut aggregated: HashMap<String, f64> = HashMap::new();

        for corpus in &self.corpus {
            for (word, count) in &corpus.word_items {
                *aggregated.entry(word.clone()).or_insert(0.0) += *count;
            }
        }

        Corpus::new(aggregated)
    }
}

impl Command {
    fn run(&self) -> anyhow::Result<()> {
        match self {
            Command::Analyze(args) => {
                let config = Config::load(Path::new(&args.common.config))?;

                let layout = Layout::new(&args.common.layout, &config.layout)
                    .map_err(|e| anyhow::anyhow!("Failed to load layout: {e}"))?;

                let corpus = args.common.corpus();
                let analyzer = Analyzer::new(corpus);
                let optimizer = Self::select_optimizer(analyzer.clone(), &config.optimization);
                let score = optimizer.score(&layout);

                let mut report_metrics = ReportMetrics::default();
                analyzer.analyze(&layout, &mut report_metrics);
                let report = Report::from(report_metrics);

                println!("Initial Layout:");
                println!("{layout}");
                println!("Optimizer score: {score:.4}");
                println!("{report}");
            }
            Command::Optimize(args) => {
                let config = Config::load(Path::new(&args.common.config))?;

                let layout = Layout::new(&args.common.layout, &config.layout)
                    .map_err(|e| anyhow::anyhow!("Failed to load layout: {e}"))?;

                let corpus = args.common.corpus();
                let analyzer = Analyzer::new(corpus);
                let optimizer = Self::select_optimizer(analyzer.clone(), &config.optimization);
                let optimized_layout = optimizer.optimize(&layout, args.run_options.clone().into());

                let mut report_metrics = ReportMetrics::default();
                analyzer.analyze(&optimized_layout, &mut report_metrics);
                let report = Report::from(report_metrics);
                let score = optimizer.score(&optimized_layout);

                println!("Optimized Layout:");
                println!("{optimized_layout}");
                println!("Optimizer score: {score:.4}");
                println!("{report}");
            }
        }
        Ok(())
    }

    fn select_optimizer(
        analyzer: Analyzer,
        optimization: &OptimizationConfig,
    ) -> Box<dyn Optimizer> {
        match optimization.algorithm {
            Algorithm::HillClimb => Box::new(HillClimbOptimizer::new(
                analyzer,
                optimization.targets.clone(),
            )),
            Algorithm::SimulatedAnnealing => Box::new(SimulatedAnnealingOptimizer::new(
                analyzer,
                optimization.targets.clone(),
                optimization.simulated_annealing.clone(),
            )),
        }
    }
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    cli.command.run()?;
    Ok(())
}
