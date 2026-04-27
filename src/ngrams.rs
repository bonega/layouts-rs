use arrayvec::ArrayVec;

use crate::layout::{FingerKind, Key};

// Based on https://docs.google.com/document/d/1W0jhfqJI2ueJ2FNseR4YAFpNfsUM-_FlREHbpNGmC2o

const LATERAL_STRETCH_PAIRS: [(FingerKind, FingerKind); 6] = [
    (FingerKind::Index, FingerKind::Middle),
    (FingerKind::Ring, FingerKind::Middle),
    (FingerKind::Ring, FingerKind::Index),
    (FingerKind::Pinky, FingerKind::Ring),
    (FingerKind::Pinky, FingerKind::Middle),
    (FingerKind::Pinky, FingerKind::Index),
];

#[derive(PartialEq, Debug)]
pub struct Trigram {
    pub kinds: TrigramKinds,
    pub key1: Key,
    pub key2: Key,
    pub key3: Key,
}

#[derive(PartialEq, Debug, Clone, Copy, Eq, Hash, PartialOrd)]
pub enum Handedness {
    Same,
    Alternate,
}

#[derive(PartialEq, Debug, Clone, Copy, Eq, Hash, PartialOrd)]
pub enum RollDirection {
    In,
    Out,
}

#[derive(PartialEq, Debug, Clone, Copy, Eq, Hash, PartialOrd)]
pub enum RedirectStrength {
    Weak,
    Strong,
}

#[derive(PartialEq, Debug, Clone)]
pub enum TrigramKind {
    SameFingerSkip {
        units: u8,
        handedness: Handedness,
    },
    LateralStretch {
        finger: FingerKind,
        units: u8,
        handedness: Handedness,
    },
    Scissor {
        units: u8,
        upper_finger: FingerKind,
        lower_finger: FingerKind,
        handedness: Handedness,
        adiacent: bool,
    },
    Roll {
        length: u8,
        direction: RollDirection,
    },
    Redirect {
        strength: RedirectStrength,
    },
    Alternation,
    Other,
}

pub type TrigramKinds = ArrayVec<TrigramKind, 4>;

impl Trigram {
    pub fn new(key1: &Key, key2: &Key, key3: &Key) -> Self {
        Self {
            kinds: Self::find_kinds(key1, key2, key3),
            key1: *key1,
            key2: *key2,
            key3: *key3,
        }
    }

    fn find_kinds(key1: &Key, key2: &Key, key3: &Key) -> TrigramKinds {
        let mut kinds = TrigramKinds::new();

        if !key1.same_finger(key2) && !key3.same_finger(key2) {
            let handedness = if key2.finger.hand == key1.finger.hand {
                Handedness::Same
            } else {
                Handedness::Alternate
            };
            let bigram_kinds = Bigram::find_kinds(key1, key3);

            for kind in bigram_kinds {
                match kind {
                    BigramKind::SameFingerSkip { units } => {
                        kinds.push(TrigramKind::SameFingerSkip { units, handedness })
                    }
                    BigramKind::LateralStretch { finger, units } => {
                        kinds.push(TrigramKind::LateralStretch {
                            finger,
                            units,
                            handedness,
                        });
                    }
                    BigramKind::Scissor {
                        units,
                        upper_finger,
                        lower_finger,
                        adiacent,
                    } => {
                        kinds.push(TrigramKind::Scissor {
                            units,
                            upper_finger,
                            lower_finger,
                            handedness,
                            adiacent,
                        });
                    }
                    BigramKind::Other => {}
                }
            }
        };

        if key1.finger.hand == key2.finger.hand && key2.finger.hand == key3.finger.hand {
            if key1.finger < key2.finger && key2.finger < key3.finger {
                kinds.push(TrigramKind::Roll {
                    length: 3,
                    direction: RollDirection::In,
                });
            }
            if key1.finger > key2.finger && key2.finger > key3.finger {
                kinds.push(TrigramKind::Roll {
                    length: 3,
                    direction: RollDirection::Out,
                });
            }

            if (key1.finger < key2.finger && key3.finger < key2.finger
                || key1.finger > key2.finger && key3.finger > key2.finger)
                && !key1.same_finger(key3)
            {
                if ![FingerKind::Thumb, FingerKind::Index].contains(&key1.finger.kind)
                    && ![FingerKind::Thumb, FingerKind::Index].contains(&key2.finger.kind)
                    && ![FingerKind::Thumb, FingerKind::Index].contains(&key3.finger.kind)
                {
                    kinds.push(TrigramKind::Redirect {
                        strength: RedirectStrength::Weak,
                    });
                } else {
                    kinds.push(TrigramKind::Redirect {
                        strength: RedirectStrength::Strong,
                    });
                }
            }
        }

        if key1.finger.hand == key2.finger.hand && key2.finger.hand != key3.finger.hand {
            if key1.finger < key2.finger {
                kinds.push(TrigramKind::Roll {
                    length: 2,
                    direction: RollDirection::In,
                });
            }
            if key1.finger > key2.finger {
                kinds.push(TrigramKind::Roll {
                    length: 2,
                    direction: RollDirection::Out,
                });
            }
        }

        if key1.finger.hand == key3.finger.hand
            && key2.finger.hand != key1.finger.hand
            && !key1.same_finger(key3)
        {
            kinds.push(TrigramKind::Alternation);
        }

        if kinds.is_empty() {
            kinds.push(TrigramKind::Other);
        }

        kinds
    }
}

