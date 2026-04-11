use std::collections::HashMap;
use std::fmt::{Display, Formatter};

use crate::{
    analyzer::{Metric, Metrics},
    layout::{Finger, FingerKind, Hand},
    ngrams::{BigramKind, TrigramKind},
};

#[derive(Default, Debug)]
pub struct ReportMetrics {
    total_chars: f64,
    effort: f64,
    column_usage: HashMap<usize, f64>,
    row_usage: HashMap<usize, f64>,
    finger_usage: HashMap<Finger, f64>,
    pinky_off_home: f64,
    bigram_skips_1: f64,
    bigram_skips_n: f64,
    bigram_lateral_stretches: f64,
    bigram_scissors: f64,
    bigram_wide_scissors: f64,
    bigram_others: f64,
    trigram_same_hand_skips: f64,
    trigram_alternation_skips: f64,
    trigram_roll_in: f64,
    trigram_roll_out: f64,
    trigram_roll_in_bigrams: f64,
    trigram_roll_out_bigrams: f64,
    trigram_redirects_weak: f64,
    trigram_redirects_strong: f64,
    trigram_alternations: f64,
    trigram_others: f64,
}

impl Metrics for ReportMetrics {
    fn collect_metric(&mut self, metric: Metric) {
        match metric {
            Metric::Unigram(unigram, count) => {
                self.effort += unigram.key.effort * count;

                *self.row_usage.entry(unigram.key.position.r).or_default() += count;
                *self.column_usage.entry(unigram.key.position.c).or_default() += count;
                *self.finger_usage.entry(unigram.key.finger).or_default() += count;

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
                BigramKind::Other => {
                    self.bigram_others += count;
                }
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
                    (false, true) => self.trigram_roll_in_bigrams += count,
                    (false, false) => self.trigram_roll_out_bigrams += count,
                },
                TrigramKind::RollIn { triple } => {
                    if triple {
                        self.trigram_roll_in += count;
                    } else {
                        self.trigram_roll_in_bigrams += count;
                    }
                }
                TrigramKind::RollOut { triple } => {
                    if triple {
                        self.trigram_roll_out += count;
                    } else {
                        self.trigram_roll_out_bigrams += count;
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
                TrigramKind::Other => {
                    self.trigram_others += count;
                }
            },
            Metric::CorpusLenght(total_chars) => {
                self.total_chars = total_chars;
            }
        }
    }
}

#[derive(Debug, PartialEq)]
pub struct Report {
    effort: f64,
    left_hand_usage: f64,
    right_hand_usage: f64,
    finger_usage: HashMap<Finger, f64>,
    row_usage: HashMap<usize, f64>,
    column_usage: HashMap<usize, f64>,
    pinky_off_home: f64,
    bigram_skips_1: f64,
    bigram_skips_n: f64,
    bigram_lateral_stretches: f64,
    bigram_scissors: f64,
    bigram_wide_scissors: f64,
    bigram_others: f64,
    trigram_same_hand_skips: f64,
    trigram_alternation_skips: f64,
    trigram_roll_in: f64,
    trigram_roll_out: f64,
    trigram_roll_in_bigrams: f64,
    trigram_roll_out_bigrams: f64,
    trigram_redirects_weak: f64,
    trigram_redirects_strong: f64,
    trigram_alternations: f64,
    trigram_others: f64,
}

impl From<ReportMetrics> for Report {
    fn from(metrics: ReportMetrics) -> Self {
        let left_hand_usage = metrics
            .finger_usage
            .iter()
            .filter(|(finger, _)| finger.hand == Hand::Left)
            .map(|(_, usage)| usage)
            .sum::<f64>();

        let right_hand_usage = metrics
            .finger_usage
            .iter()
            .filter(|(finger, _)| finger.hand == Hand::Right)
            .map(|(_, usage)| usage)
            .sum::<f64>();

        let total_hand_usage = left_hand_usage + right_hand_usage;

        let total_row_usage: f64 = metrics.row_usage.values().sum();
        let total_column_usage: f64 = metrics.column_usage.values().sum();

        Self {
            effort: 100.0 * metrics.effort / metrics.total_chars,
            left_hand_usage: 100.0 * left_hand_usage / total_hand_usage,
            right_hand_usage: 100.0 * right_hand_usage / total_hand_usage,
            finger_usage: metrics
                .finger_usage
                .iter()
                .map(|(finger, usage)| (*finger, 100.0 * usage / total_hand_usage))
                .collect(),
            row_usage: metrics
                .row_usage
                .iter()
                .map(|(row, usage)| (*row, 100.0 * usage / total_row_usage))
                .collect(),
            column_usage: metrics
                .column_usage
                .iter()
                .map(|(column, usage)| (*column, 100.0 * usage / total_column_usage))
                .collect(),
            pinky_off_home: 100.0 * metrics.pinky_off_home / metrics.total_chars,
            bigram_skips_1: 100.0 * metrics.bigram_skips_1 / metrics.total_chars,
            bigram_skips_n: 100.0 * metrics.bigram_skips_n / metrics.total_chars,
            bigram_lateral_stretches: 100.0 * metrics.bigram_lateral_stretches
                / metrics.total_chars,
            bigram_scissors: 100.0 * metrics.bigram_scissors / metrics.total_chars,
            bigram_wide_scissors: 100.0 * metrics.bigram_wide_scissors / metrics.total_chars,
            bigram_others: 100.0 * metrics.bigram_others / metrics.total_chars,
            trigram_same_hand_skips: 100.0 * metrics.trigram_same_hand_skips / metrics.total_chars,
            trigram_alternation_skips: 100.0 * metrics.trigram_alternation_skips
                / metrics.total_chars,
            trigram_roll_in: 100.0 * metrics.trigram_roll_in / metrics.total_chars,
            trigram_roll_out: 100.0 * metrics.trigram_roll_out / metrics.total_chars,
            trigram_roll_in_bigrams: 100.0 * metrics.trigram_roll_in_bigrams / metrics.total_chars,
            trigram_roll_out_bigrams: 100.0 * metrics.trigram_roll_out_bigrams
                / metrics.total_chars,
            trigram_redirects_weak: 100.0 * metrics.trigram_redirects_weak / metrics.total_chars,
            trigram_redirects_strong: 100.0 * metrics.trigram_redirects_strong
                / metrics.total_chars,
            trigram_alternations: 100.0 * metrics.trigram_alternations / metrics.total_chars,
            trigram_others: 100.0 * metrics.trigram_others / metrics.total_chars,
        }
    }
}

impl Display for Report {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "Effort: {:.2}%", self.effort)?;
        writeln!(f, "Left hand usage: {:.2}%", self.left_hand_usage)?;
        writeln!(f, "Right hand usage: {:.2}%", self.right_hand_usage)?;
        writeln!(f, "Pinky off home: {:.2}%", self.pinky_off_home)?;

