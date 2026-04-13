use std::cmp::Ordering;
use std::collections::HashSet;
use std::fmt;
use std::sync::Arc;

use log::{debug, info};
use rand::{Rng, prelude::*, rngs::StdRng};
use rayon::prelude::*;
use serde::Deserialize;

use crate::{
    analyzer::Analyzer,
    layout::Layout,
    matrix::Pos,
    metrics::SimpleMetrics,
    stats::SimpleStats,
    swaps::{SwapMove, SwapMoveBuilder, SwapMoveStrategy},
};

const MAX_PERTURB_ATTEMPTS: usize = 30;

#[derive(Debug, Deserialize, Clone)]
pub struct SimulatedAnnealingConfig {
    pub init_temp: f64,
    pub cooling: f64,
    pub key_switches: usize,
    pub stall_accepted: usize,
}

#[derive(Deserialize, Default, Clone)]
pub struct Targets {
    pub effort: Target,
    pub left_hand_usage: Target,
    pub pinky_off_home: Target,
    pub bigram_skips_1: Target,
    pub bigram_skips_n: Target,
    pub bigram_lateral_stretches: Target,
    pub bigram_scissors: Target,
    pub bigram_wide_scissors: Target,
    pub trigram_skips_same_hand: Target,
    pub trigram_skips_alternation: Target,
    pub trigram_roll_ratio: Target,
    pub trigram_redirects_weak: Target,
    pub trigram_redirects_strong: Target,
    pub trigram_alternations: Target,
}

#[derive(Deserialize, Clone)]
pub struct Target {
    pub value: f64,
    pub weight: f64,
    #[serde(default = "default_scale")]
    pub scale: f64,
}

impl Default for Target {
    fn default() -> Self {
        Self {
            value: 0.0,
            weight: 0.0,
            scale: 1.0,
        }
    }
}

fn default_scale() -> f64 {
    1.0
}

impl Target {
    pub fn score(&self, current_value: f64) -> f64 {
        self.weight * ((current_value - self.value).abs() / self.scale)
    }
}

impl SimpleStats {
    pub fn score(&self, targets: &Targets) -> f64 {
        targets.effort.score(self.effort)
            + targets.left_hand_usage.score(self.left_hand_usage)
            + targets.pinky_off_home.score(self.pinky_off_home)
            + targets.bigram_skips_1.score(self.bigram_skips_1)
            + targets.bigram_skips_n.score(self.bigram_skips_n)
            + targets
                .bigram_lateral_stretches
                .score(self.bigram_lateral_stretches)
            + targets.bigram_scissors.score(self.bigram_scissors)
            + targets
                .bigram_wide_scissors
                .score(self.bigram_wide_scissors)
            + targets
                .trigram_skips_same_hand
                .score(self.trigram_skips_same_hand)
            + targets
                .trigram_skips_alternation
                .score(self.trigram_skips_alternation)
            + targets.trigram_roll_ratio.score(self.trigram_roll_ratio())
            + targets
                .trigram_redirects_weak
                .score(self.trigram_redirects_weak)
            + targets
                .trigram_redirects_strong
                .score(self.trigram_redirects_strong)
            + targets
                .trigram_alternations
                .score(self.trigram_alternations)
    }
}

#[derive(Clone)]
struct OptimizableLayout {
    initial_layout: Layout,
    layout: Layout,
    max_swapped: Option<usize>,
    swap_moves: Arc<Vec<SwapMove>>,
}

impl OptimizableLayout {
    pub fn new(
        layout: Layout,
        pinned_chars: HashSet<char>,
        max_swapped: Option<usize>,
        swap_move_builder: SwapMoveBuilder,
    ) -> Self {
        let positions: Vec<Pos> = layout
            .keys()
            .filter(|key| !pinned_chars.contains(&key.ch))
            .map(|key| key.position)
            .collect();

        Self {
            initial_layout: layout.clone(),
            layout,
            max_swapped,
            swap_moves: Arc::new(swap_move_builder.build(&positions)),
        }
    }

