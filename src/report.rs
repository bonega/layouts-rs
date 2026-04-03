use std::collections::HashMap;

use crate::{
    analyzer::{Metric, Metrics},
    layout::Finger,
    ngrams::Unigram,
};

#[derive(Default, Debug)]
pub struct ReportMetrics {
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