#[derive(PartialEq, Debug)]
pub struct Bigram {
    pub kinds: BigramKinds,
    pub key1: Key,
    pub key2: Key,
}

#[derive(PartialEq, Debug, Clone)]
pub enum BigramKind {
    SameFingerSkip {
        units: u8,
    },
    LateralStretch {
        finger: FingerKind,
        units: u8,
    },
    Scissor {
        units: u8,
        upper_finger: FingerKind,
        lower_finger: FingerKind,
        adiacent: bool,
    },
    Other,
}

pub type BigramKinds = ArrayVec<BigramKind, 4>;

impl Bigram {
    pub fn new(key1: &Key, key2: &Key) -> Self {
        Self {
            kinds: Self::find_kinds(key1, key2),
            key1: *key1,
            key2: *key2,
        }
    }

    fn find_kinds(key1: &Key, key2: &Key) -> BigramKinds {
        let row_distance = key1.row_distance(key2);
        let col_distance = key1.column_distance(key2);
        let finger_distance = key1.finger.distance(&key2.finger);

        let mut kinds = BigramKinds::new();

        if [key1.finger.kind, key2.finger.kind].contains(&FingerKind::Thumb) {
            return vec![BigramKind::Other].into_iter().collect();
        }

        if key1.same_finger(key2) && (row_distance > 0.0 || col_distance > 0.0) {
            kinds.push(BigramKind::SameFingerSkip {
                units: key1.distance(key2) as u8,
            });
        }

        // This differ from the doc definition (which uses 2.0 for adiacent keys and 3.5 for non
        // adiacent; it also does not define any rule for pair like index pinky
        if let Some(finger_distance) = finger_distance
            && finger_distance > 0
            && col_distance >= (finger_distance as f64 + 1.0)
        {
            if LATERAL_STRETCH_PAIRS.contains(&(key1.finger.kind, key2.finger.kind)) {
                kinds.push(BigramKind::LateralStretch {
                    finger: key1.finger.kind,
                    units: col_distance as u8,
                });
            }

            if LATERAL_STRETCH_PAIRS.contains(&(key2.finger.kind, key1.finger.kind)) {
                kinds.push(BigramKind::LateralStretch {
                    finger: key2.finger.kind,
                    units: col_distance as u8,
                });
            }
        }

        if let Some(finger_distance) = finger_distance
            && finger_distance >= 1
            && row_distance >= 1.0
        {
            let (upper, lower) = if key1.position.r < key2.position.r {
                (key1, key2)
            } else {
                (key2, key1)
            };

            kinds.push(BigramKind::Scissor {
                units: row_distance as u8,
                upper_finger: upper.finger.kind,
                lower_finger: lower.finger.kind,
                adiacent: finger_distance == 1,
            });
        }

        if kinds.is_empty() {
            kinds.push(BigramKind::Other);
        }

        kinds
    }
}

#[derive(PartialEq, Debug)]
pub struct Unigram {
    pub key: Key,
}

