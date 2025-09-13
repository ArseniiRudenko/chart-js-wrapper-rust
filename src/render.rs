use sailfish::TemplateSimple;
use serde::Serialize;
use crate::common::Size;
use crate::options::ChartConfig;
use crate::serde::WithTypeAndSerializer;

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
#[template(rm_whitespace = true)]
pub struct Chart<X,Y>
where X: WithTypeAndSerializer+Serialize, Y: WithTypeAndSerializer+Serialize
{
    chart_target_id: String,
    width: Size,
    height: Size,
    options: ChartConfig<X,Y>
}

impl<X,Y> Chart<X,Y> where X: WithTypeAndSerializer+Serialize, Y: WithTypeAndSerializer+Serialize
{
    pub fn new(chart_target_id: String, width: Size, height: Size, options: ChartConfig<X,Y>) -> Self {
        Self {
            chart_target_id,
            width,
            height,
            options
        }
    }
}