    fn diff(&self) -> usize {
        Self::diff_between(&self.layout, &self.initial_layout)
    }

    fn diff_between(layout: &Layout, initial_layout: &Layout) -> usize {
        layout
            .keys()
            .zip(initial_layout.keys())
            .filter(|(k1, k2)| k1.ch != k2.ch)
            .count()
    }

    fn try_improve(&mut self, score_check: impl Fn(&Layout) -> Option<f64> + Sync) -> Option<f64> {
        let initial_layout = &self.initial_layout;
        let current_layout = &self.layout;

        let (score, best_swap) = self
            .swap_moves
            .par_iter()
            .filter_map(|swap_move| {
                let mut candidate_layout = current_layout.clone();
                swap_move.apply(&mut candidate_layout);

                let score = if let Some(max) = self.max_swapped {
                    if Self::diff_between(&candidate_layout, initial_layout) <= max {
                        score_check(&candidate_layout)
                    } else {
                        None
                    }
                } else {
                    score_check(&candidate_layout)
                };

                score.map(|score| (score, swap_move))
            })
            .min_by(|(s1, _), (s2, _)| s1.partial_cmp(s2).unwrap_or(Ordering::Equal))?;

        best_swap.apply(&mut self.layout);

        Some(score)
    }

    fn shuffle<RNG: Rng + ?Sized>(&mut self, rng: &mut RNG, size: usize) {
        let n = self.swap_moves.len();

        if n == 0 {
            return;
        }

        for _ in 0..size {
            let swap = &self.swap_moves[rng.next_u64() as usize % n];
            swap.apply(&mut self.layout);
        }
    }

    pub fn perturb<RNG: Rng + ?Sized>(&mut self, rng: &mut RNG, mut n: usize) {
        let swaps_number = self.swap_moves.len();
        n = n.min(swaps_number);

        let mut applied = 0;
        for _ in 0..MAX_PERTURB_ATTEMPTS {
            if applied >= n {
                break;
            }

            let swap = &self.swap_moves[rng.next_u64() as usize % swaps_number];
            swap.apply(&mut self.layout);

            if let Some(max) = self.max_swapped
                && self.diff() > max
            {
                swap.apply(&mut self.layout);
            } else {
                applied += 1;
            }
        }
    }

    fn layout(&self) -> &Layout {
        &self.layout
    }
}

pub struct RunOptions {
    pub iterations: usize,
    pub seed: u64,
    pub pinned: HashSet<char>,
    pub max_swapped: Option<usize>,
    pub shuffle: bool,
}

impl fmt::Display for RunOptions {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "iterations: {}, seed: {}, pinned: {:?}, max_swapped: {:?}, shuffle: {}",
            self.iterations, self.seed, self.pinned, self.max_swapped, self.shuffle
        )
    }
}

pub trait Optimizer {
    fn optimize(&self, layout: &Layout, opts: RunOptions) -> Layout;
    fn score(&self, layout: &Layout) -> f64;
}

pub struct HillClimbOptimizer {
    analyzer: Analyzer,
    targets: Targets,
}

impl HillClimbOptimizer {
    pub fn new(analyzer: Analyzer, targets: Targets) -> Self {
        Self { analyzer, targets }
    }

    fn get_stats(&self, layout: &Layout) -> SimpleStats {
        let mut metrics = SimpleMetrics::default();
        self.analyzer.analyze(layout, &mut metrics);
        SimpleStats::from(metrics)
    }
}

