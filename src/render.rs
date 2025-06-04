use sailfish::TemplateSimple;
use serde::Serialize;
use crate::common::Size;
use crate::options::ChartConfig;

#[derive(TemplateSimple)]
#[template(path = "one_page_chart.stpl")]
pub struct OnePage<'a>{
    title: &'a str,
    body: &'a str
}

impl<'a> OnePage<'a> {
    pub fn new(title: &'a str, body: &'a str) -> Self {
        Self {
            title,
            body
        }
    }
}


#[derive(TemplateSimple)]
#[template(path = "chart.stpl")]
pub struct Chart<X,Y>
where ChartConfig<X,Y>: Serialize {
    chart_target_id: String,
    width: Size,
    height: Size,
    options: ChartConfig<X,Y>
}

impl<X,Y> Chart<X,Y> where ChartConfig<X,Y>: Serialize {
    pub fn new(chart_target_id: String, width: Size, height: Size, options: ChartConfig<X,Y>) -> Self {
        Self {
            chart_target_id,
            width,
            height,
            options
        }
    }
}