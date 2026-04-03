use crate::{
    corpus::Corpus,
    layout::Layout,
    ngrams::{Bigram, Trigram, Unigram},
};

pub struct Analyzer {
    corpus: Corpus,
}

impl Analyzer {
    pub fn new(corpus: Corpus) -> Self {
        Self { corpus }
    }

    pub fn analyze(&self, layout: &Layout, metrics: &mut impl Metrics) {
        metrics.collect_metric(Metric::CorpusLenght(self.corpus.chars_length));

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
pub trait Metrics {
    fn collect_metric(&mut self, metric: Metric);
}

#[derive(Debug, PartialEq)]
pub enum Metric {
    Trigram(Trigram, f64),
    Bigram(Bigram, f64),
    Unigram(Unigram, f64),
    CorpusLenght(f64),
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
        metrics
            .expect_collect_metric()
            .with(eq(Metric::CorpusLenght(100.0)))
            .once()
            .return_const(());

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
            .with(eq(Metric::CorpusLenght(100.0)))
            .once()
            .return_const(());
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
            total_chars: f64,
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
                    Metric::CorpusLenght(length) => {
                        self.total_chars = length;
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
                    total_chars: 100.0,
                }
        );
    }
}
