use std::collections::HashMap;
use std::fmt::{Display, Formatter};

use crate::{
    analyzer::{Metric, Metrics},
    layout::{Finger, FingerKind, Hand},
    ngrams::{Bigram, BigramKind, Unigram},
};

#[derive(Default, Debug)]
pub struct ReportMetrics {
    total_chars: f64,
    unigram_metrics: ReportUnigramMetrics,
    bigram_metrics: ReportBigramMetrics,
}

impl Metrics for ReportMetrics {
    fn collect_metric(&mut self, metric: Metric) {
        match metric {
            Metric::Unigram(unigram, count) => {
                self.unigram_metrics.collect(unigram, count);
            }
            Metric::Bigram(bigram, count) => {
                self.bigram_metrics.collect(bigram, count);
            }
            Metric::Trigram(_trigram, _count) => {}
            Metric::CorpusLenght(total_chars) => {
                self.total_chars = total_chars;
            }
        }
    }
}

#[derive(Default, Debug)]
struct ReportUnigramMetrics {
    effort: f64,
    column_usage: HashMap<usize, f64>,
    row_usage: HashMap<usize, f64>,
    finger_usage: HashMap<Finger, f64>,
    pinky_off_home: f64,
}

impl ReportUnigramMetrics {
    fn collect(&mut self, unigram: Unigram, count: f64) {
        self.effort += unigram.key.effort * count;

        *self.row_usage.entry(unigram.key.position.r).or_default() += count;
        *self.column_usage.entry(unigram.key.position.c).or_default() += count;
        *self.finger_usage.entry(unigram.key.finger).or_default() += count;

        if unigram.key.finger.kind == FingerKind::Pinky && !unigram.key.finger_home {
            self.pinky_off_home += count;
        }
    }
}

#[derive(Default, Debug, PartialEq)]
struct ReportBigramMetrics {
    skips_1: f64,
    skips_n: f64,
    lateral_stretches: f64,
    scissors: f64,
    wide_scissors: f64,
    others: f64,
}