impl Optimizer for HillClimbOptimizer {
    fn optimize(&self, layout: &Layout, opts: RunOptions) -> Layout {
        let mut rng: StdRng = StdRng::seed_from_u64(opts.seed);

        info!("Starting optimization with options: {opts}");

        let mut best_layout = OptimizableLayout::new(
            layout.clone(),
            opts.pinned,
            opts.max_swapped,
            SwapMoveBuilder::full(),
        );

        if opts.shuffle {
            best_layout.shuffle(&mut rng, 100);
        }

        let mut best_score = self.score(&best_layout.layout);

        for iteration in 0..opts.iterations {
            let mut candidate = best_layout.clone();

            if iteration > 0 {
                candidate.perturb(&mut rng, 2.max(candidate.swap_moves.len() / 4));
            }

            let mut current_score = self.score(&candidate.layout);

            let mut step = 0;
            while let Some(best_iteration_score) = candidate.try_improve(|layout| {
                let score = self.score(layout);
                (score < current_score).then_some(score)
            }) {
                current_score = best_iteration_score;
                debug!("Step {step}, score: {current_score}");
                step += 1;
            }

            if current_score < best_score {
                best_score = current_score;
                best_layout = candidate;
            }

            info!("Iteration {iteration}, best score: {best_score}");
        }

        best_layout.layout().clone()
    }

    fn score(&self, layout: &Layout) -> f64 {
        self.get_stats(layout).score(&self.targets)
    }
}

pub struct SimulatedAnnealingOptimizer {
    analyzer: Analyzer,
    targets: Targets,
    init_temp: f64,
    cooling: f64,
    stall_accepted: usize,
    key_switches: usize,
}

impl SimulatedAnnealingOptimizer {
    pub fn new(analyzer: Analyzer, targets: Targets, config: SimulatedAnnealingConfig) -> Self {
        Self {
            analyzer,
            targets,
            init_temp: config.init_temp,
            cooling: config.cooling,
            stall_accepted: config.stall_accepted,
            key_switches: config.key_switches.max(1),
        }
    }

    fn get_stats(&self, layout: &Layout) -> SimpleStats {
        let mut metrics = SimpleMetrics::default();
        self.analyzer.analyze(layout, &mut metrics);
        SimpleStats::from(metrics)
    }
}

impl Optimizer for SimulatedAnnealingOptimizer {
    fn optimize(&self, layout: &Layout, opts: RunOptions) -> Layout {
        let mut rng: StdRng = StdRng::seed_from_u64(opts.seed);

        info!("Starting optimization with options: {opts}");

        let mut best_layout = OptimizableLayout::new(
            layout.clone(),
            opts.pinned,
            opts.max_swapped,
            SwapMoveBuilder::new(&[SwapMoveStrategy::Single]),
        );

        if opts.shuffle {
            best_layout.shuffle(&mut rng, 100);
        }

        let mut best_score = self.score(&best_layout.layout);
        let mut current = best_layout.clone();
        let mut current_score = best_score;

        let mut temp = self.init_temp.max(1e-9);
        let mut stall = 0usize;

        for iteration in 0..opts.iterations {
            if current.swap_moves.is_empty() {
                break;
            }

            let mut candidate = current.clone();
            candidate.perturb(&mut rng, self.key_switches);

            let candidate_score = self.score(&candidate.layout);
            let delta = candidate_score - current_score;

            let accept = if delta <= 0.0 {
                true
            } else {
                let prob = (-delta / temp).exp();
                let random = rng.next_u64() as f64 / u64::MAX as f64;
                random < prob
            };

            if accept {
                if candidate_score < best_score {
                    best_score = candidate_score;
                    best_layout = candidate.clone();
                }
                current = candidate;
                current_score = candidate_score;
                stall = 0;
            } else {
                stall += 1;
            }

            info!("Iteration {iteration}, best score: {best_score}");

            temp *= self.cooling;

            if stall >= self.stall_accepted {
                break;
            }
        }

        while let Some(new_score) = best_layout.try_improve(|layout| {
            let score = self.score(layout);
            (score < best_score).then_some(score)
        }) {
            best_score = new_score;
        }

        best_layout.layout().clone()
    }

    fn score(&self, layout: &Layout) -> f64 {
        self.get_stats(layout).score(&self.targets)
    }
}

#[cfg(test)]
mod optimizer_tests {
    use super::*;
    use crate::{corpus::Corpus, layout::Config};
    use assert2::check;

