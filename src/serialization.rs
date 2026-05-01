use std::collections::HashMap;
use std::str::FromStr;

use serde::{
    Deserialize, Deserializer,
    de::{self, Error, IntoDeserializer},
};

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
    struct ConfigSource {
        layout: layout::Config,
        optimization: OptimizationConfigSource,
    }

    impl_deserialize_with_from!(ConfigSource, Config);

    #[derive(Deserialize)]
    #[mapping::map_struct_to(OptimizationConfig)]
    struct OptimizationConfigSource {
        targets: Targets,
        simulated_annealing: SimulatedAnnealingConfig,
    }

    impl_deserialize_with_from!(OptimizationConfigSource, OptimizationConfig);
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
    enum PosSource {
        Array([usize; 2]),
        String(String),
    }

    impl FromStr for Pos {
        type Err = String;
        fn from_str(s: &str) -> Result<Self, Self::Err> {
            let trimmed = s.trim().trim_start_matches('[').trim_end_matches(']');
            let parts: Vec<&str> = trimmed.split(',').map(str::trim).collect();
            if parts.len() != 2 {
                return Err(format!("expected '[r, c]', got '{}'", s));
            }
            let r = parts[0]
                .parse::<usize>()
                .map_err(|_| format!("invalid row value: {}", parts[0]))?;
            let c = parts[1]
                .parse::<usize>()
                .map_err(|_| format!("invalid column value: {}", parts[1]))?;
            Ok(Self { r, c })
        }
    }

    impl<'de> Deserialize<'de> for Pos {
        fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
            let raw: PosSource = Deserialize::deserialize(deserializer)?;
            match raw {
                PosSource::Array(array) => Ok(Self {
                    r: array[0],
                    c: array[1],
                }),
                PosSource::String(s) => s
                    .parse::<Pos>()
                    .map_err(|e| D::Error::custom(format!("invalid position format: {}", e))),
            }
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
    use super::*;
    use crate::layout::{Finger, FingerKind, Hand};

    #[derive(Debug, Deserialize)]
    #[serde(untagged)]
    enum FingerSource {
        U8(u8),
        String(String),
    }

    #[derive(Debug, Deserialize)]
    #[serde(rename_all = "snake_case")]
    #[mapping::map_enum_to(FingerKind)]
    enum FingerKindSource {
        Pinky,
        Ring,
        Middle,
        Index,
        Thumb,
    }

    #[derive(Debug, Deserialize)]
    #[serde(rename_all = "snake_case")]
    #[mapping::map_enum_to(Hand)]
    enum HandSource {
        Left,
        Right,
    }

    impl FromStr for Finger {
        type Err = String;
        fn from_str(s: &str) -> Result<Self, Self::Err> {
            if let Ok(v) = s.trim().parse::<u8>() {
                return Finger::try_from(v);
            }

            let (hand_str, kind_str) = s
                .split_once('_')
                .ok_or_else(|| format!("invalid finger format: {}", s))?;

            let hand = HandSource::deserialize(hand_str.trim().into_deserializer())
                .map_err(|e: de::value::Error| e.to_string())?;
            let kind = FingerKindSource::deserialize(kind_str.trim().into_deserializer())
                .map_err(|e: de::value::Error| e.to_string())?;

            Ok(Finger::new(hand.into(), kind.into()))
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

    impl<'de> Deserialize<'de> for Finger {
        fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where
            D: Deserializer<'de>,
        {
            let value = FingerSource::deserialize(deserializer)?;
            match value {
                FingerSource::U8(v) => Finger::try_from(v).map_err(de::Error::custom),
                FingerSource::String(s) => s.parse::<Finger>().map_err(de::Error::custom),
            }
        }
    }
}

mod layout_finger_kind {
    use crate::layout::FingerKind;

    use super::*;

    #[derive(Debug, Deserialize)]
    #[mapping::map_enum(FingerKind)]
    #[serde(rename_all = "snake_case")]
    pub enum FingerKindSource {
        Pinky,
        Ring,
        Middle,
        Index,
        Thumb,
    }

    impl_deserialize_with_from!(FingerKindSource, FingerKind);
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

    #[derive(Debug, Deserialize)]
    #[mapping::map_struct_to(Config)]
    struct ConfigSource {
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

    impl_deserialize_with_from!(ConfigSource, Config);
}

mod ngrams_handedness {
    use crate::ngrams::Handedness;

    use super::*;

    #[derive(Debug, Deserialize)]
    #[mapping::map_enum(Handedness)]
    #[serde(rename_all = "snake_case")]
    pub enum HandednessSource {
        Same,
        Alternate,
    }

    impl_deserialize_with_from!(HandednessSource, Handedness);
}

mod ngrams_redirect_strength {
    use crate::ngrams::RedirectStrength;

    use super::*;

    #[derive(Debug, Deserialize)]
    #[mapping::map_enum(RedirectStrength)]
    #[serde(rename_all = "snake_case")]
    pub enum RedirectStrengthSource {
        Weak,
        Strong,
    }

    impl_deserialize_with_from!(RedirectStrengthSource, RedirectStrength);
}

mod ngrams_roll_direction {
    use crate::ngrams::RollDirection;

    use super::*;

    #[derive(Debug, Deserialize)]
    #[mapping::map_enum(RollDirection)]
    #[serde(rename_all = "snake_case")]
    pub enum RollDirectionSource {
        In,
        Out,
    }

    impl_deserialize_with_from!(RollDirectionSource, RollDirection);
}
