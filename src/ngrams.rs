use crate::layout::{FingerKind, Key};

#[derive(PartialEq, Debug)]
struct Bigram {
    kinds: Vec<BigramKind>,
    key1: Key,
    key2: Key,
}

#[derive(PartialEq, Debug)]
pub enum BigramKind {
    SameFingerSkip(u8),
    LateralStretch(u8),
    Scissor(u8, u8),
    Other,
}

struct Ngrams;

impl Ngrams {
    pub fn classify_bigram(key1: &Key, key2: &Key) -> Bigram {
        let mut kinds = Vec::new();

        let row_distance = key1.row_distance(key2);
        let col_distance = key1.column_distance(key2);
        let finger_distance = key1.finger.distance(&key2.finger);

        if key1.same_finger(key2) {
            if row_distance > 0 || col_distance > 0 {
                kinds.push(BigramKind::SameFingerSkip(
                    row_distance as u8 + col_distance as u8,
                ));
            }
        }

        if let Some(finger_distance) = finger_distance
            && finger_distance == 1
            && col_distance > 1
            && row_distance == 0
        {
            if key1.finger.kind > FingerKind::Middle || key2.finger.kind > FingerKind::Middle {
                let highest_finger = key1.finger.max(key2.finger);
                kinds.push(BigramKind::LateralStretch(highest_finger.into()));
            }

            if key1.finger.kind < FingerKind::Middle || key2.finger.kind < FingerKind::Middle {
                let lowest_finger = key1.finger.min(key2.finger);
                kinds.push(BigramKind::LateralStretch(lowest_finger.into()));
            }
        }

        if row_distance >= 1
            && col_distance >= 1
            && !key1.same_finger(key2)
            && key1.finger.hand == key2.finger.hand
        {
            kinds.push(BigramKind::Scissor(row_distance as u8, col_distance as u8));
        }

        if kinds.is_empty() {
            kinds.push(BigramKind::Other);
        }

        Bigram {
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
            Ngrams::classify_bigram(&key1, &key2)
                == Bigram {
                    kinds: vec![BigramKind::Other],
                    key1,
                    key2,
                }
        );
    }

    #[rstest]
    #[case::left_1_vertical('q', 'a', vec![BigramKind::SameFingerSkip(1)])]
    #[case::left_2_vertical('q', 'z', vec![BigramKind::SameFingerSkip(2)])]
    #[case::left_1_lateral('f', 'g', vec![BigramKind::SameFingerSkip(1)])]
    #[case::left_1_diagonal('f', 'b', vec![BigramKind::SameFingerSkip(2)])]
    #[case::right_1_vertical('u', 'j', vec![BigramKind::SameFingerSkip(1)])]
    #[case::right_2_vertical('y', 'n', vec![BigramKind::SameFingerSkip(2)])]
    #[case::right_1_lateral('j', 'h', vec![BigramKind::SameFingerSkip(1)])]
    #[case::right_1_diagonal('j', 'n', vec![BigramKind::SameFingerSkip(2)])]
    fn it_calculates_bigram_finger_skip(
        #[case] ch1: char,
        #[case] ch2: char,
        #[case] expected_kinds: Vec<BigramKind>,
        qwerty: Layout,
    ) {
        let key1 = qwerty.key_for(ch1).unwrap();
        let key2 = qwerty.key_for(ch2).unwrap();

        check!(Ngrams::classify_bigram(&key1, &key2).kinds == expected_kinds);
        check!(Ngrams::classify_bigram(&key2, &key1).kinds == expected_kinds);
    }

    #[rstest]
    #[case::left_index('d', 'g', vec![BigramKind::LateralStretch(4)])]
    #[case::left_index('e', 't', vec![BigramKind::LateralStretch(4)])]
    #[case::left_pinky('"', 's', vec![BigramKind::LateralStretch(1)])]
    #[case::right_index('k', 'h', vec![BigramKind::LateralStretch(7)])]
    #[case::right_index('i', 'y', vec![BigramKind::LateralStretch(7)])]
    #[case::right_pinky('l', '\'', vec![BigramKind::LateralStretch(10)])]
    fn it_calculates_bigram_lateral_stretch(
        #[case] ch1: char,
        #[case] ch2: char,
        #[case] expected_kinds: Vec<BigramKind>,
        qwerty: Layout,
    ) {
        let key1 = qwerty.key_for(ch1).unwrap();
        let key2 = qwerty.key_for(ch2).unwrap();

        check!(Ngrams::classify_bigram(&key1, &key2).kinds == expected_kinds);
        check!(Ngrams::classify_bigram(&key2, &key1).kinds == expected_kinds);
    }

    #[rstest]
    #[case::left_small('z', 's', vec![BigramKind::Scissor(1, 1)])]
    #[case::left_wide('z', 'w', vec![BigramKind::Scissor(2, 1)])]
    #[case::left_very_wide('z', 'e', vec![BigramKind::Scissor(2, 2)])]
    #[case::left_wide_reverse('v', 'e', vec![BigramKind::Scissor(2, 1)])]
    #[case::left_flat_wide('z', 'f', vec![BigramKind::Scissor(1, 3)])]
    #[case::right_small('l', 'i', vec![BigramKind::Scissor(1, 1)])]
    #[case::right_wide('.', 'i', vec![BigramKind::Scissor(2, 1)])]
    #[case::right_very_wide('.', 'u', vec![BigramKind::Scissor(2, 2)])]
    #[case::right_wide_reverse('m', 'i', vec![BigramKind::Scissor(2, 1)])]
    fn it_calculates_bigram_scissor(
        #[case] ch1: char,
        #[case] ch2: char,
        #[case] expected_kinds: Vec<BigramKind>,
        qwerty: Layout,
    ) {
        let key1 = qwerty.key_for(ch1).unwrap();
        let key2 = qwerty.key_for(ch2).unwrap();

        check!(Ngrams::classify_bigram(&key1, &key2).kinds == expected_kinds);
        check!(Ngrams::classify_bigram(&key2, &key1).kinds == expected_kinds);
    }

    #[rstest]
    #[case('a', 's')]
    #[case('d', 'y')]
    #[case('t', 'n')]
    fn it_calculates_bigram_other(#[case] ch1: char, #[case] ch2: char, qwerty: Layout) {
        let key1 = qwerty.key_for(ch1).unwrap();
        let key2 = qwerty.key_for(ch2).unwrap();

        check!(Ngrams::classify_bigram(&key1, &key2).kinds == vec![BigramKind::Other]);
        check!(Ngrams::classify_bigram(&key2, &key1).kinds == vec![BigramKind::Other]);
    }
}
