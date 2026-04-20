use std::collections::HashMap;

use anyhow::anyhow;
use pest::{Parser, iterators::Pair};
use pest_derive::Parser;

#[derive(Debug, PartialEq, Eq)]
pub struct Rules {
    pub metrics: Vec<Metric>,
    pub stats: Stats,
    pub targets: Vec<Target>,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Metric {
    pub name: String,
    pub category: MetricCategory,
    pub value: MetricValue,
    pub condition: Option<RustExpression>,
    pub group: Option<MetricGroup>,
    pub ty: Ty,
}
impl<'i> From<Pair<'i, Rule>> for Metric {
    fn from(pair: Pair<'i, Rule>) -> Self {
        let mut inner = pair.into_inner();
        let name = inner.next().unwrap().as_str().to_string();
        let category = inner.next().unwrap().into();

        let mut value = None;
        let mut condition = None;
        let mut group: Option<MetricGroup> = None;

        for clause in inner {
            match clause.as_rule() {
                Rule::metric_condition => condition = Some(unwrap_inner(clause).into()),
                Rule::metric_value => value = Some(clause.into()),
                Rule::metric_group => group = Some(clause.into()),
                _ => unreachable!(),
            }
        }

        let ty = group
            .as_ref()
            .map(|g| Ty::Map(g.kind.clone()))
            .unwrap_or(Ty::Scalar);

        Metric {
            name,
            category,
            condition: condition,
            value: value.unwrap_or_default(),
            group,
            ty,
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum MetricValue {
    Set(RustExpression),
    Add(RustExpression),
}
impl<'i> From<Pair<'i, Rule>> for MetricValue {
    fn from(pair: Pair<'i, Rule>) -> Self {
        let pair = unwrap_inner(pair);
        match pair.as_rule() {
            Rule::metric_value_set => Self::Set(unwrap_inner(pair).into()),
            Rule::metric_value_add => Self::Add(unwrap_inner(pair).into()),
            _ => unreachable!(),
        }
    }
}
impl Default for MetricValue {
    fn default() -> Self {
        MetricValue::Add(RustExpression("count".to_string()))
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct MetricGroup {
    pub key: String,
    pub kind: String,
}
impl<'i> From<Pair<'i, Rule>> for MetricGroup {
    fn from(pair: Pair<'i, Rule>) -> Self {
        let mut inner = pair.into_inner();
        let key = inner.next().unwrap().as_str().to_string();
        let kind = inner.next().unwrap().as_str().to_string();
        Self { key, kind }
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum MetricCategory {
    Unigram,
    Bigram(Option<String>),
    Trigram(Option<String>),
}
impl<'i> From<Pair<'i, Rule>> for MetricCategory {
    fn from(pair: Pair<'i, Rule>) -> Self {
        match pair.as_rule() {
            Rule::category_unigram => Self::Unigram,
            Rule::category_bigram => {
                Self::Bigram(pair.into_inner().next().map(|p| p.as_str().to_string()))
            }
            Rule::category_trigram => {
                Self::Trigram(pair.into_inner().next().map(|p| p.as_str().to_string()))
            }
            _ => unreachable!("{:?}", pair.as_rule()),
        }
    }
}
impl MetricCategory {
    pub fn is_unigram(&self) -> bool {
        matches!(self, Self::Unigram)
    }

    pub fn is_bigram_no_kind(&self) -> bool {
        matches!(self, Self::Bigram(None))
    }

    pub fn is_bigram_kind(&self, kind: &str) -> bool {
        self == &Self::Bigram(Some(kind.to_string()))
    }

    pub fn is_trigram_no_kind(&self) -> bool {
        matches!(self, Self::Trigram(None))
    }

    pub fn is_trigram_kind(&self, kind: &str) -> bool {
        self == &Self::Trigram(Some(kind.to_string()))
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Stats {
    pub lets: Vec<StatsLet>,
    pub sections: Vec<StatsSection>,
}
impl<'i> From<Pair<'i, Rule>> for Stats {
    fn from(pair: Pair<'i, Rule>) -> Self {
        let mut lets = vec![];
        let mut sections = vec![];

        for item in pair.into_inner() {
            match item.as_rule() {
                Rule::stats_let => lets.push(item.into()),
                Rule::stats_section => sections.push(item.into()),
                _ => unreachable!(),
            }
        }

        Self { lets, sections }
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct StatsSection {
    pub name: String,
    pub stats: Vec<Stat>,
}
impl<'i> From<Pair<'i, Rule>> for StatsSection {
    fn from(pair: Pair<'i, Rule>) -> Self {
        let mut inner = pair.into_inner();
        let name = inner.next().unwrap().as_str().to_string();
        let stats = inner.map(|p| p.into()).collect();
        Self { name, stats }
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct StatsLet {
    pub name: String,
    pub value: Expression,
}
impl<'i> From<Pair<'i, Rule>> for StatsLet {
    fn from(pair: Pair<'i, Rule>) -> Self {
        let mut inner = pair.into_inner();
        let name = inner.next().unwrap().as_str().to_string();
        let value = inner.next().unwrap().into();
        Self { name, value }
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Stat {
    pub name: String,
    pub value: StatValue,
    pub ty: Ty,
}
impl<'i> From<Pair<'i, Rule>> for Stat {
    fn from(pair: Pair<'i, Rule>) -> Self {
        let mut inner = pair.into_inner();
        let name = inner.next().unwrap().as_str().to_string();
        let value = inner.next().unwrap().into();
        Self {
            name,
            value,
            ty: Ty::Scalar,
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum StatValue {
    Percent(Expression, Expression),
    Ratio(Expression, Expression),
    Normalize(Expression, Expression),
    Reference(String),
}
impl<'i> From<Pair<'i, Rule>> for StatValue {
    fn from(pair: Pair<'i, Rule>) -> Self {
        match pair.as_rule() {
            Rule::call_percent => {
                let mut inner = pair.into_inner();
                StatValue::Percent(inner.next().unwrap().into(), inner.next().unwrap().into())
            }
            Rule::call_ratio => {
                let mut inner = pair.into_inner();
                StatValue::Ratio(inner.next().unwrap().into(), inner.next().unwrap().into())
            }
            Rule::call_normalize => {
                let mut inner = pair.into_inner();
                StatValue::Normalize(inner.next().unwrap().into(), inner.next().unwrap().into())
            }
            Rule::call_identity => {
                let mut inner = pair.into_inner();
                StatValue::Reference(inner.next().unwrap().as_str().to_string())
            }
            Rule::call_reference => {
                let mut inner = pair.into_inner();
                StatValue::Reference(inner.next().unwrap().as_str().to_string())
            }
            _ => unreachable!("{:?}", pair.as_rule()),
        }
    }
}
impl StatValue {
    fn ty(&self, env: &HashMap<String, Ty>) -> anyhow::Result<Ty> {
        match self {
            StatValue::Percent(e1, e2) => {
                let ty1 = e1.ty(env)?;
                let ty2 = e2.ty(env)?;

                if !ty1.is_scalar() || !ty2.is_scalar() {
                    return Err(anyhow!("percent value must be scalar: {ty1:?} vs {ty2:?}"));
                }

                Ok(ty1)
            }
            StatValue::Ratio(e1, e2) => {
                let ty1 = e1.ty(env)?;
                let ty2 = e2.ty(env)?;

                if !ty1.is_scalar() || !ty2.is_scalar() {
                    return Err(anyhow!("ratio value must be scalar: {ty1:?} vs {ty2:?}"));
                }

                Ok(ty1)
            }
            StatValue::Normalize(e1, e2) => {
                let ty1 = e1.ty(env)?;
                let ty2 = e2.ty(env)?;

                if !(ty1.is_map() && ty2.is_scalar()) {
                    return Err(anyhow!(
                        "normalize values must be map and scalar: {ty1:?} vs {ty2:?}"
                    ));
                }

                Ok(ty1)
            }
            StatValue::Reference(reference) => env
                .get(reference)
                .cloned()
                .ok_or_else(|| anyhow!("undefined reference: {}", reference)),
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Target {
    pub name: String,
    pub source: String,
    pub ty: Ty,
}
impl<'i> From<Pair<'i, Rule>> for Target {
    fn from(pair: Pair<'i, Rule>) -> Self {
        let mut inner = pair.into_inner();
        let name = inner.next().unwrap().as_str().to_string();
        let source = inner.next().unwrap().as_str().to_string();
        Self {
            name,
            source,
            ty: Ty::Scalar,
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum Expression {
    Ref(String),
    Sum {
        reference: String,
        condition: Option<RustExpression>,
    },
    Add(Box<Expression>, Box<Expression>),
}
impl<'i> From<Pair<'i, Rule>> for Expression {
    fn from(pair: Pair<'i, Rule>) -> Self {
        match pair.as_rule() {
            Rule::expression => {
                let mut terms = pair.into_inner().map(Expression::from);
                let first = terms.next().unwrap();
                terms.fold(first, |acc, t| Expression::Add(Box::new(acc), Box::new(t)))
            }
            Rule::call_sum => {
                let mut inner = pair.into_inner();
                let reference = inner.next().unwrap().as_str().to_string();
                let condition = inner.next().map(|p| p.into());
                Expression::Sum {
                    reference,
                    condition,
                }
            }
            Rule::ident => Expression::Ref(pair.as_str().to_string()),
            _ => unreachable!("{:?}", pair.as_rule()),
        }
    }
}
impl Expression {
    fn ty(&self, env: &HashMap<String, Ty>) -> anyhow::Result<Ty> {
        match self {
            Expression::Ref(name) => env
                .get(name)
                .cloned()
                .ok_or_else(|| anyhow!("undefined reference: {}", name)),
            Expression::Sum { reference, .. } => {
                let ty = env
                    .get(reference)
                    .cloned()
                    .ok_or_else(|| anyhow!("undefined reference: {}", reference))?;

                if ty == Ty::Scalar {
                    return Err(anyhow!("cannot sum a scalar value: {}", reference));
                }

                Ok(Ty::Scalar)
            }
            Expression::Add(a, b) => {
                let ty1 = a.ty(env)?;
                let ty2 = b.ty(env)?;

                if ty1 != Ty::Scalar || ty2 != Ty::Scalar {
                    return Err(anyhow!("can only add scalar values: {:?} + {:?}", ty1, ty2));
                }

                Ok(Ty::Scalar)
            }
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct RustExpression(pub String);
impl<'i> From<Pair<'i, Rule>> for RustExpression {
    fn from(pair: Pair<'i, Rule>) -> Self {
        Self(unwrap_inner(pair).as_str().to_string())
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum Ty {
    Scalar,
    Map(String),
}
impl Ty {
    fn is_scalar(&self) -> bool {
        matches!(self, Ty::Scalar)
    }

    fn is_map(&self) -> bool {
        matches!(self, Ty::Map(_))
    }
}

#[derive(Parser)]
#[grammar = "src/grammar.pest"]
struct RulesParser;

pub fn parse(src: &str) -> anyhow::Result<Rules> {
    let pairs = RulesParser::parse(Rule::rules, src).map_err(|e| anyhow!("parse error:\n{e}"))?;

    let mut metrics = vec![];
    let mut stats_pair = None;
    let mut targets: Vec<Target> = vec![];

    for pair in pairs.into_iter().next().unwrap().into_inner() {
        match pair.as_rule() {
            Rule::metrics => {
                for metric in pair.into_inner() {
                    metrics.push(metric.into());
                }
            }
            Rule::stats => stats_pair = Some(pair),
            Rule::targets => {
                for target in pair.into_inner() {
                    targets.push(target.into());
                }
            }
            Rule::EOI => {}
            _ => unreachable!(),
        }
    }

    let mut stats: Stats = stats_pair.map(|p| p.into()).unwrap_or_else(|| Stats {
        lets: vec![],
        sections: vec![],
    });

    let mut env: HashMap<String, Ty> = metrics
        .iter()
        .map(|m: &Metric| (m.name.clone(), m.ty.clone()))
        .collect();

    for stats_let in &stats.lets {
        let ty = stats_let.value.ty(&env)?;
        env.insert(stats_let.name.clone(), ty);
    }

    for section in &mut stats.sections {
        for stat in &mut section.stats {
            let ty = stat.value.ty(&env)?;
            stat.ty = ty;
        }
    }

    let stats_env: HashMap<String, Ty> = stats
        .sections
        .iter()
        .flat_map(|s| {
            s.stats
                .iter()
                .map(|stat| (format!("{}.{}", s.name, stat.name), stat.ty.clone()))
        })
        .collect();

    for target in &mut targets {
        let ty = stats_env
            .get(&target.source)
            .cloned()
            .ok_or_else(|| anyhow!("undefined reference: {}", target.source))?;
        target.ty = ty;
    }

    Ok(Rules {
        metrics,
        stats,
        targets,
    })
}

fn unwrap_inner(pair: Pair<Rule>) -> Pair<Rule> {
    pair.into_inner().next().unwrap()
}

#[cfg(test)]
mod tests {
    use super::*;
    use assert2::check;

    const SRC: &str = r#"
            metrics {
                finger_usage on unigram
                    value += "count"
                    where "ngram.key.ch != ' '"
                    group by ngram.key.finger as Finger;
            }

            stats {
                let left_hand_usage  = sum(finger_usage) where "hand == Hand::Left";
                let right_hand_usage = sum(finger_usage) where "hand == Hand::Right";

                section general {
                    left_hand_usage = percent(left_hand_usage, left_hand_usage + right_hand_usage);
                    finger_usage = normalize(finger_usage, sum(finger_usage));
                }
            }

            targets {
                left_hand_usage = general.left_hand_usage;
                finger_usage = general.finger_usage;
            }
        "#;

    #[test]
    fn it_parses_metrics() {
        let file = parse(SRC).unwrap();
        check!(
            file.metrics
                == vec![Metric {
                    name: "finger_usage".to_string(),
                    category: MetricCategory::Unigram,
                    value: MetricValue::Add(RustExpression("count".to_string())),
                    condition: Some(RustExpression("ngram.key.ch != ' '".to_string())),
                    ty: Ty::Map("Finger".to_string()),
                    group: Some(MetricGroup {
                        key: "ngram.key.finger".to_string(),
                        kind: "Finger".to_string(),
                    }),
                }]
        );
    }

    #[test]
    fn it_parses_stats() {
        let file = parse(SRC).unwrap();
        check!(
            file.stats
                == Stats {
                    lets: vec![
                        StatsLet {
                            name: "left_hand_usage".to_string(),
                            value: Expression::Sum {
                                reference: "finger_usage".to_string(),
                                condition: Some(RustExpression("hand == Hand::Left".to_string())),
                            },
                        },
                        StatsLet {
                            name: "right_hand_usage".to_string(),
                            value: Expression::Sum {
                                reference: "finger_usage".to_string(),
                                condition: Some(RustExpression("hand == Hand::Right".to_string())),
                            },
                        },
                    ],
                    sections: vec![StatsSection {
                        name: "general".to_string(),
                        stats: vec![
                            Stat {
                                name: "left_hand_usage".to_string(),
                                value: StatValue::Percent(
                                    Expression::Ref("left_hand_usage".to_string()),
                                    Expression::Add(
                                        Box::new(Expression::Ref("left_hand_usage".to_string())),
                                        Box::new(Expression::Ref("right_hand_usage".to_string())),
                                    ),
                                ),
                                ty: Ty::Scalar,
                            },
                            Stat {
                                name: "finger_usage".to_string(),
                                value: StatValue::Normalize(
                                    Expression::Ref("finger_usage".to_string()),
                                    Expression::Sum {
                                        reference: "finger_usage".to_string(),
                                        condition: None,
                                    },
                                ),
                                ty: Ty::Map("Finger".to_string()),
                            }
                        ],
                    }],
                }
        );
    }

    #[test]
    fn it_parses_targets() {
        let file = parse(SRC).unwrap();
        check!(
            file.targets
                == vec![
                    Target {
                        name: "left_hand_usage".to_string(),
                        source: "general.left_hand_usage".to_string(),
                        ty: Ty::Map("Finger".to_string()),
                    },
                    Target {
                        name: "finger_usage".to_string(),
                        source: "general.finger_usage".to_string(),
                        ty: Ty::Scalar,
                    }
                ]
        );
    }
}
