use crate::layout::{FingerKind, Key};

#[derive(PartialEq, Debug)]
pub struct Trigram {
    pub kind: TrigramKind,
    key1: Key,
    key2: Key,
    key3: Key,
}

#[derive(PartialEq, Debug)]
pub enum TrigramKind {
    SameFingerSkip { skips: u8, same_hand: bool },
    RollIn { triple: bool },
    RollOut { triple: bool },
    Roll { triple: bool, inward: bool },
    Redirect { weak: bool },
    Alternation,
    Other,
}

impl Trigram {
    pub fn new(key1: &Key, key2: &Key, key3: &Key) -> Self {
        Self {
            kind: Self::find_kind(key1, key2, key3),
            key1: *key1,
            key2: *key2,
            key3: *key3,
        }
    }

    fn find_kind(key1: &Key, key2: &Key, key3: &Key) -> TrigramKind {
        if key1.finger == key3.finger && key1.finger != key2.finger {
            let row_distance = key1.row_distance(key3);
            let col_distance = key1.column_distance(key3);

            if row_distance > 0 || col_distance > 0 {
                return TrigramKind::SameFingerSkip {
                    skips: row_distance as u8 + col_distance as u8,
                    same_hand: key2.finger.hand == key1.finger.hand,
                };
            }
        }

        if key1.finger.hand == key2.finger.hand && key2.finger.hand == key3.finger.hand {
            if key1.finger < key2.finger && key2.finger < key3.finger {
                return TrigramKind::Roll {
                    triple: true,
                    inward: true,
                };
            }
            if key1.finger > key2.finger && key2.finger > key3.finger {
                return TrigramKind::Roll {
                    triple: true,
                    inward: false,
                };
            }

            if (key1.finger < key2.finger && key3.finger < key2.finger
                || key1.finger > key2.finger && key3.finger > key2.finger)
                && key1.finger.hand == key3.finger.hand
                && !key1.same_finger(key3)
            {
                if key1.finger.kind == FingerKind::Index
                    || key2.finger.kind == FingerKind::Index
                    || key3.finger.kind == FingerKind::Index
                {
                    return TrigramKind::Redirect { weak: true };
                } else {
                    return TrigramKind::Redirect { weak: false };
                }
            }
        }

        if key1.finger.hand == key2.finger.hand && key2.finger.hand != key3.finger.hand {
            if key1.finger < key2.finger {
                return TrigramKind::Roll {
                    triple: false,
                    inward: true,
                };
            }
            if key1.finger > key2.finger {
                return TrigramKind::Roll {
                    triple: false,
                    inward: false,
                };
            }
        }

        if key1.finger.hand == key3.finger.hand
            && key2.finger.hand != key1.finger.hand
            && !key1.same_finger(key3)
        {
            return TrigramKind::Alternation;
        }

        TrigramKind::Other
    }
}

#[derive(PartialEq, Debug)]
pub struct Bigram {
    pub kind: BigramKind,
    key1: Key,
    key2: Key,
}

#[derive(PartialEq, Debug)]
pub enum BigramKind {
    SameFingerSkip { skips: u8 },
    LateralStretch { distance: u8 },
    Scissor { col_distance: u8, row_distance: u8 },
    Other,
}

impl Bigram {
    pub fn new(key1: &Key, key2: &Key) -> Self {
        Self {
            kind: Self::find_kind(key1, key2),
            key1: *key1,
            key2: *key2,
        }
    }