impl ReportBigramMetrics {
    fn collect(&mut self, bigram: Bigram, count: f64) {
        match bigram.kind {
            BigramKind::SameFingerSkip { skips } => {
                if skips == 1 {
                    self.skips_1 += count;
                } else {
                    self.skips_n += count;
                }
            }
            BigramKind::LateralStretch { .. } => {
                self.lateral_stretches += count;
            }
            BigramKind::Scissor {
                col_distance,
                row_distance,
            } => {
                if col_distance + row_distance > 2 {
                    self.wide_scissors += count;
                } else {
                    self.scissors += count;
                }
            }
            BigramKind::Other => {
                self.others += count;
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
}

impl From<ReportMetrics> for Report {
    fn from(metrics: ReportMetrics) -> Self {
        let left_hand_usage = metrics
            .unigram_metrics
            .finger_usage
            .iter()
            .filter(|(finger, _)| finger.hand == Hand::Left)
            .map(|(_, usage)| usage)
            .sum::<f64>();

        let right_hand_usage = metrics
            .unigram_metrics
            .finger_usage
            .iter()
            .filter(|(finger, _)| finger.hand == Hand::Right)
            .map(|(_, usage)| usage)
            .sum::<f64>();

        let total_hand_usage = left_hand_usage + right_hand_usage;

        let total_row_usage: f64 = metrics.unigram_metrics.row_usage.values().sum();
        let total_column_usage: f64 = metrics.unigram_metrics.column_usage.values().sum();

        Self {
            effort: 100.0 * metrics.unigram_metrics.effort / metrics.total_chars,
            left_hand_usage: 100.0 * left_hand_usage / total_hand_usage,
            right_hand_usage: 100.0 * right_hand_usage / total_hand_usage,
            finger_usage: metrics
                .unigram_metrics
                .finger_usage
                .iter()
                .map(|(finger, usage)| (*finger, 100.0 * usage / total_hand_usage))
                .collect(),
            row_usage: metrics
                .unigram_metrics
                .row_usage
                .iter()
                .map(|(row, usage)| (*row, 100.0 * usage / total_row_usage))
                .collect(),
            column_usage: metrics
                .unigram_metrics
                .column_usage
                .iter()
                .map(|(column, usage)| (*column, 100.0 * usage / total_column_usage))
                .collect(),
            pinky_off_home: 100.0 * metrics.unigram_metrics.pinky_off_home / metrics.total_chars,
            bigram_skips_1: 100.0 * metrics.bigram_metrics.skips_1 / metrics.total_chars,
            bigram_skips_n: 100.0 * metrics.bigram_metrics.skips_n / metrics.total_chars,
            bigram_lateral_stretches: 100.0 * metrics.bigram_metrics.lateral_stretches
                / metrics.total_chars,
            bigram_scissors: 100.0 * metrics.bigram_metrics.scissors / metrics.total_chars,
            bigram_wide_scissors: 100.0 * metrics.bigram_metrics.wide_scissors
                / metrics.total_chars,
            bigram_others: 100.0 * metrics.bigram_metrics.others / metrics.total_chars,
        }
    }
}

impl Display for Report {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "Effort: {}%", self.effort)?;
        writeln!(f, "Left hand usage: {}%", self.left_hand_usage)?;
        writeln!(f, "Right hand usage: {}%", self.right_hand_usage)?;
        writeln!(f, "Pinky off home: {}%", self.pinky_off_home)?;
        writeln!(f, "Finger usage:")?;
        for (finger, usage) in &self.finger_usage {
            writeln!(f, "  {:?}: {}%", u8::from(*finger), usage)?;
        }
        writeln!(f, "Row usage:")?;
        for (row, usage) in &self.row_usage {
            writeln!(f, "  Row {}: {}%", row, usage)?;
        }
        writeln!(f, "Column usage:")?;
        for (column, usage) in &self.column_usage {
            writeln!(f, "  Column {}: {}%", column, usage)?;
        }
        writeln!(f, "Bigram metrics:")?;
        writeln!(f, "  Skips (1): {}%", self.bigram_skips_1)?;
        writeln!(f, "  Skips (n): {}%", self.bigram_skips_n)?;
        writeln!(f, "  Lateral stretches: {}%", self.bigram_lateral_stretches)?;
        writeln!(f, "  Scissors: {}%", self.bigram_scissors)?;
        writeln!(f, "  Others: {}%", self.bigram_others)?;
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
            let mut metrics = ReportUnigramMetrics::default();

            metrics.collect(Unigram::new(qwerty.key_for('q').unwrap()), 1.0);
            metrics.collect(Unigram::new(qwerty.key_for('w').unwrap()), 1.0);
            metrics.collect(Unigram::new(qwerty.key_for('a').unwrap()), 10.0);
            metrics.collect(Unigram::new(qwerty.key_for('z').unwrap()), 100.0);

            check!(metrics.row_usage.get(&0).unwrap() == &2.0);
            check!(metrics.row_usage.get(&1).unwrap() == &10.0);
            check!(metrics.row_usage.get(&2).unwrap() == &100.0);
        }

        #[rstest]
        fn it_collects_column_usage(qwerty: Layout) {
            let mut metrics = ReportUnigramMetrics::default();

            metrics.collect(Unigram::new(qwerty.key_for('q').unwrap()), 1.0);
            metrics.collect(Unigram::new(qwerty.key_for('a').unwrap()), 1.0);
            metrics.collect(Unigram::new(qwerty.key_for('w').unwrap()), 10.0);
            metrics.collect(Unigram::new(qwerty.key_for('e').unwrap()), 100.0);

            check!(metrics.column_usage.get(&1).unwrap() == &2.0);
            check!(metrics.column_usage.get(&2).unwrap() == &10.0);
            check!(metrics.column_usage.get(&3).unwrap() == &100.0);
        }

        #[rstest]
        fn it_collects_finger_usage(qwerty: Layout) {
            let mut metrics = ReportUnigramMetrics::default();

            metrics.collect(Unigram::new(qwerty.key_for('q').unwrap()), 1.0);
            metrics.collect(Unigram::new(qwerty.key_for('a').unwrap()), 1.0);
            metrics.collect(Unigram::new(qwerty.key_for('w').unwrap()), 10.0);
            metrics.collect(Unigram::new(qwerty.key_for('e').unwrap()), 100.0);

            check!(metrics.finger_usage.get(&Finger::from(1)).unwrap() == &2.0);
            check!(metrics.finger_usage.get(&Finger::from(2)).unwrap() == &10.0);
            check!(metrics.finger_usage.get(&Finger::from(3)).unwrap() == &100.0);
        }

        #[rstest]
        fn it_collects_pinky_off_home(qwerty: Layout) {
            let mut metrics = ReportUnigramMetrics::default();

            metrics.collect(Unigram::new(qwerty.key_for('a').unwrap()), 1000.0);
            metrics.collect(Unigram::new(qwerty.key_for('q').unwrap()), 1.0);
            metrics.collect(Unigram::new(qwerty.key_for('z').unwrap()), 10.0);
            metrics.collect(Unigram::new(qwerty.key_for('"').unwrap()), 100.0);

            check!(metrics.pinky_off_home == 111.0);
        }

        #[rstest]
        fn it_collects_key_effort(qwerty: Layout) {
            let mut metrics = ReportUnigramMetrics::default();

            metrics.collect(Unigram::new(qwerty.key_for('a').unwrap()), 1.0);
            metrics.collect(Unigram::new(qwerty.key_for('q').unwrap()), 2.0);
            metrics.collect(Unigram::new(qwerty.key_for('z').unwrap()), 1.0);
            metrics.collect(Unigram::new(qwerty.key_for('"').unwrap()), 1.0);

            check!(metrics.effort == 9.0);
        }
    }