    #[test]
    fn it_optimizes_with_hill_climbing() {
        let layout = Layout::new(
            "ab\ncd",
            &Config {
                finger_assignment: matrix!([[1, 2], [1, 2]]),
                finger_effort: matrix!([[1.0, 100.0], [100.0, 100.0]]),
                finger_home_positions: [(1, pos!(0, 0)), (2, pos!(0, 1))].into(),
            },
        )
        .unwrap();

        let corpus = Corpus::new([("c".to_string(), 10.0)]);
        let analyzer = Analyzer::new(corpus);
        let optimizer = HillClimbOptimizer::new(
            analyzer,
            Targets {
                effort: Target {
                    value: 0.0,
                    weight: 1.0,
                    scale: 1.0,
                },
                ..Default::default()
            },
        );

        let optimized_layout = optimizer.optimize(
            &layout,
            RunOptions {
                iterations: 10,
                seed: 42,
                pinned: HashSet::new(),
                max_swapped: None,
                shuffle: false,
            },
        );

        check!(optimized_layout.key_for('c').unwrap().effort == 1.0);
    }

    #[test]
    fn it_optimizes_with_simulated_annealing() {
        let layout = Layout::new(
            "ab\ncd",
            &Config {
                finger_assignment: matrix!([[1, 2], [1, 2]]),
                finger_effort: matrix!([[1.0, 100.0], [100.0, 100.0]]),
                finger_home_positions: [(1, pos!(0, 0)), (2, pos!(0, 1))].into(),
            },
        )
        .unwrap();

        let corpus = Corpus::new([("c".to_string(), 10.0)]);
        let analyzer = Analyzer::new(corpus);
        let optimizer = SimulatedAnnealingOptimizer::new(
            analyzer,
            Targets {
                effort: Target {
                    value: 0.0,
                    weight: 1.0,
                    scale: 1.0,
                },
                ..Default::default()
            },
            SimulatedAnnealingConfig {
                key_switches: 2,
                init_temp: 100.0,
                cooling: 0.95,
                stall_accepted: 100,
            },
        );

        let optimized_layout = optimizer.optimize(
            &layout,
            RunOptions {
                iterations: 1000,
                seed: 42,
                pinned: HashSet::new(),
                max_swapped: None,
                shuffle: true,
            },
        );

        check!(optimized_layout.key_for('c').unwrap().effort == 1.0);
    }
}

#[cfg(test)]
mod optimizable_layout_tests {
    use crate::layout::Config;

    use super::*;
    use assert2::check;

    fn make_layout() -> Layout {
        Layout::new(
            "ab\ncd",
            &Config {
                finger_assignment: matrix!([[1, 2], [1, 2]]),
                finger_effort: matrix!([[1.0, 50.0], [100.0, 200.0]]),
                finger_home_positions: [(1, pos!(0, 0)), (2, pos!(0, 1))].into(),
            },
        )
        .unwrap()
    }

    #[test]
    fn it_does_not_improve_layout_when_no_swap_gives_better_score() {
        let mut optimizable =
            OptimizableLayout::new(make_layout(), [].into(), None, SwapMoveBuilder::full());
        let result = optimizable.try_improve(|layout| {
            let score = layout_effort_score(layout);
            if score < 0.0 { Some(score) } else { None }
        });

        check!(result == None);
    }

    #[test]
    fn it_does_not_improve_layout_if_no_swaps_available() {
        let mut optimizable =
            OptimizableLayout::new(make_layout(), [].into(), None, SwapMoveBuilder::default());
        let result = optimizable.try_improve(|layout| {
            let score = layout.key_for('c').unwrap().effort;
            if score < 100.0 { Some(score) } else { None }
        });

        check!(result == None);
    }

    #[test]
    fn it_improves_layout_by_applying_the_best_swap() {
        let mut optimizable =
            OptimizableLayout::new(make_layout(), [].into(), None, SwapMoveBuilder::full());
        let score = optimizable
            .try_improve(|layout| {
                let score = layout.key_for('c').unwrap().effort;
                if score < 100.0 { Some(score) } else { None }
            })
            .unwrap();

        check!(score == 1.0);
        check!(optimizable.layout().key_for('c').unwrap().effort == 1.0);
    }