    fn find_kind(key1: &Key, key2: &Key) -> BigramKind {
        let row_distance = key1.row_distance(key2);
        let col_distance = key1.column_distance(key2);
        let finger_distance = key1.finger.distance(&key2.finger);

        if key1.same_finger(key2) && (row_distance > 0 || col_distance > 0) {
            return BigramKind::SameFingerSkip {
                skips: row_distance as u8 + col_distance as u8,
            };
        }

        if let Some(finger_distance) = finger_distance
            && finger_distance == 1
            && col_distance > 1
            && row_distance == 0
        {
            if key1.finger.kind > FingerKind::Middle || key2.finger.kind > FingerKind::Middle {
                let highest_finger = key1.finger.max(key2.finger);
                return BigramKind::LateralStretch {
                    distance: highest_finger.into(),
                };
            }

            if key1.finger.kind < FingerKind::Middle || key2.finger.kind < FingerKind::Middle {
                let lowest_finger = key1.finger.min(key2.finger);
                return BigramKind::LateralStretch {
                    distance: lowest_finger.into(),
                };
            }
        }

        if row_distance >= 1
            && col_distance >= 1
            && !key1.same_finger(key2)
            && key1.finger.hand == key2.finger.hand
        {
            return BigramKind::Scissor {
                col_distance: col_distance as u8,
                row_distance: row_distance as u8,
            };
        }

        BigramKind::Other
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

        check!(
            Bigram::new(&key1, &key2)
                == Bigram {
                    kind: BigramKind::Other,
                    key1,
                    key2,
                }
        );
    }

    #[rstest]
    #[case::left_1_vertical('q', 'a', BigramKind::SameFingerSkip { skips: 1 })]
    #[case::left_2_vertical('q', 'z', BigramKind::SameFingerSkip { skips: 2 })]
    #[case::left_1_lateral('f', 'g', BigramKind::SameFingerSkip { skips: 1 })]
    #[case::left_1_diagonal('f', 'b', BigramKind::SameFingerSkip { skips: 2 })]
    #[case::right_1_vertical('u', 'j', BigramKind::SameFingerSkip { skips: 1 })]
    #[case::right_2_vertical('y', 'n', BigramKind::SameFingerSkip { skips: 2 })]
    #[case::right_1_lateral('j', 'h', BigramKind::SameFingerSkip { skips: 1 })]
    #[case::right_1_diagonal('j', 'n', BigramKind::SameFingerSkip { skips: 2 })]
    fn it_calculates_bigram_finger_skip(
        #[case] ch1: char,
        #[case] ch2: char,
        #[case] expected_kind: BigramKind,
        qwerty: Layout,
    ) {
        let key1 = qwerty.key_for(ch1).unwrap();
        let key2 = qwerty.key_for(ch2).unwrap();

        check!(Bigram::new(key1, key2).kind == expected_kind);
        check!(Bigram::new(key2, key1).kind == expected_kind);
    }

    #[rstest]
    #[case::left_index('d', 'g', BigramKind::LateralStretch { distance: 4 })]
    #[case::left_index('e', 't', BigramKind::LateralStretch { distance: 4 })]
    #[case::left_pinky('"', 's', BigramKind::LateralStretch { distance: 1 })]
    #[case::right_index('k', 'h', BigramKind::LateralStretch { distance: 7 })]
    #[case::right_index('i', 'y', BigramKind::LateralStretch { distance: 7 })]
    #[case::right_pinky('l', '\'', BigramKind::LateralStretch { distance: 10 })]
    fn it_calculates_bigram_lateral_stretch(
        #[case] ch1: char,
        #[case] ch2: char,
        #[case] expected_kind: BigramKind,
        qwerty: Layout,
    ) {
        let key1 = qwerty.key_for(ch1).unwrap();
        let key2 = qwerty.key_for(ch2).unwrap();

        check!(Bigram::new(key1, key2).kind == expected_kind);
        check!(Bigram::new(key2, key1).kind == expected_kind);
    }

