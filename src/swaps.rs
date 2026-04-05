use crate::layout::{Layout, Pos};

#[derive(Clone, Debug, PartialEq)]
pub struct SwapMove(pub Vec<(Pos, Pos)>);

impl SwapMove {
    pub fn all_moves(positions: &[Pos]) -> Vec<Self> {
        let mut moves = Self::single_moves(positions);
        moves.extend(Self::column_moves(positions));
        moves.extend(Self::row_moves(positions));
        moves
    }

    pub fn single_moves(positions: &[Pos]) -> Vec<Self> {
        let mut moves = Vec::new();
        for (i, &p1) in positions.iter().enumerate() {
            for &p2 in positions.iter().skip(i + 1) {
                moves.push(Self(vec![(p1, p2)]));
            }
        }
        moves
    }

    pub fn column_moves(positions: &[Pos]) -> Vec<Self> {
        Self::group_moves(positions, |pos| pos.c)
    }

    pub fn row_moves(positions: &[Pos]) -> Vec<Self> {
        Self::group_moves(positions, |pos| pos.r)
    }

    fn group_moves(positions: &[Pos], key_fn: fn(Pos) -> usize) -> Vec<Self> {
        use std::collections::BTreeMap;

        let mut groups: BTreeMap<usize, Vec<Pos>> = BTreeMap::new();
        for &pos in positions {
            groups.entry(key_fn(pos)).or_default().push(pos);
        }

        let keys: Vec<usize> = groups.keys().copied().collect();
        let mut moves = Vec::new();

        for (i, &k1) in keys.iter().enumerate() {
            for &k2 in keys.iter().skip(i + 1) {
                let g1 = &groups[&k1];
                let g2 = &groups[&k2];
                let pairs: Vec<(Pos, Pos)> = g1.iter().copied().zip(g2.iter().copied()).collect();
                if !pairs.is_empty() {
                    moves.push(Self(pairs));
                }
            }
        }

        moves
    }

    pub fn apply<const C: usize, const R: usize>(&self, layout: &mut Layout<C, R>) {
        for &(p1, p2) in &self.0 {
            layout.swap_chars(&p1, &p2);
        }
    }
}

#[cfg(test)]
mod single_moves_tests {
    use assert2::check;

    use super::*;

    #[test]
    fn it_builds() {
        let positions = vec![pos!(0, 0), pos!(1, 0), pos!(0, 1)];
        let swap_moves = SwapMove::single_moves(&positions);
        check!(
            swap_moves
                == vec![
                    SwapMove(vec![(pos!(0, 0), pos!(1, 0))]),
                    SwapMove(vec![(pos!(0, 0), pos!(0, 1))]),
                    SwapMove(vec![(pos!(1, 0), pos!(0, 1))]),
                ]
        );
    }

    #[test]
    fn it_builds_from_single_position() {
        let swap_moves = SwapMove::single_moves(&[pos!(0, 0)]);
        check!(swap_moves == vec![]);
    }

    #[test]
    fn it_builds_from_empty() {
        let swap_moves = SwapMove::single_moves(&[]);
        check!(swap_moves == vec![]);
    }

    #[test]
    fn it_applies() {
        let mut layout = Layout::<2, 2>::new(
            "abcd",
            vec![vec![1, 2], vec![1, 2]],
            vec![vec![1.0, 2.0], vec![3.0, 4.0]],
            vec![pos!(0, 0), pos!(0, 1)],
        )
        .unwrap();

        let swap = SwapMove(vec![(pos!(0, 0), pos!(1, 0))]);
        swap.apply(&mut layout);

        check!(layout.key_for('a').unwrap().position == pos!(1, 0));
        check!(layout.key_for('c').unwrap().position == pos!(0, 0));
    }

    #[test]
    fn it_reverts_when_applied_twice() {
        let original = Layout::<2, 2>::new(
            "abcd",
            vec![vec![1, 2], vec![1, 2]],
            vec![vec![1.0, 2.0], vec![3.0, 4.0]],
            vec![pos!(0, 0), pos!(0, 1)],
        )
        .unwrap();

        let mut layout = original;
        let swap = SwapMove(vec![(pos!(0, 0), pos!(1, 0))]);
        swap.apply(&mut layout);
        swap.apply(&mut layout);

        check!(layout.key_for('a').unwrap().position == pos!(0, 0));
        check!(layout.key_for('c').unwrap().position == pos!(1, 0));
    }
}

