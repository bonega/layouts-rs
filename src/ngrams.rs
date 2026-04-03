use crate::layout::{FingerKind, Key};

#[derive(PartialEq, Debug)]
struct Trigram {
    kinds: Vec<TrigramKind>,
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
        let mut kinds = Vec::new();

        if key1.finger == key3.finger && key1.finger != key2.finger {
            let row_distance = key1.row_distance(key3);
            let col_distance = key1.column_distance(key3);

            if row_distance > 0 || col_distance > 0 {
                kinds.push(TrigramKind::SameFingerSkip {
                    skips: row_distance as u8 + col_distance as u8,
                    same_hand: key2.finger.hand == key1.finger.hand,
                });
            }
        }

        if key1.finger.hand == key2.finger.hand && key2.finger.hand == key3.finger.hand {
            if key1.finger < key2.finger && key2.finger < key3.finger {
                kinds.push(TrigramKind::Roll {
                    triple: true,
                    inward: true,
                });
            }
            if key1.finger > key2.finger && key2.finger > key3.finger {
                kinds.push(TrigramKind::Roll {
                    triple: true,
                    inward: false,
                });
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
                    kinds.push(TrigramKind::Redirect { weak: true });
                } else {
                    kinds.push(TrigramKind::Redirect { weak: false });
                }
            }
        }

        if key1.finger.hand == key2.finger.hand && key2.finger.hand != key3.finger.hand {
            if key1.finger < key2.finger {
                kinds.push(TrigramKind::Roll {
                    triple: false,
                    inward: true,
                });
            }
            if key1.finger > key2.finger {
                kinds.push(TrigramKind::Roll {
                    triple: false,
                    inward: false,
                });
            }
        }

        if key1.finger.hand == key3.finger.hand
            && key2.finger.hand != key1.finger.hand
            && !key1.same_finger(&key3)
        {
            kinds.push(TrigramKind::Alternation);
        }

        if kinds.is_empty() {
            kinds.push(TrigramKind::Other);
        }

        Self {
            kinds,
            key1: key1.clone(),
            key2: key2.clone(),
            key3: key3.clone(),
        }
    }
}

#[derive(PartialEq, Debug)]
struct Bigram {
    kinds: Vec<BigramKind>,
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
        let mut kinds = Vec::new();

        let row_distance = key1.row_distance(key2);
        let col_distance = key1.column_distance(key2);
        let finger_distance = key1.finger.distance(&key2.finger);

        if key1.same_finger(key2) {
            if row_distance > 0 || col_distance > 0 {
                kinds.push(BigramKind::SameFingerSkip {
                    skips: row_distance as u8 + col_distance as u8,
                });
            }
        }

        if let Some(finger_distance) = finger_distance
            && finger_distance == 1
            && col_distance > 1
            && row_distance == 0
        {
            if key1.finger.kind > FingerKind::Middle || key2.finger.kind > FingerKind::Middle {
                let highest_finger = key1.finger.max(key2.finger);
                kinds.push(BigramKind::LateralStretch {
                    distance: highest_finger.into(),
                });
            }

            if key1.finger.kind < FingerKind::Middle || key2.finger.kind < FingerKind::Middle {
                let lowest_finger = key1.finger.min(key2.finger);
                kinds.push(BigramKind::LateralStretch {
                    distance: lowest_finger.into(),
                });
            }
        }

        if row_distance >= 1
            && col_distance >= 1
            && !key1.same_finger(key2)
            && key1.finger.hand == key2.finger.hand
        {
            kinds.push(BigramKind::Scissor {
                col_distance: col_distance as u8,
                row_distance: row_distance as u8,
            });
        }

        if kinds.is_empty() {
            kinds.push(BigramKind::Other);
        }

        Self {
            kinds,
            key1: key1.clone(),
            key2: key2.clone(),
        }
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
        let key1 = qwerty.key_for('a').unwrap().clone();
        let key2 = qwerty.key_for('s').unwrap().clone();

