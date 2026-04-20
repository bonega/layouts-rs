include!(concat!(env!("OUT_DIR"), "/metrics.rs"));

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
            let mut metrics = Metrics::default();

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
            let mut metrics = Metrics::default();

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
            let mut metrics = Metrics::default();

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
            let mut metrics = Metrics::default();

            metrics.collect_metric(Metric::Unigram(ngram!(qwerty, 'a'), 1000.0));
            metrics.collect_metric(Metric::Unigram(ngram!(qwerty, 'q'), 1.0));
            metrics.collect_metric(Metric::Unigram(ngram!(qwerty, 'z'), 10.0));
            metrics.collect_metric(Metric::Unigram(ngram!(qwerty, '"'), 100.0));

            check!(metrics.pinky_off_home == 111.0);
        }

        #[rstest]
        fn it_collects_key_effort(qwerty: Layout) {
            let mut metrics = Metrics::default();

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
            let mut metrics = Metrics::default();

            metrics.collect_metric(Metric::Bigram(ngram!(qwerty, 'q', 'a'), 10.0));
            metrics.collect_metric(Metric::Bigram(ngram!(qwerty, 'q', 'z'), 20.0));

            check!(metrics.bigram_skips_1 == 10.0);
            check!(metrics.bigram_skips_n == 20.0);
        }

        #[rstest]
        fn it_collects_lateral_stretches(qwerty: Layout) {
            let mut metrics = Metrics::default();

            metrics.collect_metric(Metric::Bigram(ngram!(qwerty, 'd', 'g'), 10.0));
            metrics.collect_metric(Metric::Bigram(ngram!(qwerty, 's', '"'), 20.0));

            check!(metrics.bigram_lateral_stretches == 30.0);
        }

        #[rstest]
        fn it_collects_scissors(qwerty: Layout) {
            let mut metrics = Metrics::default();

            metrics.collect_metric(Metric::Bigram(ngram!(qwerty, 'c', 'w'), 10.0));
            metrics.collect_metric(Metric::Bigram(ngram!(qwerty, 'c', 's'), 20.0));

            check!(metrics.bigram_scissors_wide == 10.0);
            check!(metrics.bigram_scissors == 20.0);
        }

        #[rstest]
        fn it_collects_others(qwerty: Layout) {
            let mut metrics = Metrics::default();

            metrics.collect_metric(Metric::Bigram(ngram!(qwerty, 'd', 'h'), 10.0));
            metrics.collect_metric(Metric::Bigram(ngram!(qwerty, 'd', 'f'), 20.0));

            check!(metrics.bigram_others == 30.0);
        }
    }

    mod trigram_metrics_tests {
        use super::*;

        #[rstest]
        fn it_collects_same_finger_skips(qwerty: Layout) {
            let mut metrics = Metrics::default();

            metrics.collect_metric(Metric::Trigram(ngram!(qwerty, 'q', 'w', 'a'), 10.0));
            metrics.collect_metric(Metric::Trigram(ngram!(qwerty, 'q', 'h', 'a'), 20.0));

            check!(metrics.trigram_skips_1_same_hand == 10.0);
            check!(metrics.trigram_skips_n_same_hand == 0.0);
            check!(metrics.trigram_skips_1_alternation == 20.0);
            check!(metrics.trigram_skips_n_alternation == 0.0);
        }

        #[rstest]
        fn it_collects_trigram_lateral_stretches(qwerty: Layout) {
            let mut metrics = Metrics::default();

            metrics.collect_metric(Metric::Trigram(ngram!(qwerty, 'l', 'i', '\''), 10.0));
            metrics.collect_metric(Metric::Trigram(ngram!(qwerty, 'd', 'u', 'g'), 20.0));

            check!(metrics.trigram_lateral_stretches_same_hand == 10.0);
            check!(metrics.trigram_lateral_stretches_alternation == 20.0);
        }

        #[rstest]
        fn it_collects_trigram_scissors(qwerty: Layout) {
            let mut metrics = Metrics::default();

            metrics.collect_metric(Metric::Trigram(ngram!(qwerty, 'c', 'a', 'w'), 10.0));
            metrics.collect_metric(Metric::Trigram(ngram!(qwerty, 'c', 'j', 'w'), 20.0));

            check!(metrics.trigram_scissors_same_hand == 0.0);
            check!(metrics.trigram_scissors_wide_same_hand == 10.0);
            check!(metrics.trigram_scissors_alternation == 0.0);
            check!(metrics.trigram_scissors_wide_alternation == 20.0);
        }

        #[rstest]
        fn it_collects_rolls(qwerty: Layout) {
            let mut metrics = Metrics::default();

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
            let mut metrics = Metrics::default();

            metrics.collect_metric(Metric::Trigram(ngram!(qwerty, 'q', 't', 'e'), 10.0));
            metrics.collect_metric(Metric::Trigram(ngram!(qwerty, 'q', 'e', 'w'), 20.0));

            check!(metrics.trigram_redirects_weak == 10.0);
            check!(metrics.trigram_redirects_strong == 20.0);
        }

        #[rstest]
        fn it_collects_alternations_and_others(qwerty: Layout) {
            let mut metrics = Metrics::default();

            metrics.collect_metric(Metric::Trigram(ngram!(qwerty, 'q', 'h', 'w'), 10.0));
            metrics.collect_metric(Metric::Trigram(ngram!(qwerty, 'q', 'q', 'a'), 20.0));

            check!(metrics.trigram_alternations == 10.0);
            check!(metrics.trigram_others == 20.0);
        }
    }
}
