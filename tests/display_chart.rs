use crate::common::show_page;
use chart_js_wrapper::common::Size;
use chart_js_wrapper::{ChartConfig, ChartType, ScaleConfig};
use sailfish::TemplateSimple;

mod common;

#[test]
fn show_chart() {
    let chart_y_cat = ChartConfig::<f64, &str>::default()
        .with_title("Line and bar".into())
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

    let chart_y_cat_bar = ChartConfig::<f64, &str>::default()
        .with_title("Bar only".into())
        .add_series(
            ChartType::Bar,
            "second_set".to_string(),
            [(2.0,"Second"),(14.0,"Third"),(15.0,"Third"),(20.0,"Second")]
        ).enable_legend()
        //so, apparently, category labels for y should be reversed. why, oh why?
        .set_y_axis(ScaleConfig::new_category(true,vec!["First","Second","Third","Fourth"]))
        .build(Size::pixels(600),Size::pixels(400));


    let chart_x_cat = ChartConfig::<&str,f64>::default()
        .with_title("Something interesting".into())
        .add_series(
            ChartType::Line,
            "first_set".to_string(),
            vec![("First",12.5),("Second",14.0),("Third",15.0),("Fourth",10.0)] //you can use vectors or arrays
        )
        .add_series(
            ChartType::Bar,
            "second_set".to_string(),
            [("First",11.0),("Second",11.0),("Third",20.0),("Fourth",5.0)]
        )
        .add_series(
            ChartType::Line,
            "third_set".to_string(),
            [("First",2.0),("Third",14.0),("Fifth",15.0),("First",20.0)]
        ).enable_legend()
        //labels for x-axis should not be reversed, though, those are fine
        .set_x_axis(ScaleConfig::new_category(false,vec!["First","Second","Third","Fourth"]))
        .build(Size::pixels(600),Size::pixels(400));



    let numeric_chart = ChartConfig::<f64, f64>::default()
        .with_title("Something completely different".into())
        .add_linear_regression_series(
            "set 2",
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
        ).unwrap()
        .add_linear_regression_series(
            "set 1",
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
        ).unwrap()
        .build(Size::pixels(600),Size::pixels(400));

    let chart_with_custom_labels = ChartConfig::<f64, f64>::default()
    .with_title("Custom labels".into())
    .add_series(
        ChartType::Line,
        "first_set".to_string(),
        vec![(12.5,12.5,"tooltip1"), (14.0,14.0,"tooltip2"), (15.0,15.0,"tooltip3"), (10.0,10.0,"tooltip4") ]
    )
        .add_series(
            ChartType::Line,
            "second_set".to_string(),
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
        )    .build(Size::pixels(600),Size::pixels(400));

    let mut body = chart_y_cat.render_once().unwrap();
    body.push_str(chart_y_cat_bar.render_once().unwrap().as_str());
    body.push_str(chart_x_cat.render_once().unwrap().as_str());
    body.push_str(numeric_chart.render_once().unwrap().as_str());
    body.push_str(chart_with_custom_labels.render_once().unwrap().as_str());

    show_page(&body);
}