#[cfg(test)]
mod column_moves_tests {
    use assert2::check;

    use super::*;

    #[test]
    fn it_builds() {
        let positions = vec![pos!(0, 0), pos!(1, 0), pos!(0, 1), pos!(1, 1)];
        let col_moves = SwapMove::column_moves(&positions);
        check!(
            col_moves
                == vec![SwapMove(vec![
                    (pos!(0, 0), pos!(0, 1)),
                    (pos!(1, 0), pos!(1, 1)),
                ])]
        );
    }

    #[test]
    fn it_builds_with_n_columns() {
        let positions = vec![
            pos!(0, 0),
            pos!(1, 0),
            pos!(0, 1),
            pos!(1, 1),
            pos!(0, 2),
            pos!(1, 2),
        ];
        let col_moves = SwapMove::column_moves(&positions);
        check!(
            col_moves
                == vec![
                    SwapMove(vec![(pos!(0, 0), pos!(0, 1)), (pos!(1, 0), pos!(1, 1))]),
                    SwapMove(vec![(pos!(0, 0), pos!(0, 2)), (pos!(1, 0), pos!(1, 2))]),
                    SwapMove(vec![(pos!(0, 1), pos!(0, 2)), (pos!(1, 1), pos!(1, 2))]),
                ]
        );
    }

    #[test]
    fn it_builds_with_n_rows() {
        let positions = vec![
            pos!(0, 0),
            pos!(1, 0),
            pos!(2, 0),
            pos!(0, 1),
            pos!(1, 1),
            pos!(2, 1),
        ];
        let col_moves = SwapMove::column_moves(&positions);
        check!(
            col_moves
                == vec![SwapMove(vec![
                    (pos!(0, 0), pos!(0, 1)),
                    (pos!(1, 0), pos!(1, 1)),
                    (pos!(2, 0), pos!(2, 1)),
                ])]
        );
    }

    #[test]
    fn it_builds_zips_to_shorter_column() {
        let positions = vec![pos!(0, 0), pos!(1, 0), pos!(2, 0), pos!(0, 1), pos!(1, 1)];
        let col_moves = SwapMove::column_moves(&positions);
        check!(
            col_moves
                == vec![SwapMove(vec![
                    (pos!(0, 0), pos!(0, 1)),
                    (pos!(1, 0), pos!(1, 1)),
                ])]
        );
    }

    #[test]
    fn it_builds_from_single_column() {
        let positions = vec![pos!(0, 0), pos!(1, 0)];
        let col_moves = SwapMove::column_moves(&positions);
        check!(col_moves == vec![]);
    }

    #[test]
    fn it_builds_from_empty() {
        let col_moves = SwapMove::column_moves(&[]);
        check!(col_moves == vec![]);
    }

    #[test]
    fn it_applies() {
        let mut layout = Layout::<2, 2>::new(
            "abcd",
            vec![vec![1, 2], vec![1, 2]],
            vec![vec![1.0, 2.0], vec![3.0, 4.0]],
            vec![pos!(0, 0), pos!(0, 1)],
        )
        .unwrap();

        let swap = SwapMove(vec![(pos!(0, 0), pos!(0, 1)), (pos!(1, 0), pos!(1, 1))]);
        swap.apply(&mut layout);

        check!(layout.key_for('a').unwrap().position == pos!(0, 1));
        check!(layout.key_for('b').unwrap().position == pos!(0, 0));
        check!(layout.key_for('c').unwrap().position == pos!(1, 1));
        check!(layout.key_for('d').unwrap().position == pos!(1, 0));
    }

    #[test]
    fn it_reverts_when_applied_twice() {
        let original = Layout::<2, 2>::new(
            "abcd",
            vec![vec![1, 2], vec![1, 2]],
            vec![vec![1.0, 2.0], vec![3.0, 4.0]],
            vec![pos!(0, 0), pos!(0, 1)],
        )
        .unwrap();

        let mut layout = original;
        let swap = SwapMove(vec![(pos!(0, 0), pos!(0, 1)), (pos!(1, 0), pos!(1, 1))]);
        swap.apply(&mut layout);
        swap.apply(&mut layout);

        check!(layout.key_for('a').unwrap().position == pos!(0, 0));
        check!(layout.key_for('b').unwrap().position == pos!(0, 1));
        check!(layout.key_for('c').unwrap().position == pos!(1, 0));
        check!(layout.key_for('d').unwrap().position == pos!(1, 1));
    }
}

