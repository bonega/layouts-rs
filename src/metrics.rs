use std::collections::HashMap;

use crate::{
    layout::{Finger, FingerKind},
    ngrams::{Bigram, BigramKind, Trigram, TrigramKind, Unigram},
};

#[cfg_attr(test, mockall::automock)]
pub trait MetricsCollector {
    fn collect_metric(&mut self, metric: Metric);
}

#[derive(Debug, PartialEq)]
pub enum Metric {
    Trigram(Trigram, f64),
    Bigram(Bigram, f64),
    Unigram(Unigram, f64),
    CorpusLenght(f64),
}

#[derive(Default)]
pub struct SimpleMetrics {
    pub total_chars: f64,
    pub effort: f64,
    pub column_usage: HashMap<usize, f64>,
    pub row_usage: HashMap<usize, f64>,
    pub finger_usage: HashMap<Finger, f64>,
    pub pinky_off_home: f64,
    pub bigram_skips_1: f64,
    pub bigram_skips_n: f64,
    pub bigram_lateral_stretches: f64,
    pub bigram_scissors: f64,
    pub bigram_wide_scissors: f64,
    pub bigram_others: f64,
    pub trigram_skips_same_hand: f64,
    pub trigram_skips_same_hand_1: f64,
    pub trigram_skips_same_hand_n: f64,
    pub trigram_skips_alternation: f64,
    pub trigram_skips_alternation_1: f64,
    pub trigram_skips_alternation_n: f64,
    pub trigram_lateral_stretches_same_hand: f64,
    pub trigram_lateral_stretches_alternation: f64,
    pub trigram_scissors_same_hand_1: f64,
    pub trigram_scissors_same_hand_n: f64,
    pub trigram_scissors_alternation_1: f64,
    pub trigram_scissors_alternation_n: f64,
    pub trigram_roll_in: f64,
    pub trigram_roll_out: f64,
    pub trigram_roll_in_bigrams: f64,
    pub trigram_roll_out_bigrams: f64,
    pub trigram_redirects_weak: f64,
    pub trigram_redirects_strong: f64,
    pub trigram_alternations: f64,
    pub trigram_others: f64,
}