    #[rstest]
    #[case::left_small('z', 's', BigramKind::Scissor { col_distance: 1, row_distance: 1 })]
    #[case::left_wide('z', 'w', BigramKind::Scissor { col_distance: 1, row_distance: 2 })]
    #[case::left_very_wide('z', 'e', BigramKind::Scissor { col_distance: 2, row_distance: 2 })]
    #[case::left_wide_reverse('v', 'e', BigramKind::Scissor { col_distance: 1, row_distance: 2 })]
    #[case::left_flat_wide('z', 'f', BigramKind::Scissor { col_distance: 3, row_distance: 1 })]
    #[case::right_small('l', 'i', BigramKind::Scissor { col_distance: 1, row_distance: 1 })]
    #[case::right_wide('.', 'i', BigramKind::Scissor { col_distance: 1, row_distance: 2 })]
    #[case::right_very_wide('.', 'u', BigramKind::Scissor { col_distance: 2, row_distance: 2 })]
    #[case::right_wide_reverse('m', 'i', BigramKind::Scissor { col_distance: 1, row_distance: 2 })]
    fn it_calculates_bigram_scissor(
        #[case] ch1: char,
        #[case] ch2: char,
        #[case] expected_kind: BigramKind,
        qwerty: Layout,
    ) {
        let key1 = qwerty.key_for(ch1).unwrap();
        let key2 = qwerty.key_for(ch2).unwrap();

        check!(Bigram::new(key1, key2).kind == expected_kind);
        check!(Bigram::new(key2, key1).kind == expected_kind);
    }

    #[rstest]
    #[case('a', 's')]
    #[case('d', 'y')]
    #[case('t', 'n')]
    fn it_calculates_bigram_other(#[case] ch1: char, #[case] ch2: char, qwerty: Layout) {
        let key1 = qwerty.key_for(ch1).unwrap();
        let key2 = qwerty.key_for(ch2).unwrap();

        check!(Bigram::new(key1, key2).kind == BigramKind::Other);
        check!(Bigram::new(key2, key1).kind == BigramKind::Other);
    }
}

#[cfg(test)]
mod trigram_tests {
    use assert2::check;
    use rstest::rstest;

    use crate::layout::{Layout, fixtures::qwerty};

    use super::*;

    #[rstest]
    #[case::left_1_vertical('q', 'w', 'a', TrigramKind::SameFingerSkip { skips: 1, same_hand: true })]
    #[case::left_2_vertical('q', 'w', 'z', TrigramKind::SameFingerSkip { skips: 2, same_hand: true })]
    #[case::left_1_vertical('q', 'h', 'a', TrigramKind::SameFingerSkip { skips: 1, same_hand: false })]
    #[case::left_2_vertical('q', 'h', 'z', TrigramKind::SameFingerSkip { skips: 2, same_hand: false })]
    #[case::left_1_horizontal('r', 'w', 't', TrigramKind::SameFingerSkip { skips: 1, same_hand: true })]
    #[case::left_1_horizontal_cross_hand('r', 'u', 't', TrigramKind::SameFingerSkip { skips: 1, same_hand: false })]
    #[case::right_1_vertical('u', 'i', 'j', TrigramKind::SameFingerSkip { skips: 1, same_hand: true })]
    #[case::right_2_vertical('u', 'i', 'm', TrigramKind::SameFingerSkip { skips: 2, same_hand: true })]
    #[case::right_1_vertical('u', 'g', 'j', TrigramKind::SameFingerSkip { skips: 1, same_hand: false })]
    #[case::right_2_vertical('u', 'g', 'm', TrigramKind::SameFingerSkip { skips: 2, same_hand: false })]
    fn it_calculates_trigram_finger_skip(
        #[case] ch1: char,
        #[case] ch2: char,
        #[case] ch3: char,
        #[case] expected_kind: TrigramKind,
        qwerty: Layout,
    ) {
        let key1 = qwerty.key_for(ch1).unwrap();
        let key2 = qwerty.key_for(ch2).unwrap();
        let key3 = qwerty.key_for(ch3).unwrap();

        check!(Trigram::new(key1, key2, key3).kind == expected_kind);
    }