        writeln!(f, "Finger usage:")?;
        let mut fingers: Vec<_> = self.finger_usage.iter().collect();
        fingers.sort_by_key(|(finger, _)| u8::from(**finger));
        for (finger, usage) in fingers {
            writeln!(f, "  {:?}: {:.2}%", u8::from(*finger), usage)?;
        }

        writeln!(f, "Row usage:")?;
        let mut rows: Vec<_> = self.row_usage.iter().collect();
        rows.sort_by_key(|(row, _)| **row);
        for (row, usage) in rows {
            writeln!(f, "  Row {}: {:.2}%", row, usage)?;
        }

        writeln!(f, "Column usage:")?;
        let mut columns: Vec<_> = self.column_usage.iter().collect();
        columns.sort_by_key(|(col, _)| **col);
        for (column, usage) in columns {
            writeln!(f, "  Column {}: {:.2}%", column, usage)?;
        }

        writeln!(f, "Bigram metrics:")?;
        writeln!(f, "  Skips (1): {:.2}%", self.bigram_skips_1)?;
        writeln!(f, "  Skips (n): {:.2}%", self.bigram_skips_n)?;
        writeln!(
            f,
            "  Lateral stretches: {:.2}%",
            self.bigram_lateral_stretches
        )?;
        writeln!(f, "  Scissors: {:.2}%", self.bigram_scissors)?;
        writeln!(f, "  Others: {:.2}%", self.bigram_others)?;
        writeln!(f, "Trigram metrics:")?;
        writeln!(f, "  Same-hand skips: {:.2}%", self.trigram_same_hand_skips)?;
        writeln!(
            f,
            "  Alternation skips: {:.2}%",
            self.trigram_alternation_skips
        )?;
        writeln!(f, "  Roll-in: {:.2}%", self.trigram_roll_in)?;
        writeln!(f, "  Roll-out: {:.2}%", self.trigram_roll_out)?;
        writeln!(f, "  Roll-in bigrams: {:.2}%", self.trigram_roll_in_bigrams)?;
        writeln!(
            f,
            "  Roll-out bigrams: {:.2}%",
            self.trigram_roll_out_bigrams
        )?;
        writeln!(f, "  Redirects (weak): {:.2}%", self.trigram_redirects_weak)?;
        writeln!(
            f,
            "  Redirects (strong): {:.2}%",
            self.trigram_redirects_strong
        )?;
        writeln!(f, "  Alternations: {:.2}%", self.trigram_alternations)?;
        writeln!(f, "  Others: {:.2}%", self.trigram_others)?;
        Ok(())
    }
}