    #[test]
    fn it_improves_layout_without_moving_pinned_chars() {
        let mut optimizable = OptimizableLayout::new(
            make_layout(),
            ['a', 'c'].into(),
            None,
            SwapMoveBuilder::full(),
        );

        while optimizable
            .try_improve(|layout| {
                let score = layout.key_for('d').unwrap().effort;
                if score < 200.0 { Some(score) } else { None }
            })
            .is_some()
        {}

        check!(optimizable.layout().key_for('a').unwrap().position == pos!(0, 0));
        check!(optimizable.layout().key_for('c').unwrap().position == pos!(1, 0));
        check!(optimizable.layout().key_for('d').unwrap().position == pos!(0, 1));
    }

    #[test]
    fn it_does_not_improve_layout_when_only_one_char_is_unpinned() {
        let mut optimizable = OptimizableLayout::new(
            make_layout(),
            ['a', 'b', 'c'].into(),
            None,
            SwapMoveBuilder::full(),
        );
        let result = optimizable.try_improve(|layout| {
            let score = layout.key_for('d').unwrap().effort;
            if score < 200.0 { Some(score) } else { None }
        });

        check!(result == None);
    }

    #[test]
    fn it_does_not_improve_layout_when_swap_exceeds_max_swapped() {
        let mut optimizable =
            OptimizableLayout::new(make_layout(), [].into(), Some(0), SwapMoveBuilder::full());
        let result = optimizable.try_improve(|layout| {
            let score = layout.key_for('c').unwrap().effort;
            if score < 100.0 { Some(score) } else { None }
        });

        check!(result == None);
    }

    #[test]
    fn it_improves_layout_when_swap_is_within_max_swapped() {
        let mut optimizable =
            OptimizableLayout::new(make_layout(), [].into(), Some(2), SwapMoveBuilder::full());
        let score = optimizable
            .try_improve(|layout| {
                let score = layout.key_for('c').unwrap().effort;
                if score < 100.0 { Some(score) } else { None }
            })
            .unwrap();

        check!(score == 1.0);
        check!(optimizable.layout().key_for('c').unwrap().effort == 1.0);
    }

    #[test]
    fn it_does_not_improve_layout_after_convergence() {
        let mut optimizable =
            OptimizableLayout::new(make_layout(), [].into(), None, SwapMoveBuilder::full());

        let first = optimizable.try_improve(|layout| {
            let score = layout.key_for('c').unwrap().effort;
            if score < 100.0 { Some(score) } else { None }
        });
        check!(first == Some(1.0));

        let second = optimizable.try_improve(|layout| {
            let score = layout.key_for('c').unwrap().effort;
            if score < 1.0 { Some(score) } else { None }
        });
        check!(second == None);
    }

    #[test]
    fn it_perturbs_layout() {
        let mut optimizable =
            OptimizableLayout::new(make_layout(), [].into(), None, SwapMoveBuilder::full());
        let before: Vec<char> = optimizable.layout().keys().map(|k| k.ch).collect();

        let mut rng = StdRng::seed_from_u64(42);
        optimizable.perturb(&mut rng, 10);

        let after: Vec<char> = optimizable.layout().keys().map(|k| k.ch).collect();
        check!(before != after);
    }

    #[test]
    fn it_does_not_perturb_layout_when_all_chars_are_pinned() {
        let mut optimizable = OptimizableLayout::new(
            make_layout(),
            ['a', 'b', 'c', 'd'].into(),
            None,
            SwapMoveBuilder::full(),
        );
        let before: Vec<char> = optimizable.layout().keys().map(|k| k.ch).collect();

        let mut rng = StdRng::seed_from_u64(42);
        optimizable.perturb(&mut rng, 10);

        let after: Vec<char> = optimizable.layout().keys().map(|k| k.ch).collect();
        check!(before == after);
    }

    #[test]
    fn it_does_not_perturb_layout_beyond_max_swapped() {
        let mut optimizable =
            OptimizableLayout::new(make_layout(), [].into(), Some(0), SwapMoveBuilder::full());

        let mut rng = StdRng::seed_from_u64(42);
        optimizable.perturb(&mut rng, 10);

        check!(optimizable.diff() == 0);
    }