    #[rstest]
    #[case::left_triple('q', 'w', 'e', TrigramKind::Roll { triple: true, inward: true })]
    #[case::left_triple('q', 'e', 't', TrigramKind::Roll { triple: true, inward: true })]
    #[case::left_triple('t', 'e', 'q', TrigramKind::Roll { triple: true, inward: false })]
    #[case::left_triple('e', 'w', 'q', TrigramKind::Roll { triple: true, inward: false })]
    #[case::right_triple('o', 'i', 'u', TrigramKind::Roll { triple: true, inward: true })]
    #[case::right_triple('p', 'i', 'y', TrigramKind::Roll { triple: true, inward: true })]
    #[case::right_triple('y', 'i', 'p', TrigramKind::Roll { triple: true, inward: false })]
    #[case::right_triple('i', 'o', 'p', TrigramKind::Roll { triple: true, inward: false })]
    #[case::left_triple_mixed_rows('a', 'w', 'd', TrigramKind::Roll { triple: true, inward: true })]
    #[case::left_double('q', 'w', 'p', TrigramKind::Roll { triple: false, inward: true })]
    #[case::left_double('q', 'e', 'p', TrigramKind::Roll { triple: false, inward: true })]
    #[case::left_double('t', 'e', 'p', TrigramKind::Roll { triple: false, inward: false })]
    #[case::left_double('e', 'w', 'p', TrigramKind::Roll { triple: false, inward: false })]
    #[case::right_double('o', 'i', 'a', TrigramKind::Roll { triple: false, inward: true })]
    #[case::right_double('p', 'i', 'a', TrigramKind::Roll { triple: false, inward: true })]
    #[case::right_double('y', 'i', 'a', TrigramKind::Roll { triple: false, inward: false })]
    #[case::right_double('i', 'o', 'a', TrigramKind::Roll { triple: false, inward: false })]
    fn it_calculates_trigram_roll(
        #[case] ch1: char,
        #[case] ch2: char,
        #[case] ch3: char,
        #[case] expected_kind: TrigramKind,
        qwerty: Layout,
    ) {
        let key1 = qwerty.key_for(ch1).unwrap();
        let key2 = qwerty.key_for(ch2).unwrap();
        let key3 = qwerty.key_for(ch3).unwrap();

        check!(Trigram::new(key1, key2, key3).kind == expected_kind);
    }

    #[rstest]
    #[case::left_strong('q', 'e', 'w',TrigramKind::Redirect { weak: false })]
    #[case::left_weak('q', 't', 'e', TrigramKind::Redirect { weak: true })]
    fn it_calculates_trigram_redirect(
        #[case] ch1: char,
        #[case] ch2: char,
        #[case] ch3: char,
        #[case] expected_kind: TrigramKind,
        qwerty: Layout,
    ) {
        let key1 = qwerty.key_for(ch1).unwrap();
        let key2 = qwerty.key_for(ch2).unwrap();
        let key3 = qwerty.key_for(ch3).unwrap();

        check!(Trigram::new(key1, key2, key3).kind == expected_kind);
    }

    #[rstest]
    #[case::simple_alternation('q', 'h', 'w', TrigramKind::Alternation)]
    #[case::right_alternation('u', 'a', 'i', TrigramKind::Alternation)]
    fn it_calculates_trigram_alternation(
        #[case] ch1: char,
        #[case] ch2: char,
        #[case] ch3: char,
        #[case] expected_kind: TrigramKind,
        qwerty: Layout,
    ) {
        let key1 = qwerty.key_for(ch1).unwrap();
        let key2 = qwerty.key_for(ch2).unwrap();
        let key3 = qwerty.key_for(ch3).unwrap();

        check!(Trigram::new(key1, key2, key3).kind == expected_kind);
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
        let key1 = qwerty.key_for(ch1).unwrap();
        let key2 = qwerty.key_for(ch2).unwrap();
        let key3 = qwerty.key_for(ch3).unwrap();

        check!(Trigram::new(key1, key2, key3).kind == TrigramKind::Other);
    }
}