#[cfg(test)]
mod report_metrics_tests {
    use assert2::check;
    use rstest::rstest;

    use super::*;
    use crate::layout::{Layout, fixtures::qwerty};

    mod unigram_metrics_tests {

        use super::*;

        #[rstest]
        fn it_collects_row_usage(qwerty: Layout) {
            let mut metrics = ReportMetrics::default();

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
            let mut metrics = ReportMetrics::default();

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
            let mut metrics = ReportMetrics::default();

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
            let mut metrics = ReportMetrics::default();

            metrics.collect_metric(Metric::Unigram(ngram!(qwerty, 'a'), 1000.0));
            metrics.collect_metric(Metric::Unigram(ngram!(qwerty, 'q'), 1.0));
            metrics.collect_metric(Metric::Unigram(ngram!(qwerty, 'z'), 10.0));
            metrics.collect_metric(Metric::Unigram(ngram!(qwerty, '"'), 100.0));

            check!(metrics.pinky_off_home == 111.0);
        }

        #[rstest]
        fn it_collects_key_effort(qwerty: Layout) {
            let mut metrics = ReportMetrics::default();

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
            let mut metrics = ReportMetrics::default();

            metrics.collect_metric(Metric::Bigram(ngram!(qwerty, 'q', 'a'), 10.0));
            metrics.collect_metric(Metric::Bigram(ngram!(qwerty, 'q', 'z'), 20.0));

            check!(metrics.bigram_skips_1 == 10.0);
            check!(metrics.bigram_skips_n == 20.0);
        }

        #[rstest]
        fn it_collects_lateral_stretches(qwerty: Layout) {
            let mut metrics = ReportMetrics::default();

            metrics.collect_metric(Metric::Bigram(ngram!(qwerty, 'd', 'g'), 10.0));
            metrics.collect_metric(Metric::Bigram(ngram!(qwerty, 's', '"'), 20.0));

            check!(metrics.bigram_lateral_stretches == 30.0);
        }

        #[rstest]
        fn it_collects_scissors(qwerty: Layout) {
            let mut metrics = ReportMetrics::default();

            metrics.collect_metric(Metric::Bigram(ngram!(qwerty, 'd', 't'), 10.0));
            metrics.collect_metric(Metric::Bigram(ngram!(qwerty, 'd', 'r'), 20.0));

            check!(metrics.bigram_wide_scissors == 10.0);
            check!(metrics.bigram_scissors == 20.0);
        }

        #[rstest]
        fn it_collects_others(qwerty: Layout) {
            let mut metrics = ReportMetrics::default();

            metrics.collect_metric(Metric::Bigram(ngram!(qwerty, 'd', 'h'), 10.0));
            metrics.collect_metric(Metric::Bigram(ngram!(qwerty, 'd', 'f'), 20.0));

            check!(metrics.bigram_others == 30.0);
        }
    }

    mod trigram_metrics_tests {
        use super::*;

