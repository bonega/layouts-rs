use std::collections::HashMap;
use std::fmt;

use serde::{Deserialize, Deserializer, Serialize, de::Error};

macro_rules! impl_deserialize_with_from {
    ($repr:path, $final:path) => {
        impl<'de> Deserialize<'de> for $final {
            fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
                let value = <$repr>::deserialize(deserializer)?;
                Ok(value.into())
            }
        }
    };
}

mod config {
    use crate::config::{Config, OptimizationConfig};
    use crate::layout;
    use crate::optimizer::SimulatedAnnealingConfig;
    use crate::targets::Targets;

    use super::*;

    #[derive(Deserialize)]
    #[mapping::map_struct_to(Config)]
    struct ConfigRepr {
        layout: layout::Config,
        optimization: OptimizationConfigRepr,
    }

    #[derive(Deserialize)]
    #[mapping::map_struct_to(OptimizationConfig)]
    struct OptimizationConfigRepr {
        targets: Targets,
        simulated_annealing: SimulatedAnnealingConfig,
    }

    impl_deserialize_with_from!(OptimizationConfigRepr, OptimizationConfig);
    impl_deserialize_with_from!(ConfigRepr, Config);
}

mod matrix {
    use crate::matrix::Matrix;

    use super::*;

    impl<'de, T: Deserialize<'de>> Deserialize<'de> for Matrix<T> {
        fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
            let data: Vec<Vec<T>> = Vec::deserialize(deserializer)?;
            Self::new(data).map_err(D::Error::custom)
        }
    }
}

mod matrix_pos {
    use crate::matrix::Pos;

    use super::*;

    #[derive(Debug, Deserialize)]
    #[serde(untagged)]
    enum PosRepr {
        Array([usize; 2]),
        String(String),
    }

    impl<'de> Deserialize<'de> for Pos {
        fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
            let raw: PosRepr = Deserialize::deserialize(deserializer)?;
            match raw {
                PosRepr::Array(array) => Ok(Self {
                    r: array[0],
                    c: array[1],
                }),
                PosRepr::String(s) => {
                    let trimmed = s.trim().trim_start_matches('[').trim_end_matches(']');
                    let parts: Vec<&str> = trimmed.split(',').map(str::trim).collect();
                    if parts.len() != 2 {
                        return Err(D::Error::custom(format!("expected '[r, c]', got '{}'", s)));
                    }
                    let r = parts[0].parse::<usize>().map_err(D::Error::custom)?;
                    let c = parts[1].parse::<usize>().map_err(D::Error::custom)?;
                    Ok(Self { r, c })
                }
            }
        }
    }

    impl fmt::Display for Pos {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            write!(f, "[{}, {}]", self.r, self.c)
        }
    }
}

mod layout_coords {
    use crate::layout::Coords;

    use super::*;

    impl<'de> Deserialize<'de> for Coords {
        fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
            let raw: [f64; 2] = Deserialize::deserialize(deserializer)?;
            Ok(Self {
                y: raw[0],
                x: raw[1],
            })
        }
    }
}

mod layout_finger {
    use crate::layout::{Finger, FingerKind, Hand};

    use super::*;

    #[derive(Debug, Deserialize)]
    #[serde(untagged)]
    enum FingerRepr {
        Enum(EnumFinger),
        String(String),
        U8(u8),
    }

    #[derive(Debug, Deserialize, Serialize, strum::Display)]
    #[serde(rename_all = "snake_case")]
    #[strum(serialize_all = "snake_case")]
    enum EnumFinger {
        LeftPinky,
        LeftRing,
        LeftMiddle,
        LeftIndex,
        LeftThumb,
        RightThumb,
        RightIndex,
        RightMiddle,
        RightRing,
        RightPinky,
    }

    impl From<EnumFinger> for Finger {
        fn from(value: EnumFinger) -> Self {
            match value {
                EnumFinger::LeftPinky => Finger::new(Hand::Left, FingerKind::Pinky),
                EnumFinger::LeftRing => Finger::new(Hand::Left, FingerKind::Ring),
                EnumFinger::LeftMiddle => Finger::new(Hand::Left, FingerKind::Middle),
                EnumFinger::LeftIndex => Finger::new(Hand::Left, FingerKind::Index),
                EnumFinger::LeftThumb => Finger::new(Hand::Left, FingerKind::Thumb),
                EnumFinger::RightThumb => Finger::new(Hand::Right, FingerKind::Thumb),
                EnumFinger::RightIndex => Finger::new(Hand::Right, FingerKind::Index),
                EnumFinger::RightMiddle => Finger::new(Hand::Right, FingerKind::Middle),
                EnumFinger::RightRing => Finger::new(Hand::Right, FingerKind::Ring),
                EnumFinger::RightPinky => Finger::new(Hand::Right, FingerKind::Pinky),
            }
        }
    }

