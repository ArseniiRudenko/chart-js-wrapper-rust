use serde::{Deserialize, Serialize};
use crate::colour::Rgb;

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ChartConfig<X,Y> {
    r#type: ChartType,

    data: ChartData<X,Y>,

    #[serde(skip_serializing_if = "Option::is_none")]
    options: Option<ChartOptions>
}

impl<X,Y> ChartConfig<X,Y> {
    pub fn new(r#type: ChartType, data: ChartData<X,Y>, options: Option<ChartOptions>) -> Self {
        Self {
            r#type,
            data,
            options
        }
    }
}


#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "lowercase")]
pub enum ChartType {
    Bubble,
    Bar,
    Line,
    Doughnut,
    Pie,
    Radar,
    PolarArea,
    Scatter
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ChartData<X,Y>{

    #[serde(skip_serializing_if = "Option::is_none")]
    labels: Option<Vec<String>>,

    datasets: Vec<Dataset<X,Y>>
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Dataset<X,Y>{

    label: String,

    data: Vec<DataElement<X,Y>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    fill: Option<Fill>,

    #[serde(skip_serializing_if = "Option::is_none")]
    border_color: Option<Rgb>,

    #[serde(skip_serializing_if = "Option::is_none")]
    background_color: Option<Rgb>
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct DataElement<X,Y>{
    x: X,
    y: Y,
    #[serde(skip_serializing_if = "Option::is_none")]
    r: Option<u32>
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(untagged)]
enum FillVariant{
    AbsIndex(u8),
    RelativeIndex(String),
    Boundary(Boundary),
    AxisValue(AxisValue)
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct AxisValue{
    value: AxisValueVariant
}

#[derive(Serialize, Deserialize, Debug, Clone)]
enum AxisValueVariant{
    Str(String),
    Num(f64)
}

#[derive(Serialize, Deserialize, Debug, Clone)]
enum Boundary{
    Start,
    End,
    Origin,
    Stack,
    Shape
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
struct  Fill {
    target: FillVariant,
    above: Option<Rgb>,
    below: Option<Rgb>
}


#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ChartOptions{
    #[serde(skip_serializing_if = "Option::is_none")]
    scales: Option<ScalingConfig>,
    aspect_ratio: Option<u8>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ScalingConfig{
    #[serde(skip_serializing_if = "Option::is_none")]
    x: Option<ScaleConfig>,

    #[serde(skip_serializing_if = "Option::is_none")]
    y: Option<ScaleConfig>
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ScaleConfig{
    #[serde(skip_serializing_if = "Option::is_none")]
    r#type: Option<ScaleType>,
    #[serde(skip_serializing_if = "Option::is_none")]
    align_to_pixels: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    reverse: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    max: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    min: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    title: Option<AxisTitle>
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub enum ScaleType{
    Linear,
    Logarithmic,
    Category,
    Time,
    TimeSeries,
    RadialLinear
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct AxisTitle{
    display: bool,
    text: String,
    align: Alignment,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "lowercase")]
pub enum Alignment{
    Start,
    Center,
    End
}