impl Unigram {
    pub fn new(key: &Key) -> Self {
        Self { key: *key }
    }
}

#[cfg(test)]
mod bigram_tests {
    use assert2::check;
    use rstest::rstest;

    use super::*;
    use crate::layout::{Layout, fixtures::qwerty};

    #[rstest]
    fn it_returns_the_bigram_with_keys(qwerty: Layout) {
        let key1 = *qwerty.key_for('a').unwrap();
        let key2 = *qwerty.key_for('s').unwrap();
        let mut kinds = BigramKinds::new();
        kinds.push(BigramKind::Other);

        check!(Bigram::new(&key1, &key2) == Bigram { kinds, key1, key2 });
    }

    #[rstest]
    #[case::left_1_vertical('q', 'a', vec![BigramKind::SameFingerSkip { units: 1 }])]
    #[case::left_2_vertical('q', 'z', vec![BigramKind::SameFingerSkip { units: 2 }])]
    #[case::left_1_lateral('f', 'g', vec![BigramKind::SameFingerSkip { units: 1 }])]
    #[case::left_1_diagonal('f', 'b', vec![BigramKind::SameFingerSkip { units: 1 }])]
    #[case::left_2_diagonal('r', 'b', vec![BigramKind::SameFingerSkip { units: 2 }])]
    #[case::right_1_vertical('u', 'j', vec![BigramKind::SameFingerSkip { units: 1 }])]
    #[case::right_2_vertical('y', 'n', vec![BigramKind::SameFingerSkip { units: 2 }])]
    #[case::right_1_lateral('j', 'h', vec![BigramKind::SameFingerSkip { units: 1 }])]
    #[case::right_1_diagonal('j', 'n', vec![BigramKind::SameFingerSkip { units: 1 }])]
    fn it_calculates_bigram_finger_skip(
        #[case] ch1: char,
        #[case] ch2: char,
        #[case] expected_kinds: Vec<BigramKind>,
        qwerty: Layout,
    ) {
        check!(
            ngram!(qwerty, ch1, ch2)
                .kinds
                .into_iter()
                .collect::<Vec<_>>()
                == expected_kinds
        );
        check!(
            ngram!(qwerty, ch2, ch1)
                .kinds
                .into_iter()
                .collect::<Vec<_>>()
                == expected_kinds
        );
    }

    #[rstest]
    #[case::left_index('d', 'g', vec![BigramKind::LateralStretch { finger: FingerKind::Index, units: 2 }])]
    #[case::left_index('e', 't', vec![BigramKind::LateralStretch { finger: FingerKind::Index, units: 2 }])]
    #[case::left_pinky('"', 's', vec![BigramKind::LateralStretch { finger: FingerKind::Pinky, units: 2 }])]
    #[case::left_non_adiacent_ring('s', 'g', vec![BigramKind::LateralStretch { finger: FingerKind::Ring, units: 3 }])]
    #[case::left_non_adiacent_pink('a', 'g', vec![BigramKind::LateralStretch { finger: FingerKind::Pinky, units: 4 }])]
    #[case::right_index('k', 'h', vec![BigramKind::LateralStretch { finger: FingerKind::Index, units: 2 }])]
    #[case::right_index('i', 'y', vec![BigramKind::LateralStretch { finger: FingerKind::Index, units: 2 }])]
    #[case::right_pinky('l', '\'', vec![BigramKind::LateralStretch { finger: FingerKind::Pinky, units: 2 }])]
    #[case::right_non_adiacent_ring('h', 'l', vec![BigramKind::LateralStretch { finger: FingerKind::Ring, units: 3 }])]
    #[case::right_non_adiacent_pink('h', ';', vec![BigramKind::LateralStretch { finger: FingerKind::Pinky, units: 4 }])]
    fn it_calculates_bigram_lateral_stretch(
        #[case] ch1: char,
        #[case] ch2: char,
        #[case] expected_kinds: Vec<BigramKind>,
        qwerty: Layout,
    ) {
        check!(
            ngram!(qwerty, ch1, ch2)
                .kinds
                .into_iter()
                .collect::<Vec<_>>()
                == expected_kinds
        );
        check!(
            ngram!(qwerty, ch2, ch1)
                .kinds
                .into_iter()
                .collect::<Vec<_>>()
                == expected_kinds
        );
    }

