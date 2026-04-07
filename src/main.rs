use layouts_rs::{
    analyzer::Analyzer,
    corpus::Corpus,
    layout::{Layout, Pos, layout_string},
    optimizer::{Optimizer, Weights},
    report::{Report, ReportMetrics},
};

fn main() {
    let layout = Layout::<3, 12>::new(
        &layout_string("qwerty"),
        vec![
            vec![1, 1, 2, 3, 4, 4, 7, 7, 8, 9, 10, 10],
            vec![1, 1, 2, 3, 4, 4, 7, 7, 8, 9, 10, 10],
            vec![1, 1, 2, 3, 4, 4, 7, 7, 8, 9, 10, 10],
        ],
        vec![
            vec![3.0, 2.0, 2.0, 2.0, 2.0, 2.0, 2.0, 2.0, 2.0, 2.0, 2.0, 3.0],
            vec![2.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 2.0],
            vec![3.0, 2.0, 2.0, 2.0, 2.0, 2.0, 2.0, 2.0, 2.0, 2.0, 2.0, 3.0],
        ],
        [
            (1, Pos::new(1, 1)),
            (2, Pos::new(1, 2)),
            (3, Pos::new(1, 3)),
            (4, Pos::new(1, 4)),
            (7, Pos::new(1, 7)),
            (8, Pos::new(1, 8)),
            (9, Pos::new(1, 9)),
            (10, Pos::new(1, 10)),
        ]
        .into(),
    )
    .unwrap();

    let corpus = Corpus::new([("hello".to_string(), 10.0)]);
    let analyzer = Analyzer::new(corpus);
    let mut report_metrics = ReportMetrics::default();
    analyzer.analyze(&layout, &mut report_metrics);
    let report = Report::from(report_metrics);

    println!("Initial Layout:");
    println!("{layout}");
    println!("{report}");

    let optimizer = Optimizer::new(analyzer, Weights { effort: 1.0 });
    let optimized_layout =
        optimizer.optimize(&layout, 10, None, &['h'].into_iter().collect(), Some(2));

    let corpus = Corpus::new([("hello".to_string(), 10.0)]);
    let analyzer = Analyzer::new(corpus);
    let mut report_metrics = ReportMetrics::default();
    analyzer.analyze(&optimized_layout, &mut report_metrics);
    let report = Report::from(report_metrics);

    println!("-----------------------");
    println!("Optimized Layout:");
    println!("{optimized_layout}");
    println!("{report}");
}