#[cfg(test)]
mod row_moves_tests {
    use assert2::check;

    use super::*;

    #[test]
    fn it_builds() {
        let positions = vec![pos!(0, 0), pos!(0, 1), pos!(1, 0), pos!(1, 1)];
        let row_moves = SwapMove::row_moves(&positions);
        check!(
            row_moves
                == vec![SwapMove(vec![
                    (pos!(0, 0), pos!(1, 0)),
                    (pos!(0, 1), pos!(1, 1)),
                ])]
        );
    }

    #[test]
    fn it_builds_with_n_rows() {
        let positions = vec![
            pos!(0, 0),
            pos!(0, 1),
            pos!(1, 0),
            pos!(1, 1),
            pos!(2, 0),
            pos!(2, 1),
        ];
        let row_moves = SwapMove::row_moves(&positions);
        check!(
            row_moves
                == vec![
                    SwapMove(vec![(pos!(0, 0), pos!(1, 0)), (pos!(0, 1), pos!(1, 1))]),
                    SwapMove(vec![(pos!(0, 0), pos!(2, 0)), (pos!(0, 1), pos!(2, 1))]),
                    SwapMove(vec![(pos!(1, 0), pos!(2, 0)), (pos!(1, 1), pos!(2, 1))]),
                ]
        );
    }

    #[test]
    fn it_builds_with_n_columns() {
        let positions = vec![
            pos!(0, 0),
            pos!(0, 1),
            pos!(0, 2),
            pos!(1, 0),
            pos!(1, 1),
            pos!(1, 2),
        ];
        let row_moves = SwapMove::row_moves(&positions);
        check!(
            row_moves
                == vec![SwapMove(vec![
                    (pos!(0, 0), pos!(1, 0)),
                    (pos!(0, 1), pos!(1, 1)),
                    (pos!(0, 2), pos!(1, 2)),
                ])]
        );
    }

    #[test]
    fn it_builds_zips_to_shorter_row() {
        let positions = vec![pos!(0, 0), pos!(0, 1), pos!(0, 2), pos!(1, 0), pos!(1, 1)];
        let row_moves = SwapMove::row_moves(&positions);
        check!(
            row_moves
                == vec![SwapMove(vec![
                    (pos!(0, 0), pos!(1, 0)),
                    (pos!(0, 1), pos!(1, 1)),
                ])]
        );
    }

    #[test]
    fn it_builds_from_single_row() {
        let positions = vec![pos!(0, 0), pos!(0, 1)];
        let row_moves = SwapMove::row_moves(&positions);
        check!(row_moves == vec![]);
    }

    #[test]
    fn it_builds_from_empty() {
        let row_moves = SwapMove::row_moves(&[]);
        check!(row_moves == vec![]);
    }

    #[test]
    fn it_applies() {
        let mut layout = Layout::<2, 2>::new(
            "abcd",
            vec![vec![1, 2], vec![1, 2]],
            vec![vec![1.0, 2.0], vec![3.0, 4.0]],
            vec![pos!(0, 0), pos!(0, 1)],
        )
        .unwrap();

        let swap = SwapMove(vec![(pos!(0, 0), pos!(1, 0)), (pos!(0, 1), pos!(1, 1))]);
        swap.apply(&mut layout);

        check!(layout.key_for('a').unwrap().position == pos!(1, 0));
        check!(layout.key_for('b').unwrap().position == pos!(1, 1));
        check!(layout.key_for('c').unwrap().position == pos!(0, 0));
        check!(layout.key_for('d').unwrap().position == pos!(0, 1));
    }

    #[test]
    fn it_reverts_when_applied_twice() {
        let original = Layout::<2, 2>::new(
            "abcd",
            vec![vec![1, 2], vec![1, 2]],
            vec![vec![1.0, 2.0], vec![3.0, 4.0]],
            vec![pos!(0, 0), pos!(0, 1)],
        )
        .unwrap();

        let mut layout = original;
        let swap = SwapMove(vec![(pos!(0, 0), pos!(1, 0)), (pos!(0, 1), pos!(1, 1))]);
        swap.apply(&mut layout);
        swap.apply(&mut layout);

        check!(layout.key_for('a').unwrap().position == pos!(0, 0));
        check!(layout.key_for('b').unwrap().position == pos!(0, 1));
        check!(layout.key_for('c').unwrap().position == pos!(1, 0));
        check!(layout.key_for('d').unwrap().position == pos!(1, 1));
    }
}