impl MetricsCollector for SimpleMetrics {
    fn collect_metric(&mut self, metric: Metric) {
        match metric {
            Metric::CorpusLenght(chars) => {
                self.total_chars = chars;
            }
            Metric::Unigram(unigram, count) => {
                self.effort += unigram.key.effort * count;

                *self.row_usage.entry(unigram.key.position.r).or_default() += count;
                *self.column_usage.entry(unigram.key.position.c).or_default() += count;
                *self.finger_usage.entry(unigram.key.finger).or_default() += count;

                if unigram.key.finger.kind == FingerKind::Pinky && !unigram.key.finger_home {
                    self.pinky_off_home += count;
                }
            }
            Metric::Bigram(bigram, count) => {
                for kind in bigram.kinds {
                    match kind {
                        BigramKind::SameFingerSkip { units } => {
                            if units == 1 {
                                self.bigram_skips_1 += count;
                            } else {
                                self.bigram_skips_n += count;
                            }
                        }
                        BigramKind::LateralStretch { .. } => {
                            self.bigram_lateral_stretches += count;
                        }
                        BigramKind::Scissor { units, .. } => {
                            if units >= 2 {
                                self.bigram_wide_scissors += count;
                            } else {
                                self.bigram_scissors += count;
                            }
                        }
                        BigramKind::Other => {
                            self.bigram_others += count;
                        }
                    }
                }
            }
            Metric::Trigram(trigram, count) => {
                for kind in trigram.kinds {
                    match kind {
                        TrigramKind::SameFingerSkip { units, same_hand } => {
                            if same_hand {
                                self.trigram_skips_same_hand += count;
                                if units == 1 {
                                    self.trigram_skips_same_hand_1 += count;
                                } else {
                                    self.trigram_skips_same_hand_n += count;
                                }
                            } else {
                                self.trigram_skips_alternation += count;
                                if units == 1 {
                                    self.trigram_skips_alternation_1 += count;
                                } else {
                                    self.trigram_skips_alternation_n += count;
                                }
                            }
                        }
                        TrigramKind::Roll { triple, inward } => match (triple, inward) {
                            (true, true) => self.trigram_roll_in += count,
                            (true, false) => self.trigram_roll_out += count,
                            (false, true) => self.trigram_roll_in_bigrams += count,
                            (false, false) => self.trigram_roll_out_bigrams += count,
                        },
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
                        TrigramKind::LateralStretch { same_hand, .. } => {
                            if same_hand {
                                self.trigram_lateral_stretches_same_hand += count;
                            } else {
                                self.trigram_lateral_stretches_alternation += count;
                            }
                        }
                        TrigramKind::Scissor {
                            units, same_hand, ..
                        } => {
                            if same_hand {
                                if units >= 2 {
                                    self.trigram_scissors_same_hand_n += count;
                                } else {
                                    self.trigram_scissors_same_hand_1 += count;
                                }
                            } else if units >= 2 {
                                self.trigram_scissors_alternation_n += count;
                            } else {
                                self.trigram_scissors_alternation_1 += count;
                            }
                        }
                        TrigramKind::Other => {
                            self.trigram_others += count;
                        }
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod simple_metrics_tests {
    use assert2::check;
    use rstest::rstest;

    use super::*;
    use crate::layout::{Layout, fixtures::qwerty};

    mod unigram_metrics_tests {
        use super::*;

        #[rstest]
        fn it_collects_row_usage(qwerty: Layout) {
            let mut metrics = SimpleMetrics::default();

            metrics.collect_metric(Metric::Unigram(ngram!(qwerty, 'q'), 1.0));
            metrics.collect_metric(Metric::Unigram(ngram!(qwerty, 'w'), 1.0));
            metrics.collect_metric(Metric::Unigram(ngram!(qwerty, 'a'), 10.0));
            metrics.collect_metric(Metric::Unigram(ngram!(qwerty, 'z'), 100.0));

            check!(metrics.row_usage.get(&0).unwrap() == &2.0);
            check!(metrics.row_usage.get(&1).unwrap() == &10.0);
            check!(metrics.row_usage.get(&2).unwrap() == &100.0);
        }

        #[rstest]
        fn it_collects_column_usage(qwerty: Layout) {
            let mut metrics = SimpleMetrics::default();

            metrics.collect_metric(Metric::Unigram(ngram!(qwerty, 'q'), 1.0));
            metrics.collect_metric(Metric::Unigram(ngram!(qwerty, 'a'), 1.0));
            metrics.collect_metric(Metric::Unigram(ngram!(qwerty, 'w'), 10.0));
            metrics.collect_metric(Metric::Unigram(ngram!(qwerty, 'e'), 100.0));

            check!(metrics.column_usage.get(&1).unwrap() == &2.0);
            check!(metrics.column_usage.get(&2).unwrap() == &10.0);
            check!(metrics.column_usage.get(&3).unwrap() == &100.0);
        }

        #[rstest]
        fn it_collects_finger_usage(qwerty: Layout) {
            let mut metrics = SimpleMetrics::default();

            metrics.collect_metric(Metric::Unigram(ngram!(qwerty, 'q'), 1.0));
            metrics.collect_metric(Metric::Unigram(ngram!(qwerty, 'a'), 1.0));
            metrics.collect_metric(Metric::Unigram(ngram!(qwerty, 'w'), 10.0));
            metrics.collect_metric(Metric::Unigram(ngram!(qwerty, 'e'), 100.0));

            check!(metrics.finger_usage.get(&Finger::from(1)).unwrap() == &2.0);
            check!(metrics.finger_usage.get(&Finger::from(2)).unwrap() == &10.0);
            check!(metrics.finger_usage.get(&Finger::from(3)).unwrap() == &100.0);
        }

        #[rstest]
        fn it_collects_pinky_off_home(qwerty: Layout) {
            let mut metrics = SimpleMetrics::default();

            metrics.collect_metric(Metric::Unigram(ngram!(qwerty, 'a'), 1000.0));
            metrics.collect_metric(Metric::Unigram(ngram!(qwerty, 'q'), 1.0));
            metrics.collect_metric(Metric::Unigram(ngram!(qwerty, 'z'), 10.0));
            metrics.collect_metric(Metric::Unigram(ngram!(qwerty, '"'), 100.0));

            check!(metrics.pinky_off_home == 111.0);
        }

        #[rstest]
        fn it_collects_key_effort(qwerty: Layout) {
            let mut metrics = SimpleMetrics::default();

            metrics.collect_metric(Metric::Unigram(ngram!(qwerty, 'a'), 1.0));
            metrics.collect_metric(Metric::Unigram(ngram!(qwerty, 'q'), 2.0));
            metrics.collect_metric(Metric::Unigram(ngram!(qwerty, 'z'), 1.0));
            metrics.collect_metric(Metric::Unigram(ngram!(qwerty, '"'), 1.0));

            check!(metrics.effort == 9.0);
        }
    }

    mod bigram_metrics_tests {
        use super::*;

        #[rstest]
        fn it_collects_skips(qwerty: Layout) {
            let mut metrics = SimpleMetrics::default();

            metrics.collect_metric(Metric::Bigram(ngram!(qwerty, 'q', 'a'), 10.0));
            metrics.collect_metric(Metric::Bigram(ngram!(qwerty, 'q', 'z'), 20.0));

            check!(metrics.bigram_skips_1 == 10.0);
            check!(metrics.bigram_skips_n == 20.0);
        }

        #[rstest]
        fn it_collects_lateral_stretches(qwerty: Layout) {
            let mut metrics = SimpleMetrics::default();

            metrics.collect_metric(Metric::Bigram(ngram!(qwerty, 'd', 'g'), 10.0));
            metrics.collect_metric(Metric::Bigram(ngram!(qwerty, 's', '"'), 20.0));

            check!(metrics.bigram_lateral_stretches == 30.0);
        }

        #[rstest]
        fn it_collects_scissors(qwerty: Layout) {
            let mut metrics = SimpleMetrics::default();

            metrics.collect_metric(Metric::Bigram(ngram!(qwerty, 'c', 'w'), 10.0));
            metrics.collect_metric(Metric::Bigram(ngram!(qwerty, 'c', 's'), 20.0));

            check!(metrics.bigram_wide_scissors == 10.0);
            check!(metrics.bigram_scissors == 20.0);
        }

        #[rstest]
        fn it_collects_others(qwerty: Layout) {
            let mut metrics = SimpleMetrics::default();

            metrics.collect_metric(Metric::Bigram(ngram!(qwerty, 'd', 'h'), 10.0));
            metrics.collect_metric(Metric::Bigram(ngram!(qwerty, 'd', 'f'), 20.0));

            check!(metrics.bigram_others == 30.0);
        }
    }

    mod trigram_metrics_tests {
        use super::*;

        #[rstest]
        fn it_collects_same_finger_skips(qwerty: Layout) {
            let mut metrics = SimpleMetrics::default();

            metrics.collect_metric(Metric::Trigram(ngram!(qwerty, 'q', 'w', 'a'), 10.0));
            metrics.collect_metric(Metric::Trigram(ngram!(qwerty, 'q', 'h', 'a'), 20.0));

            check!(metrics.trigram_skips_same_hand == 10.0);
            check!(metrics.trigram_skips_same_hand_1 == 10.0);
            check!(metrics.trigram_skips_same_hand_n == 0.0);
            check!(metrics.trigram_skips_alternation == 20.0);
            check!(metrics.trigram_skips_alternation_1 == 20.0);
            check!(metrics.trigram_skips_alternation_n == 0.0);
        }

        #[rstest]
        fn it_collects_trigram_lateral_stretches(qwerty: Layout) {
            let mut metrics = SimpleMetrics::default();

            metrics.collect_metric(Metric::Trigram(ngram!(qwerty, 'l', 'i', '\''), 10.0));
            metrics.collect_metric(Metric::Trigram(ngram!(qwerty, 'd', 'u', 'g'), 20.0));

            check!(metrics.trigram_lateral_stretches_same_hand == 10.0);
            check!(metrics.trigram_lateral_stretches_alternation == 20.0);
        }

        #[rstest]
        fn it_collects_trigram_scissors(qwerty: Layout) {
            let mut metrics = SimpleMetrics::default();

            metrics.collect_metric(Metric::Trigram(ngram!(qwerty, 'c', 'a', 'w'), 10.0));
            metrics.collect_metric(Metric::Trigram(ngram!(qwerty, 'c', 'j', 'w'), 20.0));

            check!(metrics.trigram_scissors_same_hand_1 == 0.0);
            check!(metrics.trigram_scissors_same_hand_n == 10.0);
            check!(metrics.trigram_scissors_alternation_1 == 0.0);
            check!(metrics.trigram_scissors_alternation_n == 20.0);
        }

        #[rstest]
        fn it_collects_rolls(qwerty: Layout) {
            let mut metrics = SimpleMetrics::default();

            metrics.collect_metric(Metric::Trigram(ngram!(qwerty, 'q', 'w', 'e'), 10.0));
            metrics.collect_metric(Metric::Trigram(ngram!(qwerty, 't', 'e', 'q'), 20.0));
            metrics.collect_metric(Metric::Trigram(ngram!(qwerty, 'q', 'w', 'p'), 30.0));
            metrics.collect_metric(Metric::Trigram(ngram!(qwerty, 't', 'e', 'p'), 40.0));

            check!(metrics.trigram_roll_in == 10.0);
            check!(metrics.trigram_roll_out == 20.0);
            check!(metrics.trigram_roll_in_bigrams == 30.0);
            check!(metrics.trigram_roll_out_bigrams == 40.0);
        }

        #[rstest]
        fn it_collects_redirects(qwerty: Layout) {
            let mut metrics = SimpleMetrics::default();

            metrics.collect_metric(Metric::Trigram(ngram!(qwerty, 'q', 't', 'e'), 10.0));
            metrics.collect_metric(Metric::Trigram(ngram!(qwerty, 'q', 'e', 'w'), 20.0));

            check!(metrics.trigram_redirects_weak == 10.0);
            check!(metrics.trigram_redirects_strong == 20.0);
        }

        #[rstest]
        fn it_collects_alternations_and_others(qwerty: Layout) {
            let mut metrics = SimpleMetrics::default();

            metrics.collect_metric(Metric::Trigram(ngram!(qwerty, 'q', 'h', 'w'), 10.0));
            metrics.collect_metric(Metric::Trigram(ngram!(qwerty, 'q', 'q', 'a'), 20.0));

            check!(metrics.trigram_alternations == 10.0);
            check!(metrics.trigram_others == 20.0);
        }
    }
}
