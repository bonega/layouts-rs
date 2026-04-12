use std::collections::HashMap;
use std::fmt;

use crate::{
    layout::{Finger, Hand},
    metrics::SimpleMetrics,
};

#[derive(Debug, Default, PartialEq)]
pub struct SimpleStats {
    pub finger_usage: HashMap<Finger, f64>,
    pub row_usage: HashMap<usize, f64>,
    pub column_usage: HashMap<usize, f64>,
    pub total_chars: f64,
    pub bigram_lateral_stretches: f64,
    pub bigram_scissors: f64,
    pub bigram_skips_1: f64,
    pub bigram_skips_n: f64,
    pub bigram_wide_scissors: f64,
    pub bigram_others: f64,
    pub effort: f64,
    pub left_hand_usage: f64,
    pub right_hand_usage: f64,
    pub pinky_off_home: f64,
    pub trigram_alternations: f64,
    pub trigram_lateral_stretches_alternation: f64,
    pub trigram_lateral_stretches_same_hand: f64,
    pub trigram_redirects_strong: f64,
    pub trigram_redirects_weak: f64,
    pub trigram_scissors_alternation_1: f64,
    pub trigram_scissors_alternation_n: f64,
    pub trigram_scissors_same_hand_1: f64,
    pub trigram_scissors_same_hand_n: f64,
    pub trigram_skips_alternation: f64,
    pub trigram_skips_alternation_1: f64,
    pub trigram_skips_alternation_n: f64,
    pub trigram_roll_in: f64,
    pub trigram_roll_out: f64,
    pub trigram_roll_in_bigrams: f64,
    pub trigram_roll_out_bigrams: f64,
    pub trigram_skips_same_hand: f64,
    pub trigram_skips_same_hand_1: f64,
    pub trigram_skips_same_hand_n: f64,
    pub trigram_others: f64,
}

impl SimpleStats {
    pub fn trigram_roll_ratio(&self) -> f64 {
        let total_roll = self.trigram_roll_in + self.trigram_roll_out;

        if total_roll > 0.0 {
            100.0 * self.trigram_roll_in / total_roll
        } else {
            50.0
        }
    }
}

impl From<SimpleMetrics> for SimpleStats {
    fn from(metrics: SimpleMetrics) -> Self {
        let pct = 100.0 / metrics.total_chars;

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
            total_chars: metrics.total_chars,
            effort: pct * metrics.effort,
            left_hand_usage: 100.0 * left_hand_usage / total_hand_usage,
            right_hand_usage: 100.0 * right_hand_usage / total_hand_usage,
            pinky_off_home: pct * metrics.pinky_off_home,
            bigram_skips_1: pct * metrics.bigram_skips_1,
            bigram_skips_n: pct * metrics.bigram_skips_n,
            bigram_lateral_stretches: pct * metrics.bigram_lateral_stretches,
            bigram_scissors: pct * metrics.bigram_scissors,
            bigram_wide_scissors: pct * metrics.bigram_wide_scissors,
            trigram_skips_same_hand: pct * metrics.trigram_skips_same_hand,
            trigram_skips_same_hand_1: pct * metrics.trigram_skips_same_hand_1,
            trigram_skips_same_hand_n: pct * metrics.trigram_skips_same_hand_n,
            trigram_skips_alternation: pct * metrics.trigram_skips_alternation,
            trigram_skips_alternation_1: pct * metrics.trigram_skips_alternation_1,
            trigram_skips_alternation_n: pct * metrics.trigram_skips_alternation_n,
            trigram_lateral_stretches_same_hand: pct * metrics.trigram_lateral_stretches_same_hand,
            trigram_lateral_stretches_alternation: pct
                * metrics.trigram_lateral_stretches_alternation,
            trigram_scissors_same_hand_1: pct * metrics.trigram_scissors_same_hand_1,
            trigram_scissors_same_hand_n: pct * metrics.trigram_scissors_same_hand_n,
            trigram_scissors_alternation_1: pct * metrics.trigram_scissors_alternation_1,
            trigram_scissors_alternation_n: pct * metrics.trigram_scissors_alternation_n,
            trigram_redirects_weak: pct * metrics.trigram_redirects_weak,
            trigram_redirects_strong: pct * metrics.trigram_redirects_strong,
            trigram_alternations: pct * metrics.trigram_alternations,
            bigram_others: pct * metrics.bigram_others,
            trigram_others: pct * metrics.trigram_others,
            trigram_roll_in: pct * metrics.trigram_roll_in,
            trigram_roll_out: pct * metrics.trigram_roll_out,
            trigram_roll_in_bigrams: pct * metrics.trigram_roll_in_bigrams,
            trigram_roll_out_bigrams: pct * metrics.trigram_roll_out_bigrams,
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
        }
    }
}

