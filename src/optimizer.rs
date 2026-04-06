use std::cmp::Ordering;
use std::collections::HashSet;
use std::fmt::Display;
use std::sync::Arc;

use rand::{Rng, prelude::*, rng, rngs::StdRng};

use crate::{
    analyzer::{Analyzer, Metric, Metrics},
    layout::{Layout, Pos},
    swaps::SwapMove,
};

pub struct Weights {
    pub effort: f64,
}

#[derive(Default)]
struct OptimizerMetrics {
    total_chars: f64,
    effort: f64,
}

struct OptimizerStats {
    effort: f64,
    score: f64,
}

impl OptimizerStats {
    fn from(metrics: OptimizerMetrics, weights: &Weights) -> Self {
        Self {
            effort: metrics.effort,
            score: weights.effort * metrics.effort / metrics.total_chars,
        }
    }
}

impl Display for OptimizerStats {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Effort: {:.2}, Score: {:.4}", self.effort, self.score)
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
            }
            _ => {}
        }
    }
}

pub struct Optimizer {
    analyzer: Analyzer,
    weights: Weights,
}

#[derive(Clone)]
struct OptimizableLayout<const C: usize, const R: usize> {
    initial_layout: Layout<C, R>,
    layout: Layout<C, R>,
    max_swapped: Option<usize>,
    swap_moves: Arc<Vec<SwapMove>>,
}

impl<const C: usize, const R: usize> OptimizableLayout<C, R> {
    pub fn new(
        layout: Layout<C, R>,
        pinned_chars: &HashSet<char>,
        max_swapped: Option<usize>,
    ) -> Self {
        let positions: Vec<Pos> = layout
            .keys()
            .filter(|key| !pinned_chars.contains(&key.ch))
            .map(|key| key.position)
            .collect();
        let swap_moves = Arc::new(SwapMove::all_moves(&positions));

        Self {
            initial_layout: layout,
            layout,
            max_swapped,
            swap_moves,
        }
    }

    fn diff(&self) -> usize {
        self.layout
            .keys()
            .zip(self.initial_layout.keys())
            .filter(|(k1, k2)| k1.ch != k2.ch)
            .count()
    }

    fn try_improve(&mut self, score_check: impl Fn(&Layout<C, R>) -> Option<f64>) -> Option<f64> {
        let (score, best_swap) = self
            .swap_moves
            .clone()
            .iter()
            .filter_map(|swap_move| {
                swap_move.apply(&mut self.layout);
                let score = if let Some(max) = self.max_swapped {
                    if self.diff() <= max {
                        score_check(&self.layout)
                    } else {
                        None
                    }
                } else {
                    score_check(&self.layout)
                };
                swap_move.apply(&mut self.layout);

                score.map(|score| (score, swap_move.clone()))
            })
            .min_by(|(s1, _), (s2, _)| s1.partial_cmp(s2).unwrap_or(Ordering::Equal))?;

        best_swap.apply(&mut self.layout);

        Some(score)
    }

