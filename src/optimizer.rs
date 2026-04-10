use std::cmp::Ordering;
use std::collections::HashSet;
use std::fmt::Display;
use std::sync::Arc;

use rand::{Rng, prelude::*, rng, rngs::StdRng};
use rayon::prelude::*;
use serde::Deserialize;

use crate::{
    analyzer::{Analyzer, Metric, Metrics},
    layout::{FingerKind, Hand, Layout},
    matrix::Pos,
    ngrams::{BigramKind, TrigramKind},
    swaps::SwapMove,
};

#[derive(Deserialize, Default)]
pub struct Targets {
    pub effort: Target,
    pub left_hand_usage: Target,
    pub pinky_off_home: Target,
    pub bigram_skips_1: Target,
    pub bigram_skips_n: Target,
    pub bigram_lateral_stretches: Target,
    pub bigram_scissors: Target,
    pub bigram_wide_scissors: Target,
    pub trigram_same_hand_skips: Target,
    pub trigram_alternation_skips: Target,
    pub trigram_roll_ratio: Target,
    pub trigram_redirects_weak: Target,
    pub trigram_redirects_strong: Target,
    pub trigram_alternations: Target,
}

#[derive(Deserialize)]
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

#[derive(Default)]
struct OptimizerMetrics {
    total_chars: f64,
    effort: f64,
    left_hand: f64,
    pinky_off_home: f64,
    bigram_skips_1: f64,
    bigram_skips_n: f64,
    bigram_lateral_stretches: f64,
    bigram_scissors: f64,
    bigram_wide_scissors: f64,
    trigram_same_hand_skips: f64,
    trigram_alternation_skips: f64,
    trigram_roll_in: f64,
    trigram_roll_out: f64,
    trigram_redirects_weak: f64,
    trigram_redirects_strong: f64,
    trigram_alternations: f64,
}

struct OptimizerStats {
    effort: f64,
    left_hand_usage: f64,
    pinky_off_home: f64,
    bigram_skips_1: f64,
    bigram_skips_n: f64,
    bigram_lateral_stretches: f64,
    bigram_scissors: f64,
    bigram_wide_scissors: f64,
    trigram_same_hand_skips: f64,
    trigram_alternation_skips: f64,
    trigram_roll_ratio: f64,
    trigram_redirects_weak: f64,
    trigram_redirects_strong: f64,
    trigram_alternations: f64,
}

impl OptimizerStats {
    fn score(&self, targets: &Targets) -> f64 {
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
                .trigram_same_hand_skips
                .score(self.trigram_same_hand_skips)
            + targets
                .trigram_alternation_skips
                .score(self.trigram_alternation_skips)
            + targets.trigram_roll_ratio.score(self.trigram_roll_ratio)
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

impl From<OptimizerMetrics> for OptimizerStats {
    fn from(metrics: OptimizerMetrics) -> Self {
        let pct = 100.0 / metrics.total_chars;

        let effort = pct * metrics.effort;
        let left_hand_usage = pct * metrics.left_hand;
        let pinky_off_home = pct * metrics.pinky_off_home;
        let bigram_skips_1 = pct * metrics.bigram_skips_1;
        let bigram_skips_n = pct * metrics.bigram_skips_n;
        let bigram_lateral_stretches = pct * metrics.bigram_lateral_stretches;
        let bigram_scissors = pct * metrics.bigram_scissors;
        let bigram_wide_scissors = pct * metrics.bigram_wide_scissors;
        let trigram_same_hand_skips = pct * metrics.trigram_same_hand_skips;
        let trigram_alternation_skips = pct * metrics.trigram_alternation_skips;

        let total_roll = metrics.trigram_roll_in + metrics.trigram_roll_out;
        let trigram_roll_ratio = if total_roll > 0.0 {
            100.0 * metrics.trigram_roll_in / total_roll
        } else {
            50.0
        };

        let trigram_redirects_weak = pct * metrics.trigram_redirects_weak;
        let trigram_redirects_strong = pct * metrics.trigram_redirects_strong;
        let trigram_alternations = pct * metrics.trigram_alternations;

        Self {
            effort,
            left_hand_usage,
            pinky_off_home,
            bigram_skips_1,
            bigram_skips_n,
            bigram_lateral_stretches,
            bigram_scissors,
            bigram_wide_scissors,
            trigram_same_hand_skips,
            trigram_alternation_skips,
            trigram_roll_ratio,
            trigram_redirects_weak,
            trigram_redirects_strong,
            trigram_alternations,
        }
    }
}

impl Display for OptimizerStats {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Eff: {:.2}% LHu: {:.2}% PkOH: {:.2}% Sk1: {:.2}% SkN: {:.2}% \
             LS: {:.2}% Sci: {:.2}% WSci: {:.2}% | \
             SHSk: {:.2}% ASk: {:.2}% RRat: {:.2}% \
             RedW: {:.2}% RedS: {:.2}% Alt: {:.2}%",
            self.effort,
            self.left_hand_usage,
            self.pinky_off_home,
            self.bigram_skips_1,
            self.bigram_skips_n,
            self.bigram_lateral_stretches,
            self.bigram_scissors,
            self.bigram_wide_scissors,
            self.trigram_same_hand_skips,
            self.trigram_alternation_skips,
            self.trigram_roll_ratio,
            self.trigram_redirects_weak,
            self.trigram_redirects_strong,
            self.trigram_alternations,
        )
    }
}