    mod bigram_metrics_tests {
        use crate::ngrams::Bigram;

        use super::*;

        #[rstest]
        fn it_collects_skips(qwerty: Layout) {
            let mut metrics = ReportBigramMetrics::default();

            metrics.collect(
                Bigram::new(qwerty.key_for('q').unwrap(), qwerty.key_for('a').unwrap()),
                10.0,
            );
            metrics.collect(
                Bigram::new(qwerty.key_for('q').unwrap(), qwerty.key_for('z').unwrap()),
                20.0,
            );

            check!(metrics.skips_1 == 10.0);
            check!(metrics.skips_n == 20.0);
        }

        #[rstest]
        fn it_collects_lateral_stretches(qwerty: Layout) {
            let mut metrics = ReportBigramMetrics::default();

            metrics.collect(
                Bigram::new(qwerty.key_for('d').unwrap(), qwerty.key_for('g').unwrap()),
                10.0,
            );
            metrics.collect(
                Bigram::new(qwerty.key_for('s').unwrap(), qwerty.key_for('"').unwrap()),
                20.0,
            );

            check!(metrics.lateral_stretches == 30.0);
        }

        #[rstest]
        fn it_collects_scissors(qwerty: Layout) {
            let mut metrics = ReportBigramMetrics::default();

            metrics.collect(
                Bigram::new(qwerty.key_for('d').unwrap(), qwerty.key_for('t').unwrap()),
                10.0,
            );
            metrics.collect(
                Bigram::new(qwerty.key_for('d').unwrap(), qwerty.key_for('r').unwrap()),
                20.0,
            );

            check!(metrics.wide_scissors == 10.0);
            check!(metrics.scissors == 20.0);
        }

        #[rstest]
        fn it_collects_others(qwerty: Layout) {
            let mut metrics = ReportBigramMetrics::default();

            metrics.collect(
                Bigram::new(qwerty.key_for('d').unwrap(), qwerty.key_for('h').unwrap()),
                10.0,
            );
            metrics.collect(
                Bigram::new(qwerty.key_for('d').unwrap(), qwerty.key_for('f').unwrap()),
                20.0,
            );

            check!(metrics.others == 30.0);
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
            unigram_metrics: ReportUnigramMetrics {
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
            },
            bigram_metrics: ReportBigramMetrics {
                skips_1: 20.0,
                skips_n: 40.0,
                lateral_stretches: 60.0,
                scissors: 80.0,
                wide_scissors: 80.0,
                others: 40.0,
            },
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
                }
        );
    }
}