    #[rstest]
    #[case::left_middle_ring_2('c', 'w', vec![BigramKind::Scissor { units: 2, upper_finger: FingerKind::Ring, lower_finger: FingerKind::Middle, adiacent: true }])]
    #[case::left_middle_pinky_2('c', 'q', vec![BigramKind::Scissor { units: 2, upper_finger: FingerKind::Pinky, lower_finger: FingerKind::Middle, adiacent: false }])]
    #[case::left_pinky_index_1('z', 'f', vec![BigramKind::Scissor { units: 1, upper_finger: FingerKind::Index, lower_finger: FingerKind::Pinky, adiacent: false }])]
    #[case::right_ring_index_2('.', 'u', vec![BigramKind::Scissor { units: 2, upper_finger: FingerKind::Index, lower_finger: FingerKind::Ring, adiacent: false }])]
    #[case::right_middle_ring_2(',', 'o', vec![BigramKind::Scissor { units: 2, upper_finger: FingerKind::Ring, lower_finger: FingerKind::Middle, adiacent: true }])]
    fn it_calculates_bigram_scissor(
        #[case] ch1: char,
        #[case] ch2: char,
        #[case] expected_kinds: Vec<BigramKind>,
        qwerty: Layout,
    ) {
        check!(
            ngram!(qwerty, ch1, ch2)
                .kinds
                .into_iter()
                .collect::<Vec<_>>()
                == expected_kinds
        );
        check!(
            ngram!(qwerty, ch2, ch1)
                .kinds
                .into_iter()
                .collect::<Vec<_>>()
                == expected_kinds
        );
    }

    #[rstest]
    #[case('a', 's')]
    #[case('d', 'y')]
    #[case('t', 'n')]
    fn it_calculates_bigram_other(#[case] ch1: char, #[case] ch2: char, qwerty: Layout) {
        check!(
            ngram!(qwerty, ch1, ch2)
                .kinds
                .into_iter()
                .collect::<Vec<_>>()
                == vec![BigramKind::Other]
        );
        check!(
            ngram!(qwerty, ch2, ch1)
                .kinds
                .into_iter()
                .collect::<Vec<_>>()
                == vec![BigramKind::Other]
        );
    }
}

#[cfg(test)]
mod trigram_tests {
    use assert2::check;
    use rstest::rstest;

    use crate::layout::{Layout, fixtures::qwerty};

    use super::*;

    #[rstest]
    #[case::left_1_vertical('q', 'w', 'a', vec![TrigramKind::SameFingerSkip { units: 1, handedness: Handedness::Same }])]
    #[case::left_2_vertical('q', 'w', 'z', vec![TrigramKind::SameFingerSkip { units: 2, handedness: Handedness::Same }])]
    #[case::left_1_vertical('q', 'h', 'a', vec![TrigramKind::SameFingerSkip { units: 1, handedness: Handedness::Alternate }])]
    #[case::left_2_vertical('q', 'h', 'z', vec![TrigramKind::SameFingerSkip { units: 2, handedness: Handedness::Alternate }])]
    #[case::left_1_horizontal('r', 'w', 't', vec![TrigramKind::SameFingerSkip { units: 1, handedness: Handedness::Same }])]
    #[case::left_2_diagonal('r', 'a', 'b', vec![TrigramKind::SameFingerSkip { units: 2, handedness: Handedness::Same }])]
    #[case::left_1_horizontal_cross_hand('r', 'u', 't', vec![TrigramKind::SameFingerSkip { units: 1, handedness: Handedness::Alternate }])]
    #[case::right_1_vertical('u', 'i', 'j', vec![TrigramKind::SameFingerSkip { units: 1, handedness: Handedness::Same }])]
    #[case::right_2_vertical('u', 'i', 'm', vec![TrigramKind::SameFingerSkip { units: 2, handedness: Handedness::Same }])]
    #[case::right_1_vertical('u', 'g', 'j', vec![TrigramKind::SameFingerSkip { units: 1, handedness: Handedness::Alternate }])]
    #[case::right_2_vertical('u', 'g', 'm', vec![TrigramKind::SameFingerSkip { units: 2, handedness: Handedness::Alternate }])]
    fn it_calculates_trigram_finger_skip(
        #[case] ch1: char,
        #[case] ch2: char,
        #[case] ch3: char,
        #[case] expected_kinds: Vec<TrigramKind>,
        qwerty: Layout,
    ) {
        check!(
            ngram!(qwerty, ch1, ch2, ch3)
                .kinds
                .into_iter()
                .collect::<Vec<_>>()
                == expected_kinds
        );
    }

