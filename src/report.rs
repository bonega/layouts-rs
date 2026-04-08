use std::collections::HashMap;
use std::fmt::{Display, Formatter};

use crate::{
    analyzer::{Metric, Metrics},
    layout::{Finger, FingerKind, Hand},
    ngrams::{Bigram, BigramKind, Trigram, TrigramKind, Unigram},
};

#[derive(Default, Debug)]
pub struct ReportMetrics {
    total_chars: f64,
    unigram_metrics: ReportUnigramMetrics,
    bigram_metrics: ReportBigramMetrics,
    trigram_metrics: ReportTrigramMetrics,
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
            Metric::Trigram(trigram, count) => {
                self.trigram_metrics.collect(trigram, count);
            }
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

#[derive(Default, Debug, PartialEq)]
struct ReportTrigramMetrics {
    same_hand_skips: f64,
    alternation_skips: f64,
    roll_in: f64,
    roll_out: f64,
    roll_in_bigrams: f64,
    roll_out_bigrams: f64,
    redirects_weak: f64,
    redirects_strong: f64,
    alternations: f64,
    others: f64,
}

impl ReportTrigramMetrics {
    fn collect(&mut self, trigram: Trigram, count: f64) {
        match trigram.kind {
            TrigramKind::SameFingerSkip { same_hand, .. } => {
                if same_hand {
                    self.same_hand_skips += count;
                } else {
                    self.alternation_skips += count;
                }
            }
            TrigramKind::Roll { triple, inward } => match (triple, inward) {
                (true, true) => self.roll_in += count,
                (true, false) => self.roll_out += count,
                (false, true) => self.roll_in_bigrams += count,
                (false, false) => self.roll_out_bigrams += count,
            },
            TrigramKind::RollIn { triple } => {
                if triple {
                    self.roll_in += count;
                } else {
                    self.roll_in_bigrams += count;
                }
            }
            TrigramKind::RollOut { triple } => {
                if triple {
                    self.roll_out += count;
                } else {
                    self.roll_out_bigrams += count;
                }
            }
            TrigramKind::Redirect { weak } => {
                if weak {
                    self.redirects_weak += count;
                } else {
                    self.redirects_strong += count;
                }
            }
            TrigramKind::Alternation => {
                self.alternations += count;
            }
            TrigramKind::Other => {
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
            trigram_same_hand_skips: 100.0 * metrics.trigram_metrics.same_hand_skips
                / metrics.total_chars,
            trigram_alternation_skips: 100.0 * metrics.trigram_metrics.alternation_skips
                / metrics.total_chars,
            trigram_roll_in: 100.0 * metrics.trigram_metrics.roll_in / metrics.total_chars,
            trigram_roll_out: 100.0 * metrics.trigram_metrics.roll_out / metrics.total_chars,
            trigram_roll_in_bigrams: 100.0 * metrics.trigram_metrics.roll_in_bigrams
                / metrics.total_chars,
            trigram_roll_out_bigrams: 100.0 * metrics.trigram_metrics.roll_out_bigrams
                / metrics.total_chars,
            trigram_redirects_weak: 100.0 * metrics.trigram_metrics.redirects_weak
                / metrics.total_chars,
            trigram_redirects_strong: 100.0 * metrics.trigram_metrics.redirects_strong
                / metrics.total_chars,
            trigram_alternations: 100.0 * metrics.trigram_metrics.alternations
                / metrics.total_chars,
            trigram_others: 100.0 * metrics.trigram_metrics.others / metrics.total_chars,
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

    mod trigram_metrics_tests {
        use crate::ngrams::Trigram;

        use super::*;

        #[rstest]
        fn it_collects_same_finger_skips(qwerty: Layout) {
            let mut metrics = ReportTrigramMetrics::default();

            metrics.collect(
                Trigram::new(
                    qwerty.key_for('q').unwrap(),
                    qwerty.key_for('w').unwrap(),
                    qwerty.key_for('a').unwrap(),
                ),
                10.0,
            );
            metrics.collect(
                Trigram::new(
                    qwerty.key_for('q').unwrap(),
                    qwerty.key_for('h').unwrap(),
                    qwerty.key_for('a').unwrap(),
                ),
                20.0,
            );

            check!(metrics.same_hand_skips == 10.0);
            check!(metrics.alternation_skips == 20.0);
        }

        #[rstest]
        fn it_collects_rolls(qwerty: Layout) {
            let mut metrics = ReportTrigramMetrics::default();

            metrics.collect(
                Trigram::new(
                    qwerty.key_for('q').unwrap(),
                    qwerty.key_for('w').unwrap(),
                    qwerty.key_for('e').unwrap(),
                ),
                10.0,
            );
            metrics.collect(
                Trigram::new(
                    qwerty.key_for('t').unwrap(),
                    qwerty.key_for('e').unwrap(),
                    qwerty.key_for('q').unwrap(),
                ),
                20.0,
            );
            metrics.collect(
                Trigram::new(
                    qwerty.key_for('q').unwrap(),
                    qwerty.key_for('w').unwrap(),
                    qwerty.key_for('p').unwrap(),
                ),
                30.0,
            );
            metrics.collect(
                Trigram::new(
                    qwerty.key_for('t').unwrap(),
                    qwerty.key_for('e').unwrap(),
                    qwerty.key_for('p').unwrap(),
                ),
                40.0,
            );

            check!(metrics.roll_in == 10.0);
            check!(metrics.roll_out == 20.0);
            check!(metrics.roll_in_bigrams == 30.0);
            check!(metrics.roll_out_bigrams == 40.0);
        }

        #[rstest]
        fn it_collects_redirects(qwerty: Layout) {
            let mut metrics = ReportTrigramMetrics::default();

            metrics.collect(
                Trigram::new(
                    qwerty.key_for('q').unwrap(),
                    qwerty.key_for('t').unwrap(),
                    qwerty.key_for('e').unwrap(),
                ),
                10.0,
            );
            metrics.collect(
                Trigram::new(
                    qwerty.key_for('q').unwrap(),
                    qwerty.key_for('e').unwrap(),
                    qwerty.key_for('w').unwrap(),
                ),
                20.0,
            );

            check!(metrics.redirects_weak == 10.0);
            check!(metrics.redirects_strong == 20.0);
        }

        #[rstest]
        fn it_collects_alternations_and_others(qwerty: Layout) {
            let mut metrics = ReportTrigramMetrics::default();

            metrics.collect(
                Trigram::new(
                    qwerty.key_for('q').unwrap(),
                    qwerty.key_for('h').unwrap(),
                    qwerty.key_for('w').unwrap(),
                ),
                10.0,
            );
            metrics.collect(
                Trigram::new(
                    qwerty.key_for('q').unwrap(),
                    qwerty.key_for('q').unwrap(),
                    qwerty.key_for('a').unwrap(),
                ),
                20.0,
            );

            check!(metrics.alternations == 10.0);
            check!(metrics.others == 20.0);
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
            trigram_metrics: ReportTrigramMetrics {
                same_hand_skips: 10.0,
                alternation_skips: 20.0,
                roll_in: 30.0,
                roll_out: 40.0,
                roll_in_bigrams: 50.0,
                roll_out_bigrams: 60.0,
                redirects_weak: 70.0,
                redirects_strong: 80.0,
                alternations: 90.0,
                others: 100.0,
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
