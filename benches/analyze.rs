use std::collections::HashMap;
use std::fs;

use criterion::{Criterion, criterion_group, criterion_main};

use layouts_rs::analyzer::Analyzer;
use layouts_rs::corpus::Corpus;
use layouts_rs::layout::{Layout, Pos};
use layouts_rs::report::ReportMetrics;

fn load_corpus() -> Corpus {
    let content =
        fs::read_to_string("words-english.json").expect("Failed to read words-english.json");
    let word_items: HashMap<String, f64> =
        serde_json::from_str(&content).expect("Failed to parse words-english.json");
    Corpus::new(word_items)
}

fn load_layout() -> Layout<4, 12> {
    Layout::new(
        r#"
        _ q w e r t   y u i o p _
        _ a s d f g   h j k l * _
        _ z x c v b   n m * * * _
        _ _ _ _ _ _   _ _ _ _ _ _
        "#,
        vec![
            vec![1, 1, 2, 3, 4, 4, 7, 7, 8, 9, 10, 10],
            vec![1, 1, 2, 3, 4, 4, 7, 7, 8, 9, 10, 10],
            vec![1, 1, 2, 3, 4, 4, 7, 7, 8, 9, 10, 10],
            vec![0, 0, 0, 0, 5, 5, 6, 6, 0, 0, 0, 0],
        ],
        vec![
            vec![5.0, 3.0, 2.0, 1.0, 2.0, 7.0, 7.0, 2.0, 1.0, 2.0, 3.0, 5.0],
            vec![5.0, 1.0, 1.0, 1.0, 1.0, 5.0, 5.0, 1.0, 1.0, 1.0, 1.0, 5.0],
            vec![7.0, 3.0, 2.0, 2.0, 1.0, 8.0, 8.0, 1.0, 2.0, 2.0, 3.0, 7.0],
            vec![0.0, 0.0, 0.0, 0.0, 1.0, 1.0, 1.0, 1.0, 0.0, 0.0, 0.0, 0.0],
        ],
        HashMap::from([
            (1, Pos::new(1, 1)),
            (2, Pos::new(1, 2)),
            (3, Pos::new(1, 3)),
            (4, Pos::new(1, 4)),
            (5, Pos::new(3, 4)),
            (6, Pos::new(3, 7)),
            (7, Pos::new(1, 7)),
            (8, Pos::new(1, 8)),
            (9, Pos::new(1, 9)),
            (10, Pos::new(1, 10)),
        ]),
    )
    .expect("Failed to create layout")
}

fn bench_analyze(c: &mut Criterion) {
    let corpus = load_corpus();
    let layout = load_layout();
    let analyzer = Analyzer::new(corpus);

    c.bench_function("analyze", |b| {
        b.iter(|| {
            let mut metrics = ReportMetrics::default();
            analyzer.analyze(&layout, &mut metrics);
        });
    });
}

criterion_group!(benches, bench_analyze);
criterion_main!(benches);