    #[rstest]
    #[case::left_non_adiacent_cross_hand('s', 'y', 'g', vec![TrigramKind::LateralStretch { finger: FingerKind::Ring, units: 3, handedness: Handedness::Alternate }, TrigramKind::Alternation ])]
    #[case::left_index_cross_hand('d', 'u', 'g', vec![ TrigramKind::LateralStretch { finger: FingerKind::Index, units: 2, handedness: Handedness::Alternate }, TrigramKind::Alternation ])]
    #[case::right_index_cross_hand('k', 'e', 'h', vec![ TrigramKind::LateralStretch { finger: FingerKind::Index, units: 2, handedness: Handedness::Alternate }, TrigramKind::Alternation ])]
    #[case::right_pinky_same_hand('l', 'i', '\'', vec![ TrigramKind::LateralStretch { finger: FingerKind::Pinky, units: 2, handedness: Handedness::Same }, TrigramKind::Redirect { strength: RedirectStrength::Weak } ])]
    fn it_calculates_trigram_lateral_stretch(
        #[case] ch1: char,
        #[case] ch2: char,
        #[case] ch3: char,
        #[case] expected_kinds: Vec<TrigramKind>,
        qwerty: Layout,
    ) {
        check!(
            ngram!(qwerty, ch1, ch2, ch3)
                .kinds
                .into_iter()
                .collect::<Vec<_>>()
                == expected_kinds
        );
    }

    #[rstest]
    #[case::left_middle_ring_same_hand('c', 'a', 'w', vec![ TrigramKind::Scissor { units: 2, upper_finger: FingerKind::Ring, lower_finger: FingerKind::Middle, handedness: Handedness::Same, adiacent: true }, TrigramKind::Redirect { strength: RedirectStrength::Weak } ])]
    #[case::left_middle_ring_cross_hand('c', 'j', 'w', vec![ TrigramKind::Scissor { units: 2, upper_finger: FingerKind::Ring, lower_finger: FingerKind::Middle, handedness: Handedness::Alternate, adiacent: true }, TrigramKind::Alternation ])]
    #[case::left_middle_pinky_cross_hand('c', 'j', 'q', vec![ TrigramKind::Scissor { units: 2, upper_finger: FingerKind::Pinky, lower_finger: FingerKind::Middle, handedness: Handedness::Alternate, adiacent: false }, TrigramKind::Alternation ])]
    #[case::right_ring_index_same_hand('.', 'k', 'u', vec![ TrigramKind::Scissor { units: 2, upper_finger: FingerKind::Index, lower_finger: FingerKind::Ring, handedness: Handedness::Same, adiacent: false }, TrigramKind::Roll { length: 3, direction: RollDirection::In } ])]
    #[case::right_middle_ring_cross_hand(',', 'f', 'o', vec![ TrigramKind::Scissor { units: 2, upper_finger: FingerKind::Ring, lower_finger: FingerKind::Middle, handedness: Handedness::Alternate, adiacent: true }, TrigramKind::Alternation ])]
    fn it_calculates_trigram_scissor(
        #[case] ch1: char,
        #[case] ch2: char,
        #[case] ch3: char,
        #[case] expected_kinds: Vec<TrigramKind>,
        qwerty: Layout,
    ) {
        check!(
            ngram!(qwerty, ch1, ch2, ch3)
                .kinds
                .into_iter()
                .collect::<Vec<_>>()
                == expected_kinds
        );
    }

