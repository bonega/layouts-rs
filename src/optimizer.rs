use std::cmp::Ordering;
use std::fmt::Display;

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

impl Optimizer {
    pub fn new(analyzer: Analyzer, weights: Weights) -> Self {
        Self { analyzer, weights }
    }

    pub fn optimize<const C: usize, const R: usize>(
        &self,
        layout: &Layout<C, R>,
        iterations: usize,
        seed: Option<u64>,
    ) -> Layout<C, R> {
        let mut rng: StdRng = if let Some(seed) = seed {
            StdRng::seed_from_u64(seed)
        } else {
            StdRng::from_rng(&mut rng())
        };

        let positions: Vec<Pos> = layout.keys().map(|key| key.position).collect();
        let swap_moves = SwapMove::single_moves(&positions);

        let mut best_score = f64::INFINITY;
        let mut best_layout = *layout;

        for iteration in 0..iterations {
            let mut candidate = best_layout;

            if iteration > 0 {
                Self::perturb(&mut rng, &swap_moves, &mut candidate);
            }

            let mut score = self.get_score(&candidate);

            while let Some((best_iteration_score, swap_move)) =
                self.best_swap(&mut candidate, &swap_moves, score)
            {
                swap_move.apply(&mut candidate);
                score = best_iteration_score;
            }

            if score < best_score {
                best_score = score;
                best_layout = candidate;
            }
        }

        best_layout
    }

    fn best_swap<'a, const C: usize, const R: usize>(
        &self,
        candidate: &mut Layout<C, R>,
        swap_moves: &'a Vec<SwapMove>,
        score: f64,
    ) -> Option<(f64, &'a SwapMove)> {
        swap_moves
            .iter()
            .enumerate()
            .filter_map(|(_, swap_move)| {
                swap_move.apply(candidate);
                let s = self.get_score(candidate);
                swap_move.apply(candidate);
                (s < score).then_some((s, swap_move))
            })
            .min_by(|(s1, _), (s2, _)| s1.partial_cmp(s2).unwrap_or(Ordering::Equal))
    }

    fn perturb<RNG: Rng + ?Sized, const C: usize, const R: usize>(
        rng: &mut RNG,
        swap_moves: &[SwapMove],
        layout: &mut Layout<C, R>,
    ) {
        let n = swap_moves.len();

        if n == 0 {
            return;
        }

        for _ in 0..2 {
            let swap = &swap_moves[rng.next_u64() as usize % n];
            swap.apply(layout);
        }
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
mod tests {
    use assert2::check;

    use crate::corpus::Corpus;

    use super::*;

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

        let optimized_layout = optimizer.optimize(&layout, 10, Some(42));

        check!(optimized_layout.key_for('c').unwrap().effort == 1.0);
    }

    #[test]
    fn best_swap_returns_none_when_all_swaps_are_worse() {
        let mut layout = Layout::<2, 2>::new(
            "abcd",
            vec![vec![1, 2], vec![1, 2]],
            vec![vec![1.0, 100.0], vec![100.0, 100.0]],
            vec![pos!(0, 0), pos!(0, 1)],
        )
        .unwrap();

        let corpus = Corpus::new([("c".to_string(), 10.0)]);
        let analyzer = Analyzer::new(corpus);
        let optimizer = Optimizer::new(analyzer, Weights { effort: 1.0 });

        let swap_moves = SwapMove::single_moves(&[pos!(0, 0), pos!(1, 0), pos!(0, 1), pos!(1, 1)]);
        let result = optimizer.best_swap(&mut layout, &swap_moves, 0.0);

        check!(result == None);
    }

    #[test]
    fn best_swap_picks_move_with_lowest_score() {
        let mut layout = Layout::<2, 2>::new(
            "abcd",
            vec![vec![1, 2], vec![1, 2]],
            vec![vec![1.0, 50.0], vec![100.0, 50.0]],
            vec![pos!(0, 0), pos!(0, 1)],
        )
        .unwrap();

        let corpus = Corpus::new([("c".to_string(), 10.0)]);
        let analyzer = Analyzer::new(corpus);
        let optimizer = Optimizer::new(analyzer, Weights { effort: 1.0 });

        let swap_moves = SwapMove::single_moves(&[pos!(0, 0), pos!(1, 0), pos!(0, 1), pos!(1, 1)]);
        let score = optimizer.get_score(&layout);
        let (_, swap_move) = optimizer
            .best_swap(&mut layout, &swap_moves, score)
            .unwrap();

        check!(swap_move == &SwapMove::Single(pos!(0, 0), pos!(1, 0)));
    }
}