        #[rstest]
        fn it_collects_same_finger_skips(qwerty: Layout) {
            let mut metrics = ReportMetrics::default();

            metrics.collect_metric(Metric::Trigram(ngram!(qwerty, 'q', 'w', 'a'), 10.0));
            metrics.collect_metric(Metric::Trigram(ngram!(qwerty, 'q', 'h', 'a'), 20.0));

            check!(metrics.trigram_same_hand_skips == 10.0);
            check!(metrics.trigram_alternation_skips == 20.0);
        }

        #[rstest]
        fn it_collects_rolls(qwerty: Layout) {
            let mut metrics = ReportMetrics::default();

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
            let mut metrics = ReportMetrics::default();

            metrics.collect_metric(Metric::Trigram(ngram!(qwerty, 'q', 't', 'e'), 10.0));
            metrics.collect_metric(Metric::Trigram(ngram!(qwerty, 'q', 'e', 'w'), 20.0));

            check!(metrics.trigram_redirects_weak == 10.0);
            check!(metrics.trigram_redirects_strong == 20.0);
        }

        #[rstest]
        fn it_collects_alternations_and_others(qwerty: Layout) {
            let mut metrics = ReportMetrics::default();

            metrics.collect_metric(Metric::Trigram(ngram!(qwerty, 'q', 'h', 'w'), 10.0));
            metrics.collect_metric(Metric::Trigram(ngram!(qwerty, 'q', 'q', 'a'), 20.0));

            check!(metrics.trigram_alternations == 10.0);
            check!(metrics.trigram_others == 20.0);
        }
    }
}

#[cfg(test)]
mod report_tests {
    use assert2::check;

    use super::*;

    #[test]
    fn it_can_be_built_from_metrics() {
        let metrics = ReportMetrics {
            total_chars: 200.0,
            effort: 400.0,
            finger_usage: [
                (1.into(), 20.0),
                (2.into(), 40.0),
                (3.into(), 60.0),
                (8.into(), 80.0),
            ]
            .into(),
            column_usage: [(1, 20.0), (2, 40.0), (3, 60.0), (4, 80.0)].into(),
            row_usage: [(0, 20.0), (1, 40.0), (2, 60.0), (3, 80.0)].into(),
            pinky_off_home: 10.0,
            bigram_skips_1: 20.0,
            bigram_skips_n: 40.0,
            bigram_lateral_stretches: 60.0,
            bigram_scissors: 80.0,
            bigram_wide_scissors: 80.0,
            bigram_others: 40.0,
            trigram_same_hand_skips: 10.0,
            trigram_alternation_skips: 20.0,
            trigram_roll_in: 30.0,
            trigram_roll_out: 40.0,
            trigram_roll_in_bigrams: 50.0,
            trigram_roll_out_bigrams: 60.0,
            trigram_redirects_weak: 70.0,
            trigram_redirects_strong: 80.0,
            trigram_alternations: 90.0,
            trigram_others: 100.0,
        };

        let report = Report::from(metrics);

        check!(
            report
                == Report {
                    effort: 200.0,
                    left_hand_usage: 60.0,
                    right_hand_usage: 40.0,
                    finger_usage: [
                        (1.into(), 10.0),
                        (2.into(), 20.0),
                        (3.into(), 30.0),
                        (8.into(), 40.0),
                    ]
                    .into(),
                    column_usage: [(1, 10.0), (2, 20.0), (3, 30.0), (4, 40.0)].into(),
                    row_usage: [(0, 10.0), (1, 20.0), (2, 30.0), (3, 40.0)].into(),
                    pinky_off_home: 5.0,
                    bigram_skips_1: 10.0,
                    bigram_skips_n: 20.0,
                    bigram_lateral_stretches: 30.0,
                    bigram_scissors: 40.0,
                    bigram_wide_scissors: 40.0,
                    bigram_others: 20.0,
                    trigram_same_hand_skips: 5.0,
                    trigram_alternation_skips: 10.0,
                    trigram_roll_in: 15.0,
                    trigram_roll_out: 20.0,
                    trigram_roll_in_bigrams: 25.0,
                    trigram_roll_out_bigrams: 30.0,
                    trigram_redirects_weak: 35.0,
                    trigram_redirects_strong: 40.0,
                    trigram_alternations: 45.0,
                    trigram_others: 50.0,
                }
        );
    }
}
