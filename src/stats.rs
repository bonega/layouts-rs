use std::fmt;
use std::hash::Hash;

use indexmap::IndexMap;

use crate::layout::*;
use crate::metrics::*;

include!(concat!(env!("OUT_DIR"), "/stats.rs"));

#[cfg(test)]
mod stats_tests {
    use assert2::check;

    use super::*;

    #[test]
    fn it_builds_stats_from_simple_metrics() {
        let metrics = Metrics {
            total_chars: 200.0,
            effort: 10.0,
            column_usage: [(0, 160.0), (1, 40.0)].into(),
            row_usage: [(0, 50.0), (1, 150.0)].into(),
            finger_usage: [(1.into(), 40.0), (2.into(), 40.0), (8.into(), 120.0)].into(),
            pinky_off_home: 30.0,

            bigram_skips_1: 40.0,
            bigram_skips_n: 50.0,
            bigram_lateral_stretches: 60.0,
            bigram_scissors: 70.0,
            bigram_scissors_wide: 80.0,
            bigram_others: 90.0,

            trigram_skips_1_same_hand: 20.0,
            trigram_skips_n_same_hand: 30.0,
            trigram_skips_1_alternation: 50.0,
            trigram_skips_n_alternation: 60.0,
            trigram_roll_in: 80.0,
            trigram_roll_out: 120.0,
            trigram_redirects_weak: 90.0,
            trigram_redirects_strong: 100.0,
            trigram_alternations: 110.0,
            trigram_lateral_stretches_same_hand: 120.0,
            trigram_lateral_stretches_alternation: 130.0,
            trigram_scissors_same_hand: 140.0,
            trigram_scissors_wide_same_hand: 150.0,
            trigram_scissors_alternation: 160.0,
            trigram_scissors_wide_alternation: 170.0,
            trigram_roll_in_bigrams: 180.0,
            trigram_roll_out_bigrams: 190.0,
            trigram_others: 200.0,
        };

        let stats = Stats::from(metrics);

        check!(
            stats
                == Stats {
                    general: GeneralStats {
                        total_chars: 200.0,
                        effort: 5.0,
                        pinky_off_home: 15.0,
                        finger_usage: [(1.into(), 20.0), (2.into(), 20.0), (8.into(), 60.0)].into(),
                        row_usage: [(0, 25.0), (1, 75.0)].into(),
                        column_usage: [(0, 80.0), (1, 20.0)].into(),
                        left_hand_usage: 40.0,
                        right_hand_usage: 60.0,
                    },
                    bigram: BigramStats {
                        skips_1: 20.0,
                        skips_n: 25.0,
                        lateral_stretches: 30.0,
                        scissors: 35.0,
                        scissors_wide: 40.0,
                        others: 45.0,
                    },
                    trigram: TrigramStats {
                        skips_1_same_hand: 10.0,
                        skips_1_alternation: 25.0,
                        skips_n_same_hand: 15.0,
                        skips_n_alternation: 30.0,
                        scissors_same_hand: 70.0,
                        scissors_alternation: 80.0,
                        scissors_wide_same_hand: 75.0,
                        scissors_wide_alternation: 85.0,
                        lateral_stretches_same_hand: 60.0,
                        lateral_stretches_alternation: 65.0,
                        redirects_strong: 50.0,
                        redirects_weak: 45.0,
                        roll_in: 40.0,
                        roll_out: 60.0,
                        roll_ratio: 40.0,
                        roll_in_bigrams: 90.0,
                        roll_out_bigrams: 95.0,
                        roll_ratio_bigrams: 100.0 * 180.0 / (180.0 + 190.0),
                        alternations: 55.0,
                        others: 100.0,
                    }
                }
        );
    }
}