impl fmt::Display for SimpleStats {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
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
        writeln!(f, "  Skips 1: {:.2}%", self.bigram_skips_1)?;
        writeln!(f, "  Skips n: {:.2}%", self.bigram_skips_n)?;
        writeln!(
            f,
            "  Lateral stretches: {:.2}%",
            self.bigram_lateral_stretches
        )?;
        writeln!(f, "  Scissors: {:.2}%", self.bigram_scissors)?;
        writeln!(f, "  Scissors wide: {:.2}%", self.bigram_wide_scissors)?;
        writeln!(f, "  Others: {:.2}%", self.bigram_others)?;
        writeln!(f, "Trigram metrics:")?;
        writeln!(
            f,
            "  Same-hand skips 1: {:.2}%",
            self.trigram_skips_same_hand_1
        )?;
        writeln!(
            f,
            "  Same-hand skips n: {:.2}%",
            self.trigram_skips_same_hand_n
        )?;
        writeln!(
            f,
            "  Alternation skips 1: {:.2}%",
            self.trigram_skips_alternation_1
        )?;
        writeln!(
            f,
            "  Alternation skips n: {:.2}%",
            self.trigram_skips_alternation_n
        )?;
        writeln!(
            f,
            "  Lateral stretches (same hand): {:.2}%",
            self.trigram_lateral_stretches_same_hand
        )?;
        writeln!(
            f,
            "  Lateral stretches (alternation): {:.2}%",
            self.trigram_lateral_stretches_alternation
        )?;
        writeln!(
            f,
            "  Scissors (same hand 1): {:.2}%",
            self.trigram_scissors_same_hand_1
        )?;
        writeln!(
            f,
            "  Scissors (same hand n): {:.2}%",
            self.trigram_scissors_same_hand_n
        )?;
        writeln!(
            f,
            "  Scissors (alternation 1): {:.2}%",
            self.trigram_scissors_alternation_1
        )?;
        writeln!(
            f,
            "  Scissors (alternation n): {:.2}%",
            self.trigram_scissors_alternation_n
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
        writeln!(
            f,
            "  Alternations (total): {:.2}%",
            self.trigram_alternations
        )?;
        writeln!(f, "  Others: {:.2}%", self.trigram_others)?;
        Ok(())
    }
}

#[cfg(test)]
mod simple_stats_tests {
    use assert2::check;

    use super::*;

    #[test]
    fn it_builds_stats_from_simple_metrics() {
        let metrics = SimpleMetrics {
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
            bigram_wide_scissors: 80.0,
            bigram_others: 90.0,

            trigram_skips_same_hand: 10.0,
            trigram_skips_same_hand_1: 20.0,
            trigram_skips_same_hand_n: 30.0,
            trigram_skips_alternation: 40.0,
            trigram_skips_alternation_1: 50.0,
            trigram_skips_alternation_n: 60.0,
            trigram_roll_in: 80.0,
            trigram_roll_out: 120.0,
            trigram_redirects_weak: 90.0,
            trigram_redirects_strong: 100.0,
            trigram_alternations: 110.0,
            trigram_lateral_stretches_same_hand: 120.0,
            trigram_lateral_stretches_alternation: 130.0,
            trigram_scissors_same_hand_1: 140.0,
            trigram_scissors_same_hand_n: 150.0,
            trigram_scissors_alternation_1: 160.0,
            trigram_scissors_alternation_n: 170.0,
            trigram_roll_in_bigrams: 180.0,
            trigram_roll_out_bigrams: 190.0,
            trigram_others: 200.0,
        };

        let stats = SimpleStats::from(metrics);

        check!(
            stats
                == SimpleStats {
                    total_chars: 200.0,
                    column_usage: [(0, 80.0), (1, 20.0)].into(),
                    row_usage: [(0, 25.0), (1, 75.0)].into(),
                    finger_usage: [(1.into(), 20.0), (2.into(), 20.0), (8.into(), 60.0)].into(),
                    effort: 5.0,
                    left_hand_usage: 40.0,
                    right_hand_usage: 60.0,
                    pinky_off_home: 15.0,
                    bigram_skips_1: 20.0,
                    bigram_skips_n: 25.0,
                    bigram_lateral_stretches: 30.0,
                    bigram_scissors: 35.0,
                    bigram_wide_scissors: 40.0,
                    bigram_others: 45.0,
                    trigram_skips_same_hand: 5.0,
                    trigram_skips_same_hand_1: 10.0,
                    trigram_skips_same_hand_n: 15.0,
                    trigram_skips_alternation: 20.0,
                    trigram_skips_alternation_1: 25.0,
                    trigram_skips_alternation_n: 30.0,
                    trigram_roll_in: 40.0,
                    trigram_roll_out: 60.0,
                    trigram_redirects_weak: 45.0,
                    trigram_redirects_strong: 50.0,
                    trigram_alternations: 55.0,
                    trigram_lateral_stretches_same_hand: 60.0,
                    trigram_lateral_stretches_alternation: 65.0,
                    trigram_scissors_same_hand_1: 70.0,
                    trigram_scissors_same_hand_n: 75.0,
                    trigram_scissors_alternation_1: 80.0,
                    trigram_scissors_alternation_n: 85.0,
                    trigram_roll_in_bigrams: 90.0,
                    trigram_roll_out_bigrams: 95.0,
                    trigram_others: 100.0,
                }
        );

        check!(stats.trigram_roll_ratio() == 40.0);
    }
}
