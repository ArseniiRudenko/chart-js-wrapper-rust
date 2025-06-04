use sailfish::TemplateSimple;
use chart_js_wrapper_rust::{ChartConfig, ChartType};
use chart_js_wrapper_rust::common::Size;
use crate::common::show_page;

mod common;

#[test]
fn show_chart() {
    let chart = ChartConfig::<f64, &str>::default()
        .title_str("Something interesting".to_string())
        .add_series(
            ChartType::Line,
            "first_set".to_string(),
            vec![(12.5,"First"),(14.0,"Second"),(15.0,"Third"),(10.0,"Fourth")] //you can use vectors or arrays
        )
        .add_series(
            ChartType::Bar,
            "second_set".to_string(),
            [(2.0,"First"),(14.0,"Third"),(15.0,"Third"),(20.0,"First")]
        ).enable_legend()
        .build(Size::pixels(600),Size::pixels(400));

    let numeric_chart = ChartConfig::<f64, f64>::default()
        .title_str("Something completely different".to_string())
        .add_linear_regression_series(
            "regression set2",
            vec![
                (1.0,1.0),
                (1.0,2.0),
                (3.5,3.0),
                (4.0,4.0),
                (4.1,1.0),
                (4.1,3.0),
                (5.0,4.0),
                (14.0,3.0),
                (15.0,1.0),
                (20.0,1.0)
            ]
        )
        .add_linear_regression_series(
            "regression set",
            [
                (1.0,11.0),
                (1.0,20.0),
                (3.5,30.0),
                (4.0,40.0),
                (4.1,11.0),
                (4.1,35.0),
                (5.0,40.0),
                (14.0,33.0),
                (15.0,31.0),
                (20.0,11.0)
            ]
        ) .build(Size::pixels(600),Size::pixels(400));

    let mut body = chart.render_once().unwrap();
    body.push_str(numeric_chart.render_once().unwrap().as_str());
    show_page(&body);
}