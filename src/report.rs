use std::collections::HashMap;
use std::fmt::{Display, Formatter};

use crate::{
    analyzer::{Metric, Metrics},
    layout::{Finger, Hand},
    ngrams::Unigram,
};

#[derive(Default, Debug)]
pub struct ReportMetrics {
    total_chars: f64,
    unigram_metrics: ReportUnigramMetrics,
}

impl Metrics for ReportMetrics {
    fn collect_metric(&mut self, metric: Metric) {
        match metric {
            Metric::Unigram(unigram, count) => {
                self.unigram_metrics.collect(unigram, count);
            }
            Metric::Bigram(_bigram, _count) => {}
            Metric::Trigram(_trigram, _count) => {}
            Metric::CorpusLenght(total_chars) => {
                self.total_chars = total_chars;
            }
        }
    }
}

#[derive(Default, Debug)]
struct ReportUnigramMetrics {
    column_usage: HashMap<usize, f64>,
    row_usage: HashMap<usize, f64>,
    finger_usage: HashMap<Finger, f64>,
}

impl ReportUnigramMetrics {
    fn collect(&mut self, unigram: Unigram, count: f64) {
        *self.row_usage.entry(unigram.key.position.r).or_default() += count;
        *self.column_usage.entry(unigram.key.position.c).or_default() += count;
        *self.finger_usage.entry(unigram.key.finger).or_default() += count;
    }
}

#[derive(Debug, PartialEq)]
pub struct Report {
    left_hand_usage: f64,
    right_hand_usage: f64,
    finger_usage: HashMap<Finger, f64>,
    row_usage: HashMap<usize, f64>,
    column_usage: HashMap<usize, f64>,
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
        }
    }
}

impl Display for Report {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "Left hand usage: {}%", self.left_hand_usage)?;
        writeln!(f, "Right hand usage: {}%", self.right_hand_usage)?;
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
        Ok(())
    }
}

#[cfg(test)]
mod report_metrics_tests {
    use assert2::check;
    use rstest::rstest;

    use super::*;

    mod unigram_metrics_tests {
        use crate::layout::{Layout, fixtures::qwerty};

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
    }
}

#[cfg(test)]
mod report_tests {
    use assert2::check;

    use super::*;

    #[test]
    fn it_can_be_built_from_metrics() {
        let metrics = ReportMetrics {
            total_chars: 100.0,
            unigram_metrics: ReportUnigramMetrics {
                finger_usage: [
                    (1.into(), 20.0),
                    (2.into(), 40.0),
                    (3.into(), 60.0),
                    (8.into(), 80.0),
                ]
                .into(),
                column_usage: [(1, 20.0), (2, 40.0), (3, 60.0), (4, 80.0)].into(),
                row_usage: [(0, 20.0), (1, 40.0), (2, 60.0), (3, 80.0)].into(),
            },
        };

        let report = Report::from(metrics);

        check!(
            report
                == Report {
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
                }
        );
    }
}