    fn perturb<RNG: Rng + ?Sized>(&mut self, rng: &mut RNG, attempts: usize) {
        let n = self.swap_moves.len();

        if n == 0 {
            return;
        }

        let mut applied = 0;
        for _ in 0..attempts {
            if applied >= 2 {
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

    fn layout(&self) -> &Layout<C, R> {
        &self.layout
    }
}

impl Optimizer {
    pub fn new(analyzer: Analyzer, weights: Weights) -> Self {
        Self { analyzer, weights }
    }

    pub fn optimize<const C: usize, const R: usize>(
        &self,
        layout: &Layout<C, R>,
        iterations: usize,
        seed: Option<u64>,
        pinned_chars: &HashSet<char>,
        max_swapped: Option<usize>,
    ) -> Layout<C, R> {
        let mut rng: StdRng = if let Some(seed) = seed {
            StdRng::seed_from_u64(seed)
        } else {
            StdRng::from_rng(&mut rng())
        };

        let mut best_layout = OptimizableLayout::new(*layout, pinned_chars, max_swapped);
        let mut best_score = f64::INFINITY;

        for iteration in 0..iterations {
            let mut candidate = best_layout.clone();

            if iteration > 0 {
                candidate.perturb(&mut rng, 10);
            }

            let mut current_score = self.get_score(&candidate.layout);

            while let Some(best_iteration_score) = candidate.try_improve(|layout| {
                let score = self.get_score(layout);
                (score < current_score).then_some(score)
            }) {
                current_score = best_iteration_score;
            }

            if current_score < best_score {
                best_score = current_score;
                best_layout = candidate;
            }
        }

        *best_layout.layout()
    }

    fn get_score<const C: usize, const R: usize>(&self, layout: &Layout<C, R>) -> f64 {
        self.get_stats(layout).score
    }

    fn get_stats<const C: usize, const R: usize>(&self, layout: &Layout<C, R>) -> OptimizerStats {
        let mut metrics = OptimizerMetrics::default();
        self.analyzer.analyze(layout, &mut metrics);
        OptimizerStats::from(metrics, &self.weights)
    }
}

#[cfg(test)]
mod optimizer_tests {
    use super::*;
    use crate::corpus::Corpus;
    use assert2::check;

    #[test]
    fn it_optimizes() {
        let layout = Layout::<2, 2>::new(
            "abcd",
            vec![vec![1, 2], vec![1, 2]],
            vec![vec![1.0, 100.0], vec![100.0, 100.0]],
            vec![pos!(0, 0), pos!(0, 1)],
        )
        .unwrap();

        let corpus = Corpus::new([("c".to_string(), 10.0)]);
        let analyzer = Analyzer::new(corpus);
        let optimizer = Optimizer::new(analyzer, Weights { effort: 1.0 });

        let optimized_layout = optimizer.optimize(&layout, 10, Some(42), &HashSet::new(), None);

        check!(optimized_layout.key_for('c').unwrap().effort == 1.0);
    }
}

#[cfg(test)]
mod optimizable_layout_tests {
    use super::*;
    use assert2::check;

    fn make_layout() -> Layout<2, 2> {
        Layout::<2, 2>::new(
            "abcd",
            vec![vec![1, 2], vec![1, 2]],
            vec![vec![1.0, 50.0], vec![100.0, 200.0]],
            vec![pos!(0, 0), pos!(0, 1)],
        )
        .unwrap()
    }

    #[test]
    fn it_does_not_improve_layout_when_no_swap_gives_better_score() {
        let mut optimizable = OptimizableLayout::new(make_layout(), &[].into(), None);
        let result = optimizable.try_improve(|layout| {
            let score = layout_effort_score(layout);
            if score < 0.0 { Some(score) } else { None }
        });

        check!(result == None);
    }

    #[test]
    fn it_improves_layout_by_applying_the_best_swap() {
        let mut optimizable = OptimizableLayout::new(make_layout(), &[].into(), None);
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
        let mut optimizable = OptimizableLayout::new(make_layout(), &['a', 'c'].into(), None);

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
        let mut optimizable = OptimizableLayout::new(make_layout(), &['a', 'b', 'c'].into(), None);
        let result = optimizable.try_improve(|layout| {
            let score = layout.key_for('d').unwrap().effort;
            if score < 200.0 { Some(score) } else { None }
        });

        check!(result == None);
    }

    #[test]
    fn it_does_not_improve_layout_when_swap_exceeds_max_swapped() {
        let mut optimizable = OptimizableLayout::new(make_layout(), &[].into(), Some(0));
        let result = optimizable.try_improve(|layout| {
            let score = layout.key_for('c').unwrap().effort;
            if score < 100.0 { Some(score) } else { None }
        });

        check!(result == None);
    }

    #[test]
    fn it_improves_layout_when_swap_is_within_max_swapped() {
        let mut optimizable = OptimizableLayout::new(make_layout(), &[].into(), Some(2));
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
        let mut optimizable = OptimizableLayout::new(make_layout(), &[].into(), None);

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
        let mut optimizable = OptimizableLayout::new(make_layout(), &[].into(), None);
        let before: Vec<char> = optimizable.layout().keys().map(|k| k.ch).collect();

        let mut rng = StdRng::seed_from_u64(42);
        optimizable.perturb(&mut rng, 10);

        let after: Vec<char> = optimizable.layout().keys().map(|k| k.ch).collect();
        check!(before != after);
    }

    #[test]
    fn it_does_not_perturb_layout_when_all_chars_are_pinned() {
        let mut optimizable =
            OptimizableLayout::new(make_layout(), &['a', 'b', 'c', 'd'].into(), None);
        let before: Vec<char> = optimizable.layout().keys().map(|k| k.ch).collect();

        let mut rng = StdRng::seed_from_u64(42);
        optimizable.perturb(&mut rng, 10);

        let after: Vec<char> = optimizable.layout().keys().map(|k| k.ch).collect();
        check!(before == after);
    }

    #[test]
    fn it_does_not_perturb_layout_beyond_max_swapped() {
        let mut optimizable = OptimizableLayout::new(make_layout(), &[].into(), Some(0));

        let mut rng = StdRng::seed_from_u64(42);
        optimizable.perturb(&mut rng, 10);

        check!(optimizable.diff() == 0);
    }

    #[test]
    fn it_perturbs_layout_within_max_swapped() {
        let mut optimizable = OptimizableLayout::new(make_layout(), &[].into(), Some(4));

        let mut rng = StdRng::seed_from_u64(42);
        optimizable.perturb(&mut rng, 10);

        check!(optimizable.diff() > 0);
        check!(optimizable.diff() <= 4);
    }

    fn layout_effort_score<const C: usize, const R: usize>(layout: &Layout<C, R>) -> f64 {
        layout.keys().map(|k| k.effort).sum()
    }
}