    #[test]
    fn it_perturbs_layout_within_max_swapped() {
        let mut optimizable =
            OptimizableLayout::new(make_layout(), [].into(), Some(4), SwapMoveBuilder::full());

        let mut rng = StdRng::seed_from_u64(42);
        optimizable.perturb(&mut rng, 10);

        check!(optimizable.diff() > 0);
        check!(optimizable.diff() <= 4);
    }

    #[test]
    fn it_shuffles_layoyt_ignoring_max_swapped() {
        let mut optimizable =
            OptimizableLayout::new(make_layout(), [].into(), Some(0), SwapMoveBuilder::full());
        let before: Vec<char> = optimizable.layout().keys().map(|k| k.ch).collect();

        let mut rng = StdRng::seed_from_u64(42);
        optimizable.shuffle(&mut rng, 10);

        let after: Vec<char> = optimizable.layout().keys().map(|k| k.ch).collect();
        check!(before != after);
    }

    fn layout_effort_score(layout: &Layout) -> f64 {
        layout.keys().map(|k| k.effort).sum()
    }
}

#[cfg(test)]
mod tests {
    use assert2::check;

    use super::*;

    #[test]
    fn it_gives_the_right_score() {
        let stats = SimpleStats {
            total_chars: 10.0,
            effort: 10.0,
            left_hand_usage: 10.0,
            right_hand_usage: 90.0,
            pinky_off_home: 10.0,
            bigram_skips_1: 10.0,
            bigram_skips_n: 10.0,
            bigram_lateral_stretches: 10.0,
            bigram_scissors: 10.0,
            bigram_wide_scissors: 10.0,
            bigram_others: 0.0,
            trigram_skips_same_hand: 10.0,
            trigram_skips_same_hand_1: 0.0,
            trigram_skips_same_hand_n: 0.0,
            trigram_skips_alternation: 10.0,
            trigram_skips_alternation_1: 0.0,
            trigram_skips_alternation_n: 0.0,
            trigram_lateral_stretches_same_hand: 0.0,
            trigram_lateral_stretches_alternation: 0.0,
            trigram_scissors_same_hand_1: 0.0,
            trigram_scissors_same_hand_n: 0.0,
            trigram_scissors_alternation_1: 0.0,
            trigram_scissors_alternation_n: 0.0,
            trigram_roll_in: 10.0,
            trigram_roll_out: 90.0,
            trigram_roll_in_bigrams: 20.0,
            trigram_roll_out_bigrams: 40.0,
            trigram_redirects_weak: 10.0,
            trigram_redirects_strong: 10.0,
            trigram_alternations: 10.0,
            trigram_others: 0.0,
            finger_usage: [].into(),
            row_usage: [].into(),
            column_usage: [].into(),
        };

        let targets = Targets {
            effort: optimizer_target!(20.0, 1.0),
            left_hand_usage: optimizer_target!(20.0, 2.0),
            pinky_off_home: optimizer_target!(20.0, 3.0),
            bigram_skips_1: optimizer_target!(20.0, 4.0),
            bigram_skips_n: optimizer_target!(20.0, 5.0),
            bigram_lateral_stretches: optimizer_target!(20.0, 6.0),
            bigram_scissors: optimizer_target!(20.0, 7.0),
            bigram_wide_scissors: optimizer_target!(20.0, 8.0),
            trigram_skips_same_hand: optimizer_target!(20.0, 9.0),
            trigram_skips_alternation: optimizer_target!(20.0, 10.0),
            trigram_roll_ratio: optimizer_target!(20.0, 11.0),
            trigram_redirects_weak: optimizer_target!(20.0, 12.0),
            trigram_redirects_strong: optimizer_target!(20.0, 13.0),
            trigram_alternations: optimizer_target!(20.0, 14.0),
        };

        let n = 14.0;
        let expected = 10.0 * n * (n + 1.0) / 2.0;
        check!(stats.score(&targets) == expected);
    }
}