        check!(
            Bigram::new(&key1, &key2)
                == Bigram {
                    kinds: vec![BigramKind::Other],
                    key1,
                    key2,
                }
        );
    }

    #[rstest]
    #[case::left_1_vertical('q', 'a', vec![BigramKind::SameFingerSkip { skips: 1 }])]
    #[case::left_2_vertical('q', 'z', vec![BigramKind::SameFingerSkip { skips: 2 }])]
    #[case::left_1_lateral('f', 'g', vec![BigramKind::SameFingerSkip { skips: 1 }])]
    #[case::left_1_diagonal('f', 'b', vec![BigramKind::SameFingerSkip { skips: 2 }])]
    #[case::right_1_vertical('u', 'j', vec![BigramKind::SameFingerSkip { skips: 1 }])]
    #[case::right_2_vertical('y', 'n', vec![BigramKind::SameFingerSkip { skips: 2 }])]
    #[case::right_1_lateral('j', 'h', vec![BigramKind::SameFingerSkip { skips: 1 }])]
    #[case::right_1_diagonal('j', 'n', vec![BigramKind::SameFingerSkip { skips: 2 }])]
    fn it_calculates_bigram_finger_skip(
        #[case] ch1: char,
        #[case] ch2: char,
        #[case] expected_kinds: Vec<BigramKind>,
        qwerty: Layout,
    ) {
        let key1 = qwerty.key_for(ch1).unwrap();
        let key2 = qwerty.key_for(ch2).unwrap();

        check!(Bigram::new(&key1, &key2).kinds == expected_kinds);
        check!(Bigram::new(&key2, &key1).kinds == expected_kinds);
    }

    #[rstest]
    #[case::left_index('d', 'g', vec![BigramKind::LateralStretch { distance: 4 }])]
    #[case::left_index('e', 't', vec![BigramKind::LateralStretch { distance: 4 }])]
    #[case::left_pinky('"', 's', vec![BigramKind::LateralStretch { distance: 1 }])]
    #[case::right_index('k', 'h', vec![BigramKind::LateralStretch { distance: 7 }])]
    #[case::right_index('i', 'y', vec![BigramKind::LateralStretch { distance: 7 }])]
    #[case::right_pinky('l', '\'', vec![BigramKind::LateralStretch { distance: 10 }])]
    fn it_calculates_bigram_lateral_stretch(
        #[case] ch1: char,
        #[case] ch2: char,
        #[case] expected_kinds: Vec<BigramKind>,
        qwerty: Layout,
    ) {
        let key1 = qwerty.key_for(ch1).unwrap();
        let key2 = qwerty.key_for(ch2).unwrap();

        check!(Bigram::new(&key1, &key2).kinds == expected_kinds);
        check!(Bigram::new(&key2, &key1).kinds == expected_kinds);
    }

    #[rstest]
    #[case::left_small('z', 's', vec![BigramKind::Scissor { col_distance: 1, row_distance: 1 }])]
    #[case::left_wide('z', 'w', vec![BigramKind::Scissor { col_distance: 1, row_distance: 2 }])]
    #[case::left_very_wide('z', 'e', vec![BigramKind::Scissor { col_distance: 2, row_distance: 2 }])]
    #[case::left_wide_reverse('v', 'e', vec![BigramKind::Scissor { col_distance: 1, row_distance: 2 }])]
    #[case::left_flat_wide('z', 'f', vec![BigramKind::Scissor { col_distance: 3, row_distance: 1 }])]
    #[case::right_small('l', 'i', vec![BigramKind::Scissor { col_distance: 1, row_distance: 1 }])]
    #[case::right_wide('.', 'i', vec![BigramKind::Scissor { col_distance: 1, row_distance: 2 }])]
    #[case::right_very_wide('.', 'u', vec![BigramKind::Scissor { col_distance: 2, row_distance: 2 }])]
    #[case::right_wide_reverse('m', 'i', vec![BigramKind::Scissor { col_distance: 1, row_distance: 2 }])]
    fn it_calculates_bigram_scissor(
        #[case] ch1: char,
        #[case] ch2: char,
        #[case] expected_kinds: Vec<BigramKind>,
        qwerty: Layout,
    ) {
        let key1 = qwerty.key_for(ch1).unwrap();
        let key2 = qwerty.key_for(ch2).unwrap();

        check!(Bigram::new(&key1, &key2).kinds == expected_kinds);
        check!(Bigram::new(&key2, &key1).kinds == expected_kinds);
    }

    #[rstest]
    #[case('a', 's')]
    #[case('d', 'y')]
    #[case('t', 'n')]
    fn it_calculates_bigram_other(#[case] ch1: char, #[case] ch2: char, qwerty: Layout) {
        let key1 = qwerty.key_for(ch1).unwrap();
        let key2 = qwerty.key_for(ch2).unwrap();

        check!(Bigram::new(&key1, &key2).kinds == vec![BigramKind::Other]);
        check!(Bigram::new(&key2, &key1).kinds == vec![BigramKind::Other]);
    }
}

#[cfg(test)]
mod trigram_tests {
    use assert2::check;
    use rstest::rstest;

    use crate::layout::{Layout, fixtures::qwerty};

    use super::*;

