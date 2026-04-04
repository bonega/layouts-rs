use crate::layout::{Layout, Pos};

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum SwapMove {
    Single(Pos, Pos),
}

impl SwapMove {
    pub fn single_moves(positions: &[Pos]) -> Vec<Self> {
        let mut moves = Vec::new();
        for (i, &p1) in positions.iter().enumerate() {
            for &p2 in positions.iter().skip(i + 1) {
                moves.push(Self::Single(p1, p2));
            }
        }
        moves
    }

    pub fn apply<const C: usize, const R: usize>(&self, layout: &mut Layout<C, R>) {
        match self {
            SwapMove::Single(p1, p2) => {
                layout.swap_chars(p1, p2);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use assert2::check;

    use super::*;

    #[test]
    fn it_builds_single_moves() {
        let positions = vec![pos!(0, 0), pos!(1, 0), pos!(0, 1)];
        let swap_moves = SwapMove::single_moves(&positions);
        check!(
            swap_moves
                == vec![
                    SwapMove::Single(pos!(0, 0), pos!(1, 0)),
                    SwapMove::Single(pos!(0, 0), pos!(0, 1)),
                    SwapMove::Single(pos!(1, 0), pos!(0, 1)),
                ]
        );
    }

    #[test]
    fn it_builds_single_moves_from_single_position() {
        let swap_moves = SwapMove::single_moves(&[pos!(0, 0)]);
        check!(swap_moves == vec![]);
    }

    #[test]
    fn it_builds_single_moves_from_empty() {
        let swap_moves = SwapMove::single_moves(&[]);
        check!(swap_moves == vec![]);
    }

    #[test]
    fn it_applies_single_move() {
        let mut layout = Layout::<2, 2>::new(
            "abcd",
            vec![vec![1, 2], vec![1, 2]],
            vec![vec![1.0, 2.0], vec![3.0, 4.0]],
            vec![pos!(0, 0), pos!(0, 1)],
        )
        .unwrap();

        let swap = SwapMove::Single(pos!(0, 0), pos!(1, 0));
        swap.apply(&mut layout);

        check!(layout.key_for('a').unwrap().position == pos!(1, 0));
        check!(layout.key_for('c').unwrap().position == pos!(0, 0));
    }

    #[test]
    fn it_reverts_single_move_when_applied_twice() {
        let original = Layout::<2, 2>::new(
            "abcd",
            vec![vec![1, 2], vec![1, 2]],
            vec![vec![1.0, 2.0], vec![3.0, 4.0]],
            vec![pos!(0, 0), pos!(0, 1)],
        )
        .unwrap();

        let mut layout = original;
        let swap = SwapMove::Single(pos!(0, 0), pos!(1, 0));
        swap.apply(&mut layout);
        swap.apply(&mut layout);

        check!(layout.key_for('a').unwrap().position == pos!(0, 0));
        check!(layout.key_for('c').unwrap().position == pos!(1, 0));
    }
}
