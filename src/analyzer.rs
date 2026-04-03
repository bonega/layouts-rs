use std::collections::HashMap;

use crate::{
    corpus::Corpus,
    layout::{Finger, Layout},
    ngrams::{Bigram, Trigram, Unigram},
};

struct Analyzer {
    corpus: Corpus,
}

impl Analyzer {
    fn new(corpus: Corpus) -> Self {
        Self { corpus }
    }

    fn analyze(&self, layout: &Layout, metrics: &mut impl Metrics) {
        for (char, count) in self.corpus.unigrams.iter() {
            let Some(key) = layout.key_for(*char) else {
                continue;
            };

            metrics.collect_metric(Metric::Unigram(Unigram::new(&key), *count));
        }

        for ((char1, char2), count) in self.corpus.bigrams.iter() {
            let Some((key1, key2)) = layout.key_for(*char1).zip(layout.key_for(*char2)) else {
                continue;
            };

            metrics.collect_metric(Metric::Bigram(Bigram::new(&key1, &key2), *count));
        }

        for ((char1, char2, char3), count) in self.corpus.trigrams.iter() {
            let Some((key1, key2, key3)) = layout
                .key_for(*char1)
                .zip(layout.key_for(*char2))
                .zip(layout.key_for(*char3))
                .map(|((k1, k2), k3)| (k1, k2, k3))
            else {
                continue;
            };

            metrics.collect_metric(Metric::Trigram(Trigram::new(&key1, &key2, &key3), *count));
        }
    }
}

#[cfg_attr(test, mockall::automock)]
trait Metrics {
    fn collect_metric(&mut self, metric: Metric);
}

#[derive(Debug, PartialEq)]
enum Metric {
    Trigram(Trigram, f64),
    Bigram(Bigram, f64),
    Unigram(Unigram, f64),
}

#[derive(Default)]
struct ReportMetrics {
    unigram_metrics: ReportUnigramMetrics,
}

impl Metrics for ReportMetrics {
    fn collect_metric(&mut self, metric: Metric) {
        match metric {
            Metric::Unigram(unigram, count) => {
                self.unigram_metrics.collect(unigram, count);
            }
            Metric::Bigram(bigram, count) => {}
            Metric::Trigram(trigram, count) => {}
        }
    }
}

#[derive(Default)]
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
mod tests {
    use assert2::check;
    use mockall::predicate::eq;
    use rstest::rstest;

    use crate::{corpus::Corpus, layout::fixtures::qwerty};

    use super::*;

    #[rstest]
    fn it_does_not_collect_metrics_for_not_existing_keys(qwerty: Layout) {
        let corpus = Corpus {
            chars_length: 100.0,
            word_items: vec![],
            unigrams: [('[', 10.0)].into(),
            bigrams: [(('[', '['), 10.0)].into(),
            trigrams: [(('[', '[', '['), 10.0)].into(),
        };

        let mut metrics = MockMetrics::new();
        metrics.expect_collect_metric().never();

        let analyzer = Analyzer::new(corpus);

        analyzer.analyze(&qwerty, &mut metrics)
    }

    #[rstest]
    fn it_collects_metrics(qwerty: Layout) {
        let corpus = Corpus {
            chars_length: 100.0,
            word_items: vec![],
            unigrams: [('a', 1.0)].into(),
            bigrams: [(('a', 'a'), 2.0)].into(),
            trigrams: [(('a', 'a', 'a'), 3.0)].into(),
        };

        let key = qwerty.key_for('a').unwrap();

        let mut metrics = MockMetrics::new();
        metrics
            .expect_collect_metric()
            .with(eq(Metric::Unigram(Unigram::new(&key), 1.0)))
            .once()
            .return_const(());
        metrics
            .expect_collect_metric()
            .with(eq(Metric::Bigram(Bigram::new(&key, &key), 2.0)))
            .once()
            .return_const(());
        metrics
            .expect_collect_metric()
            .with(eq(Metric::Trigram(Trigram::new(&key, &key, &key), 3.0)))
            .once()
            .return_const(());

        let analyzer = Analyzer::new(corpus);

        analyzer.analyze(&qwerty, &mut metrics)
    }

    #[rstest]
    fn it_generates_metrics(qwerty: Layout) {
        #[derive(Default, Debug, PartialEq)]
        struct FakeMetrics {
            unigrams: f64,
            bigrams: f64,
            trigrams: f64,
        }

        impl Metrics for FakeMetrics {
            fn collect_metric(&mut self, metric: Metric) {
                match metric {
                    Metric::Unigram(_, count) => {
                        self.unigrams += count;
                    }
                    Metric::Bigram(_, count) => {
                        self.bigrams += count;
                    }
                    Metric::Trigram(_, count) => {
                        self.trigrams += count;
                    }
                }
            }
        }

        let corpus = Corpus {
            chars_length: 100.0,
            word_items: vec![],
            unigrams: [('a', 1.0)].into(),
            bigrams: [(('a', 'a'), 2.0)].into(),
            trigrams: [(('a', 'a', 'a'), 3.0)].into(),
        };

        let mut fake_metrics = FakeMetrics::default();

        let analyzer = Analyzer::new(corpus);

        analyzer.analyze(&qwerty, &mut fake_metrics);

        check!(
            fake_metrics
                == FakeMetrics {
                    unigrams: 1.0,
                    bigrams: 2.0,
                    trigrams: 3.0,
                }
        );
    }
}

#[cfg(test)]
mod report_metrics_tests {
    use assert2::check;
    use rstest::rstest;

    use super::*;

    mod unigram_metrics_tests {
        use crate::layout::fixtures::qwerty;

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