    #[rstest]
    #[case::left_1_vertical('q', 'w', 'a', vec![TrigramKind::SameFingerSkip { skips: 1, same_hand: true }])]
    #[case::left_2_vertical('q', 'w', 'z', vec![TrigramKind::SameFingerSkip { skips: 2, same_hand: true }])]
    #[case::left_1_vertical('q', 'h', 'a', vec![TrigramKind::SameFingerSkip { skips: 1, same_hand: false }])]
    #[case::left_2_vertical('q', 'h', 'z', vec![TrigramKind::SameFingerSkip { skips: 2, same_hand: false }])]
    #[case::left_1_horizontal('r', 'w', 't', vec![TrigramKind::SameFingerSkip { skips: 1, same_hand: true }])]
    #[case::left_1_horizontal_cross_hand('r', 'u', 't', vec![TrigramKind::SameFingerSkip { skips: 1, same_hand: false }])]
    #[case::right_1_vertical('u', 'i', 'j', vec![TrigramKind::SameFingerSkip { skips: 1, same_hand: true }])]
    #[case::right_2_vertical('u', 'i', 'm', vec![TrigramKind::SameFingerSkip { skips: 2, same_hand: true }])]
    #[case::right_1_vertical('u', 'g', 'j', vec![TrigramKind::SameFingerSkip { skips: 1, same_hand: false }])]
    #[case::right_2_vertical('u', 'g', 'm', vec![TrigramKind::SameFingerSkip { skips: 2, same_hand: false }])]
    fn it_calculates_trigram_finger_skip(
        #[case] ch1: char,
        #[case] ch2: char,
        #[case] ch3: char,
        #[case] expected_kinds: Vec<TrigramKind>,
        qwerty: Layout,
    ) {
        let key1 = qwerty.key_for(ch1).unwrap();
        let key2 = qwerty.key_for(ch2).unwrap();
        let key3 = qwerty.key_for(ch3).unwrap();

        check!(Trigram::new(&key1, &key2, &key3).kinds == expected_kinds);
    }

    #[rstest]
    #[case::left_triple('q', 'w', 'e', vec![TrigramKind::Roll { triple: true , inward: true }])]
    #[case::left_triple('q', 'e', 't', vec![TrigramKind::Roll { triple: true , inward: true }])]
    #[case::left_triple('t', 'e', 'q', vec![TrigramKind::Roll { triple: true, inward: false }])]
    #[case::left_triple('e', 'w', 'q', vec![TrigramKind::Roll { triple: true, inward: false }])]
    #[case::right_triple('o', 'i', 'u', vec![TrigramKind::Roll { triple: true , inward: true }])]
    #[case::right_triple('p', 'i', 'y', vec![TrigramKind::Roll { triple: true , inward: true }])]
    #[case::right_triple('y', 'i', 'p', vec![TrigramKind::Roll { triple: true, inward: false }])]
    #[case::right_triple('i', 'o', 'p', vec![TrigramKind::Roll { triple: true, inward: false }])]
    #[case::left_triple_mixed_rows('a', 'w', 'd', vec![TrigramKind::Roll { triple: true , inward: true }])]
    #[case::left_double('q', 'w', 'p', vec![TrigramKind::Roll { triple: false , inward: true }])]
    #[case::left_double('q', 'e', 'p', vec![TrigramKind::Roll { triple: false , inward: true }])]
    #[case::left_double('t', 'e', 'p', vec![TrigramKind::Roll { triple: false, inward: false }])]
    #[case::left_double('e', 'w', 'p', vec![TrigramKind::Roll { triple: false, inward: false }])]
    #[case::right_double('o', 'i', 'a', vec![TrigramKind::Roll { triple: false , inward: true }])]
    #[case::right_double('p', 'i', 'a', vec![TrigramKind::Roll { triple: false , inward: true }])]
    #[case::right_double('y', 'i', 'a', vec![TrigramKind::Roll { triple: false, inward: false }])]
    #[case::right_double('i', 'o', 'a', vec![TrigramKind::Roll { triple: false, inward: false }])]
    fn it_calculates_trigram_roll(
        #[case] ch1: char,
        #[case] ch2: char,
        #[case] ch3: char,
        #[case] expected_kinds: Vec<TrigramKind>,
        qwerty: Layout,
    ) {
        let key1 = qwerty.key_for(ch1).unwrap();
        let key2 = qwerty.key_for(ch2).unwrap();
        let key3 = qwerty.key_for(ch3).unwrap();

        check!(Trigram::new(&key1, &key2, &key3).kinds == expected_kinds);
    }

    #[rstest]
    #[case::left_strong('q', 'e', 'w',vec![TrigramKind::Redirect { weak: false }])]
    #[case::left_weak('q', 't', 'e', vec![TrigramKind::Redirect { weak: true }])]
    fn it_calculates_trigram_redirect(
        #[case] ch1: char,
        #[case] ch2: char,
        #[case] ch3: char,
        #[case] expected_kinds: Vec<TrigramKind>,
        qwerty: Layout,
    ) {
        let key1 = qwerty.key_for(ch1).unwrap();
        let key2 = qwerty.key_for(ch2).unwrap();
        let key3 = qwerty.key_for(ch3).unwrap();

        check!(Trigram::new(&key1, &key2, &key3).kinds == expected_kinds);
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
        let key1 = qwerty.key_for(ch1).unwrap();
        let key2 = qwerty.key_for(ch2).unwrap();
        let key3 = qwerty.key_for(ch3).unwrap();

        check!(Trigram::new(&key1, &key2, &key3).kinds == expected_kinds);
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

        check!(Trigram::new(&key1, &key2, &key3).kinds == vec![TrigramKind::Other]);
    }
}
