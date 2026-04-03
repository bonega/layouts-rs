use layouts_rs::{
    analyzer::Analyzer,
    corpus::Corpus,
    layout::{Layout, Pos},
    report::{Report, ReportMetrics},
};

fn main() {
    let corpus = Corpus::new([("hello".to_string(), 10.0)]);
    let layout = Layout::new(
        r#"
            _ q w e r t   y u i o p _
            " a s d f g   h j k l ; '
            _ z x c v b   n m , . / _
            "#,
        vec![
            vec![1, 1, 2, 3, 4, 4, 7, 7, 8, 9, 10, 10],
            vec![1, 1, 2, 3, 4, 4, 7, 7, 8, 9, 10, 10],
            vec![1, 1, 2, 3, 4, 4, 7, 7, 8, 9, 10, 10],
        ],
        vec![
            Pos::new(1, 1),
            Pos::new(1, 2),
            Pos::new(1, 3),
            Pos::new(1, 4),
            Pos::new(1, 7),
            Pos::new(1, 8),
            Pos::new(1, 9),
            Pos::new(1, 10),
        ],
    )
    .unwrap();

    let mut report_metrics = ReportMetrics::default();

    let analyzer = Analyzer::new(corpus);
    analyzer.analyze(&layout, &mut report_metrics);

    let report = Report::from(report_metrics);

    println!("{report}");
}