    impl From<Finger> for EnumFinger {
        fn from(value: Finger) -> Self {
            match (value.hand, value.kind) {
                (Hand::Left, FingerKind::Pinky) => EnumFinger::LeftPinky,
                (Hand::Left, FingerKind::Ring) => EnumFinger::LeftRing,
                (Hand::Left, FingerKind::Middle) => EnumFinger::LeftMiddle,
                (Hand::Left, FingerKind::Index) => EnumFinger::LeftIndex,
                (Hand::Left, FingerKind::Thumb) => EnumFinger::LeftThumb,
                (Hand::Right, FingerKind::Thumb) => EnumFinger::RightThumb,
                (Hand::Right, FingerKind::Index) => EnumFinger::RightIndex,
                (Hand::Right, FingerKind::Middle) => EnumFinger::RightMiddle,
                (Hand::Right, FingerKind::Ring) => EnumFinger::RightRing,
                (Hand::Right, FingerKind::Pinky) => EnumFinger::RightPinky,
            }
        }
    }

    impl<'de> Deserialize<'de> for Finger {
        fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
            let value = FingerRepr::deserialize(deserializer)?;
            match value {
                FingerRepr::Enum(e) => Ok(e.into()),
                FingerRepr::U8(v) => Ok(From::from(Finger::try_from(v).map_err(D::Error::custom)?)),
                FingerRepr::String(v) => Ok(From::from(
                    Finger::try_from(v.parse::<u8>().map_err(D::Error::custom)?)
                        .map_err(D::Error::custom)?,
                )),
            }
        }
    }

    impl fmt::Display for Finger {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            let enum_finger: EnumFinger = (*self).into();
            enum_finger.fmt(f)
        }
    }

    impl TryFrom<u8> for Finger {
        type Error = String;

        fn try_from(value: u8) -> Result<Self, Self::Error> {
            let (hand, kind) = match value {
                1 => (Hand::Left, FingerKind::Pinky),
                2 => (Hand::Left, FingerKind::Ring),
                3 => (Hand::Left, FingerKind::Middle),
                4 => (Hand::Left, FingerKind::Index),
                5 => (Hand::Left, FingerKind::Thumb),
                6 => (Hand::Right, FingerKind::Thumb),
                7 => (Hand::Right, FingerKind::Index),
                8 => (Hand::Right, FingerKind::Middle),
                9 => (Hand::Right, FingerKind::Ring),
                10 => (Hand::Right, FingerKind::Pinky),
                _ => return Err(format!("invalid finger value: {}", value)),
            };

            Ok(Finger { hand, kind })
        }
    }
}

mod layout_finger_kind {
    use crate::layout::FingerKind;

    use super::*;

    #[derive(Debug, Deserialize, strum::Display)]
    #[mapping::map_enum(FingerKind)]
    #[serde(rename_all = "snake_case")]
    #[strum(serialize_all = "snake_case")]
    pub enum FingerKindRepr {
        Pinky,
        Ring,
        Middle,
        Index,
        Thumb,
    }

    impl_deserialize_with_from!(FingerKindRepr, FingerKind);

    impl fmt::Display for FingerKind {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            let enum_finger_kind: FingerKindRepr = (*self).into();
            enum_finger_kind.fmt(f)
        }
    }
}

mod layout_key_size {
    use crate::layout::KeySize;

    use super::*;

    impl<'de> Deserialize<'de> for KeySize {
        fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
            let raw: [f64; 2] = Deserialize::deserialize(deserializer)?;
            Ok(Self {
                width: raw[0],
                height: raw[1],
            })
        }
    }
}

mod layout_config {
    use crate::{
        layout::{Config, Coords, Finger, KeySize},
        matrix::{Matrix, Pos},
    };

    use super::*;

    impl_deserialize_with_from!(ConfigRepr, Config);

    #[derive(Debug, Deserialize)]
    #[mapping::map_struct_to(Config)]
    struct ConfigRepr {
        #[serde(deserialize_with = "deserialize_finger_assignment")]
        pub finger_assignment: Matrix<Option<Finger>>,
        pub finger_effort: Matrix<f64>,
        pub key_centers: Matrix<Coords>,
        pub key_size: KeySize,
        pub finger_home_positions: HashMap<Finger, Pos>,
    }

    fn deserialize_finger_assignment<'de, D: Deserializer<'de>>(
        deserializer: D,
    ) -> Result<Matrix<Option<Finger>>, D::Error> {
        let raw_data: Result<Vec<Vec<Option<Finger>>>, _> =
            Vec::<Vec<u8>>::deserialize(deserializer)?
                .into_iter()
                .map(|row| {
                    row.into_iter()
                        .map(|f| {
                            if f == 0 {
                                return Ok(None);
                            }

                            Finger::try_from(f).map(Some).map_err(Error::custom)
                        })
                        .collect()
                })
                .collect();

        Matrix::new(raw_data?).map_err(Error::custom)
    }
}