impl Metrics for OptimizerMetrics {
    fn collect_metric(&mut self, metric: Metric) {
        match metric {
            Metric::CorpusLenght(chars) => {
                self.total_chars = chars;
            }
            Metric::Unigram(unigram, count) => {
                self.effort += unigram.key.effort * count;

                if unigram.key.finger.hand == Hand::Left {
                    self.left_hand += count;
                }

                if unigram.key.finger.kind == FingerKind::Pinky && !unigram.key.finger_home {
                    self.pinky_off_home += count;
                }
            }
            Metric::Bigram(bigram, count) => match bigram.kind {
                BigramKind::SameFingerSkip { skips } => {
                    if skips == 1 {
                        self.bigram_skips_1 += count;
                    } else {
                        self.bigram_skips_n += count;
                    }
                }
                BigramKind::LateralStretch { .. } => {
                    self.bigram_lateral_stretches += count;
                }
                BigramKind::Scissor {
                    col_distance,
                    row_distance,
                } => {
                    if col_distance + row_distance > 2 {
                        self.bigram_wide_scissors += count;
                    } else {
                        self.bigram_scissors += count;
                    }
                }
                BigramKind::Other => {}
            },
            Metric::Trigram(trigram, count) => match trigram.kind {
                TrigramKind::SameFingerSkip { same_hand, .. } => {
                    if same_hand {
                        self.trigram_same_hand_skips += count;
                    } else {
                        self.trigram_alternation_skips += count;
                    }
                }
                TrigramKind::Roll { triple, inward } => match (triple, inward) {
                    (true, true) => self.trigram_roll_in += count,
                    (true, false) => self.trigram_roll_out += count,
                    (false, true) | (false, false) => {}
                },
                TrigramKind::RollIn { triple } => {
                    if triple {
                        self.trigram_roll_in += count;
                    }
                }
                TrigramKind::RollOut { triple } => {
                    if triple {
                        self.trigram_roll_out += count;
                    }
                }
                TrigramKind::Redirect { weak } => {
                    if weak {
                        self.trigram_redirects_weak += count;
                    } else {
                        self.trigram_redirects_strong += count;
                    }
                }
                TrigramKind::Alternation => {
                    self.trigram_alternations += count;
                }
                TrigramKind::Other => {}
            },
        }
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
    pub fn new(layout: Layout, pinned_chars: HashSet<char>, max_swapped: Option<usize>) -> Self {
        let positions: Vec<Pos> = layout
            .keys()
            .filter(|key| !pinned_chars.contains(&key.ch))
            .map(|key| key.position)
            .collect();
        let swap_moves = Arc::new(SwapMove::all_moves(&positions));

        Self {
            initial_layout: layout.clone(),
            layout,
            max_swapped,
            swap_moves,
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

    fn perturb<RNG: Rng + ?Sized>(&mut self, rng: &mut RNG, attempts: usize) {
        let n = self.swap_moves.len();

        if n == 0 {
            return;
        }

        let mut applied = 0;
        for _ in 0..attempts {
            if applied >= 2.max(n / 4) {
                break;
            }

            let swap = &self.swap_moves[rng.next_u64() as usize % n];
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
    pub seed: Option<u64>,
    pub pinned: HashSet<char>,
    pub max_swapped: Option<usize>,
    pub shuffle: bool,
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

    fn get_stats(&self, layout: &Layout) -> OptimizerStats {
        let mut metrics = OptimizerMetrics::default();
        self.analyzer.analyze(layout, &mut metrics);
        OptimizerStats::from(metrics)
    }
}

impl Optimizer for HillClimbOptimizer {
    fn optimize(&self, layout: &Layout, opts: RunOptions) -> Layout {
        let mut rng: StdRng = if let Some(seed) = opts.seed {
            StdRng::seed_from_u64(seed)
        } else {
            StdRng::from_rng(&mut rng())
        };

        let mut best_layout = OptimizableLayout::new(layout.clone(), opts.pinned, opts.max_swapped);

        if opts.shuffle {
            best_layout.shuffle(&mut rng, 100);
        }

        let mut best_score = self.score(&best_layout.layout);

        for iteration in 0..opts.iterations {
            let mut candidate = best_layout.clone();

            if iteration > 0 {
                candidate.perturb(&mut rng, 10);
            }

            let mut current_score = self.score(&candidate.layout);

            while let Some(best_iteration_score) = candidate.try_improve(|layout| {
                let score = self.score(layout);
                (score < current_score).then_some(score)
            }) {
                current_score = best_iteration_score;
            }

            if current_score < best_score {
                best_score = current_score;
                best_layout = candidate;
            }
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
    fn it_optimizes() {
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
                seed: Some(42),
                pinned: HashSet::new(),
                max_swapped: None,
                shuffle: false,
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
        let mut optimizable = OptimizableLayout::new(make_layout(), [].into(), None);
        let result = optimizable.try_improve(|layout| {
            let score = layout_effort_score(layout);
            if score < 0.0 { Some(score) } else { None }
        });

        check!(result == None);
    }

    #[test]
    fn it_improves_layout_by_applying_the_best_swap() {
        let mut optimizable = OptimizableLayout::new(make_layout(), [].into(), None);
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
        let mut optimizable = OptimizableLayout::new(make_layout(), ['a', 'c'].into(), None);

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
        let mut optimizable = OptimizableLayout::new(make_layout(), ['a', 'b', 'c'].into(), None);
        let result = optimizable.try_improve(|layout| {
            let score = layout.key_for('d').unwrap().effort;
            if score < 200.0 { Some(score) } else { None }
        });

        check!(result == None);
    }

    #[test]
    fn it_does_not_improve_layout_when_swap_exceeds_max_swapped() {
        let mut optimizable = OptimizableLayout::new(make_layout(), [].into(), Some(0));
        let result = optimizable.try_improve(|layout| {
            let score = layout.key_for('c').unwrap().effort;
            if score < 100.0 { Some(score) } else { None }
        });

        check!(result == None);
    }

    #[test]
    fn it_improves_layout_when_swap_is_within_max_swapped() {
        let mut optimizable = OptimizableLayout::new(make_layout(), [].into(), Some(2));
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
        let mut optimizable = OptimizableLayout::new(make_layout(), [].into(), None);

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
        let mut optimizable = OptimizableLayout::new(make_layout(), [].into(), None);
        let before: Vec<char> = optimizable.layout().keys().map(|k| k.ch).collect();

        let mut rng = StdRng::seed_from_u64(42);
        optimizable.perturb(&mut rng, 10);

        let after: Vec<char> = optimizable.layout().keys().map(|k| k.ch).collect();
        check!(before != after);
    }

    #[test]
    fn it_does_not_perturb_layout_when_all_chars_are_pinned() {
        let mut optimizable =
            OptimizableLayout::new(make_layout(), ['a', 'b', 'c', 'd'].into(), None);
        let before: Vec<char> = optimizable.layout().keys().map(|k| k.ch).collect();

        let mut rng = StdRng::seed_from_u64(42);
        optimizable.perturb(&mut rng, 10);

        let after: Vec<char> = optimizable.layout().keys().map(|k| k.ch).collect();
        check!(before == after);
    }

    #[test]
    fn it_does_not_perturb_layout_beyond_max_swapped() {
        let mut optimizable = OptimizableLayout::new(make_layout(), [].into(), Some(0));

        let mut rng = StdRng::seed_from_u64(42);
        optimizable.perturb(&mut rng, 10);

        check!(optimizable.diff() == 0);
    }

    #[test]
    fn it_perturbs_layout_within_max_swapped() {
        let mut optimizable = OptimizableLayout::new(make_layout(), [].into(), Some(4));

        let mut rng = StdRng::seed_from_u64(42);
        optimizable.perturb(&mut rng, 10);

        check!(optimizable.diff() > 0);
        check!(optimizable.diff() <= 4);
    }

    #[test]
    fn it_shuffles_layoyt_ignoring_max_swapped() {
        let mut optimizable = OptimizableLayout::new(make_layout(), [].into(), Some(0));
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
mod optimizer_metrics_tests {
    use assert2::check;
    use rstest::rstest;

    use super::*;
    use crate::{
        analyzer::Metric,
        layout::{Layout, fixtures::qwerty},
        ngrams::{Bigram, Trigram, Unigram},
    };

    #[rstest]
    fn it_collects_unigram_metrics(qwerty: Layout) {
        let mut metrics = OptimizerMetrics::default();

        metrics.collect_metric(Metric::CorpusLenght(100.0));
        metrics.collect_metric(Metric::Unigram(
            Unigram::new(qwerty.key_for('a').unwrap()),
            1.0,
        ));
        metrics.collect_metric(Metric::Unigram(
            Unigram::new(qwerty.key_for('q').unwrap()),
            2.0,
        ));
        metrics.collect_metric(Metric::Unigram(
            Unigram::new(qwerty.key_for('z').unwrap()),
            1.0,
        ));
        metrics.collect_metric(Metric::Unigram(
            Unigram::new(qwerty.key_for('"').unwrap()),
            1.0,
        ));

        check!(metrics.total_chars == 100.0);
        check!(metrics.effort == 9.0);
        check!(metrics.left_hand == 5.0);
        check!(metrics.pinky_off_home == 4.0);
    }

    #[rstest]
    fn it_collects_bigram_skips_and_stretches(qwerty: Layout) {
        let mut metrics = OptimizerMetrics::default();

        metrics.collect_metric(Metric::Bigram(
            Bigram::new(qwerty.key_for('q').unwrap(), qwerty.key_for('a').unwrap()),
            10.0,
        ));
        metrics.collect_metric(Metric::Bigram(
            Bigram::new(qwerty.key_for('q').unwrap(), qwerty.key_for('z').unwrap()),
            20.0,
        ));
        metrics.collect_metric(Metric::Bigram(
            Bigram::new(qwerty.key_for('d').unwrap(), qwerty.key_for('g').unwrap()),
            10.0,
        ));
        metrics.collect_metric(Metric::Bigram(
            Bigram::new(qwerty.key_for('s').unwrap(), qwerty.key_for('"').unwrap()),
            20.0,
        ));

        check!(metrics.bigram_skips_1 == 10.0);
        check!(metrics.bigram_skips_n == 20.0);
        check!(metrics.bigram_lateral_stretches == 30.0);
    }

    #[rstest]
    fn it_collects_bigram_scissors(qwerty: Layout) {
        let mut metrics = OptimizerMetrics::default();

        metrics.collect_metric(Metric::Bigram(
            Bigram::new(qwerty.key_for('d').unwrap(), qwerty.key_for('t').unwrap()),
            10.0,
        ));
        metrics.collect_metric(Metric::Bigram(
            Bigram::new(qwerty.key_for('d').unwrap(), qwerty.key_for('r').unwrap()),
            20.0,
        ));

        check!(metrics.bigram_wide_scissors == 10.0);
        check!(metrics.bigram_scissors == 20.0);
    }

    #[rstest]
    fn it_collects_trigram_skips(qwerty: Layout) {
        let mut metrics = OptimizerMetrics::default();

        metrics.collect_metric(Metric::Trigram(
            Trigram::new(
                qwerty.key_for('q').unwrap(),
                qwerty.key_for('w').unwrap(),
                qwerty.key_for('a').unwrap(),
            ),
            10.0,
        ));
        metrics.collect_metric(Metric::Trigram(
            Trigram::new(
                qwerty.key_for('q').unwrap(),
                qwerty.key_for('h').unwrap(),
                qwerty.key_for('a').unwrap(),
            ),
            20.0,
        ));

        check!(metrics.trigram_same_hand_skips == 10.0);
        check!(metrics.trigram_alternation_skips == 20.0);
    }

    #[rstest]
    fn it_collects_trigram_rolls(qwerty: Layout) {
        let mut metrics = OptimizerMetrics::default();

        metrics.collect_metric(Metric::Trigram(
            Trigram::new(
                qwerty.key_for('q').unwrap(),
                qwerty.key_for('w').unwrap(),
                qwerty.key_for('e').unwrap(),
            ),
            10.0,
        ));
        metrics.collect_metric(Metric::Trigram(
            Trigram::new(
                qwerty.key_for('t').unwrap(),
                qwerty.key_for('e').unwrap(),
                qwerty.key_for('q').unwrap(),
            ),
            20.0,
        ));
        metrics.collect_metric(Metric::Trigram(
            Trigram::new(
                qwerty.key_for('q').unwrap(),
                qwerty.key_for('w').unwrap(),
                qwerty.key_for('p').unwrap(),
            ),
            30.0,
        ));
        metrics.collect_metric(Metric::Trigram(
            Trigram::new(
                qwerty.key_for('t').unwrap(),
                qwerty.key_for('e').unwrap(),
                qwerty.key_for('p').unwrap(),
            ),
            40.0,
        ));

        check!(metrics.trigram_roll_in == 10.0);
        check!(metrics.trigram_roll_out == 20.0);
    }

    #[rstest]
    fn it_collects_trigram_redirects_and_alternations(qwerty: Layout) {
        let mut metrics = OptimizerMetrics::default();

        metrics.collect_metric(Metric::Trigram(
            Trigram::new(
                qwerty.key_for('q').unwrap(),
                qwerty.key_for('t').unwrap(),
                qwerty.key_for('e').unwrap(),
            ),
            10.0,
        ));
        metrics.collect_metric(Metric::Trigram(
            Trigram::new(
                qwerty.key_for('q').unwrap(),
                qwerty.key_for('e').unwrap(),
                qwerty.key_for('w').unwrap(),
            ),
            20.0,
        ));
        metrics.collect_metric(Metric::Trigram(
            Trigram::new(
                qwerty.key_for('q').unwrap(),
                qwerty.key_for('h').unwrap(),
                qwerty.key_for('w').unwrap(),
            ),
            30.0,
        ));

        check!(metrics.trigram_redirects_weak == 10.0);
        check!(metrics.trigram_redirects_strong == 20.0);
        check!(metrics.trigram_alternations == 30.0);
    }
}

#[cfg(test)]
mod optimizer_stats_tests {
    use assert2::check;

    use super::*;

    #[test]
    fn it_builds_stats_from_metrics_and_targets() {
        let metrics = OptimizerMetrics {
            total_chars: 200.0,
            effort: 10.0,
            left_hand: 20.0,
            pinky_off_home: 30.0,
            bigram_skips_1: 40.0,
            bigram_skips_n: 50.0,
            bigram_lateral_stretches: 60.0,
            bigram_scissors: 70.0,
            bigram_wide_scissors: 80.0,
            trigram_same_hand_skips: 90.0,
            trigram_alternation_skips: 100.0,
            trigram_roll_in: 10.0,
            trigram_roll_out: 30.0,
            trigram_redirects_weak: 130.0,
            trigram_redirects_strong: 140.0,
            trigram_alternations: 150.0,
        };

        let stats = OptimizerStats::from(metrics);

        check!(stats.effort == 5.0);
        check!(stats.left_hand_usage == 10.0);
        check!(stats.pinky_off_home == 15.0);
        check!(stats.bigram_skips_1 == 20.0);
        check!(stats.bigram_skips_n == 25.0);
        check!(stats.bigram_lateral_stretches == 30.0);
        check!(stats.bigram_scissors == 35.0);
        check!(stats.bigram_wide_scissors == 40.0);
        check!(stats.trigram_same_hand_skips == 45.0);
        check!(stats.trigram_alternation_skips == 50.0);
        check!(stats.trigram_roll_ratio == 25.0);
        check!(stats.trigram_redirects_weak == 65.0);
        check!(stats.trigram_redirects_strong == 70.0);
        check!(stats.trigram_alternations == 75.0);
    }

    #[test]
    fn it_gives_the_right_score() {
        let stats = OptimizerStats {
            effort: 10.0,
            left_hand_usage: 10.0,
            pinky_off_home: 10.0,
            bigram_skips_1: 10.0,
            bigram_skips_n: 10.0,
            bigram_lateral_stretches: 10.0,
            bigram_scissors: 10.0,
            bigram_wide_scissors: 10.0,
            trigram_same_hand_skips: 10.0,
            trigram_alternation_skips: 10.0,
            trigram_roll_ratio: 10.0,
            trigram_redirects_weak: 10.0,
            trigram_redirects_strong: 10.0,
            trigram_alternations: 10.0,
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
            trigram_same_hand_skips: optimizer_target!(20.0, 9.0),
            trigram_alternation_skips: optimizer_target!(20.0, 10.0),
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