    #[rstest]
    #[case::left_triple('q', 'w', 'e', vec![TrigramKind::Roll { length: 3, direction: RollDirection::In }])]
    #[case::left_triple('q', 'e', 'r', vec![TrigramKind::Roll { length: 3, direction: RollDirection::In }])]
    #[case::left_triple('r', 'e', 'q', vec![TrigramKind::Roll { length: 3, direction: RollDirection::Out }])]
    #[case::left_triple('e', 'w', 'q', vec![TrigramKind::Roll { length: 3, direction: RollDirection::Out }])]
    #[case::right_triple('o', 'i', 'u', vec![TrigramKind::Roll { length: 3, direction: RollDirection::In }])]
    #[case::right_triple('p', 'i', 'u', vec![TrigramKind::Roll { length: 3, direction: RollDirection::In }])]
    #[case::right_triple('u', 'i', 'p', vec![TrigramKind::Roll { length: 3, direction: RollDirection::Out }])]
    #[case::right_triple('i', 'o', 'p', vec![TrigramKind::Roll { length: 3, direction: RollDirection::Out }])]
    #[case::left_triple_mixed_rows('a', 'w', 'd', vec![TrigramKind::Roll { length: 3, direction: RollDirection::In }])]
    #[case::left_double('q', 'w', 'p', vec![TrigramKind::Roll { length: 2, direction: RollDirection::In }])]
    #[case::left_double('q', 'e', 'p', vec![TrigramKind::Roll { length: 2, direction: RollDirection::In }])]
    #[case::left_double('t', 'e', 'p', vec![TrigramKind::Roll { length: 2, direction: RollDirection::Out }])]
    #[case::left_double('e', 'w', 'p', vec![TrigramKind::Roll { length: 2, direction: RollDirection::Out }])]
    #[case::right_double('o', 'i', 'a', vec![TrigramKind::Roll { length: 2, direction: RollDirection::In }])]
    #[case::right_double('p', 'i', 'a', vec![TrigramKind::Roll { length: 2, direction: RollDirection::In }])]
    #[case::right_double('y', 'i', 'a', vec![TrigramKind::Roll { length: 2, direction: RollDirection::Out }])]
    #[case::right_double('i', 'o', 'a', vec![TrigramKind::Roll { length: 2, direction: RollDirection::Out }])]
    fn it_calculates_trigram_roll(
        #[case] ch1: char,
        #[case] ch2: char,
        #[case] ch3: char,
        #[case] expected_kinds: Vec<TrigramKind>,
        qwerty: Layout,
    ) {
        check!(
            ngram!(qwerty, ch1, ch2, ch3)
                .kinds
                .into_iter()
                .collect::<Vec<_>>()
                == expected_kinds
        );
    }

    #[rstest]
    #[case::left_strong('q', 'e', 'w', vec![TrigramKind::Redirect { strength: RedirectStrength::Weak }])]
    #[case::left_weak('q', 't', 'e', vec![TrigramKind::Redirect { strength: RedirectStrength::Strong }])]
    fn it_calculates_trigram_redirect(
        #[case] ch1: char,
        #[case] ch2: char,
        #[case] ch3: char,
        #[case] expected_kinds: Vec<TrigramKind>,
        qwerty: Layout,
    ) {
        check!(
            ngram!(qwerty, ch1, ch2, ch3)
                .kinds
                .into_iter()
                .collect::<Vec<_>>()
                == expected_kinds
        );
    }

    #[rstest]
    #[case::simple_alternation('q', 'h', 'w', vec![TrigramKind::Alternation])]
    #[case::right_alternation('u', 'a', 'i', vec![TrigramKind::Alternation])]
    fn it_calculates_trigram_alternation(
        #[case] ch1: char,
        #[case] ch2: char,
        #[case] ch3: char,
        #[case] expected_kinds: Vec<TrigramKind>,
        qwerty: Layout,
    ) {
        check!(
            ngram!(qwerty, ch1, ch2, ch3)
                .kinds
                .into_iter()
                .collect::<Vec<_>>()
                == expected_kinds
        );
    }

    #[rstest]
    #[case('q', 'q', 'a')]
    #[case('t', 'r', 'e')]
    #[case('q', 't', 'q')]
    fn it_calculates_trigram_other(
        #[case] ch1: char,
        #[case] ch2: char,
        #[case] ch3: char,
        qwerty: Layout,
    ) {
        check!(
            ngram!(qwerty, ch1, ch2, ch3)
                .kinds
                .into_iter()
                .collect::<Vec<_>>()
                == vec![TrigramKind::Other]
        );
    }
}
