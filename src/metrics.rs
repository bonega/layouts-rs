use std::collections::HashMap;

use crate::ngrams::{Bigram, BigramKind, Trigram, TrigramKind, Unigram};

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

#[derive(Debug, PartialEq, Eq, Hash)]
pub enum NgramMetricType {
    BigramSkips1,
    BigramSkipsN,
    BigramLateralStretches,
    BigramScissors,
    BigramWideScissors,
    BigramOthers,
    TrigramSkipsSameHand,
    TrigramSkipsSameHand1,
    TrigramSkipsSameHandN,
    TrigramSkipsAlternation,
    TrigramSkipsAlternation1,
    TrigramSkipsAlternationN,
    TrigramLateralStretchesSameHand,
    TrigramLateralStretchesAlternation,
    TrigramScissorsSameHand1,
    TrigramScissorsSameHandN,
    TrigramScissorsAlternation1,
    TrigramScissorsAlternationN,
    TrigramRollIn,
    TrigramRollOut,
    TrigramRollInBigrams,
    TrigramRollOutBigrams,
    TrigramRedirectsWeak,
    TrigramRedirectsStrong,
    TrigramAlternations,
    TrigramOthers,
}

#[derive(Default)]
pub struct NgramKindMetrics {
    data: HashMap<NgramMetricType, f64>,
}

impl NgramKindMetrics {
    pub fn get(&self, metric_type: &NgramMetricType) -> f64 {
        *self.data.get(metric_type).unwrap_or(&0.0)
    }
}

impl Metrics for NgramKindMetrics {
    fn collect_metric(&mut self, metric: Metric) {
        match metric {
            Metric::Bigram(bigram, count) => {
                for kind in bigram.kinds {
                    let metric_type = match kind {
                        BigramKind::SameFingerSkip { units } => {
                            if units == 1 {
                                NgramMetricType::BigramSkips1
                            } else {
                                NgramMetricType::BigramSkipsN
                            }
                        }
                        BigramKind::LateralStretch { .. } => {
                            NgramMetricType::BigramLateralStretches
                        }
                        BigramKind::Scissor { units, .. } => {
                            if units >= 2 {
                                NgramMetricType::BigramWideScissors
                            } else {
                                NgramMetricType::BigramScissors
                            }
                        }
                        BigramKind::Other => NgramMetricType::BigramOthers,
                    };

                    *self.data.entry(metric_type).or_insert(0.0) += count;
                }
            }
            Metric::Trigram(trigram, count) => {
                for kind in trigram.kinds {
                    let metric_type = match kind {
                        TrigramKind::SameFingerSkip { units, same_hand } => {
                            if same_hand {
                                if units == 1 {
                                    NgramMetricType::TrigramSkipsSameHand1
                                } else {
                                    NgramMetricType::TrigramSkipsSameHandN
                                }
                            } else {
                                if units == 1 {
                                    NgramMetricType::TrigramSkipsAlternation1
                                } else {
                                    NgramMetricType::TrigramSkipsAlternationN
                                }
                            }
                        }
                        TrigramKind::LateralStretch { same_hand, .. } => {
                            if same_hand {
                                NgramMetricType::TrigramLateralStretchesSameHand
                            } else {
                                NgramMetricType::TrigramLateralStretchesAlternation
                            }
                        }
                        TrigramKind::Scissor {
                            units, same_hand, ..
                        } => {
                            if same_hand {
                                if units >= 2 {
                                    NgramMetricType::TrigramScissorsSameHandN
                                } else {
                                    NgramMetricType::TrigramScissorsSameHand1
                                }
                            } else if units >= 2 {
                                NgramMetricType::TrigramScissorsAlternationN
                            } else {
                                NgramMetricType::TrigramScissorsAlternation1
                            }
                        }
                        TrigramKind::Roll { triple, inward } => match (triple, inward) {
                            (true, true) => NgramMetricType::TrigramRollIn,
                            (true, false) => NgramMetricType::TrigramRollOut,
                            (false, true) => NgramMetricType::TrigramRollInBigrams,
                            (false, false) => NgramMetricType::TrigramRollOutBigrams,
                        },
                        TrigramKind::Redirect { weak } => {
                            if weak {
                                NgramMetricType::TrigramRedirectsWeak
                            } else {
                                NgramMetricType::TrigramRedirectsStrong
                            }
                        }
                        TrigramKind::Alternation => NgramMetricType::TrigramAlternations,
                        TrigramKind::Other => NgramMetricType::TrigramOthers,
                    };

                    *self.data.entry(metric_type).or_insert(0.0) += count;
                }
            }
            Metric::CorpusLenght(_) => {}
            Metric::Unigram(_, _) => {}
        }
    }
}
