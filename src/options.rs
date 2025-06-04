use serde::{Deserialize, Serialize};
use uuid::Uuid;
use crate::common::{Padding, Rgb, Size};
use crate::options::ChartData::Vector2D;
use crate::render::Chart;

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ChartConfig<X,Y> {


    data: ChartDataSection<X,Y>,

    #[serde(skip_serializing_if = "Option::is_none")]
    options: Option<ChartOptions>
}

impl<X, Y> ChartConfig<X, Y> where ChartConfig<X, Y>:Serialize {

    pub fn new(options: Option<ChartOptions>) -> Self {
        Self {
            data: ChartDataSection {
                labels: None,
                datasets: vec![],
            },
            options
        }
    }

    pub fn title_str(mut self, text: String) -> Self {
        let mut options = self.options.unwrap_or(ChartOptions::default());
        let mut plugins = options.plugins.unwrap_or(vec![]);
        plugins.push(
            Plugin::Title(Title{
                display: true,
                full_size: false,
                text: vec![text],
                padding: None,
                position: None,
            })
        );
        options.plugins = Some(plugins);
        self.options = Some(options);
        self
    }


    pub fn add_series<T: Into<ChartData<X,Y>>>(mut self, r#type: ChartType, title:String, data: T)->Self{
        let mut chart_data = self.data;
        let mut datasets = chart_data.datasets;
        datasets.push(Dataset{
            r#type,
            label: title,
            data: data.into(),
            fill: None,
            border_color: None,
            background_color: None,
        });
        chart_data.datasets = datasets;
        self.data = chart_data;
        self
    }

    pub fn enable_legend(mut self) ->  Self {
        todo!()
    }
    
    pub fn build(self, width: Size, height: Size) -> Chart<X,Y>{
        Chart::new(Uuid::new_v4().to_string(), width, height, self)
    }
    
}


impl<X,Y> Default for ChartConfig<X,Y>{
    fn default() -> Self {
        ChartConfig{
            data: ChartDataSection {
                labels: None,
                datasets: vec![],
            },
            options: None,
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
pub struct ChartDataSection<X,Y>{

    #[serde(skip_serializing_if = "Option::is_none")]
    labels: Option<Vec<String>>,

    datasets: Vec<Dataset<X,Y>>
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Dataset<X,Y>{

    r#type: ChartType,

    label: String,

    data: ChartData<X,Y>,

    #[serde(skip_serializing_if = "Option::is_none")]
    fill: Option<Fill>,

    #[serde(skip_serializing_if = "Option::is_none")]
    border_color: Option<Rgb>,

    #[serde(skip_serializing_if = "Option::is_none")]
    background_color: Option<Rgb>
}

impl<X,Y> From<(X,Y)> for DataElement<X,Y>{
    fn from(value: (X, Y)) -> Self {
        DataElement{
            x: value.0,
            y: value.1,
            r: None,
        }
    }
}

impl<X,Y> From<Vec<(X,Y)>> for ChartData<X,Y>{
    fn from(value: Vec<(X, Y)>) -> Self {
       Vector2D(value.into_iter().map(|x| x.into()).collect())
    }
}

impl<const N: usize,X,Y> From<[(X,Y);N]> for ChartData<X,Y>{
    fn from(value: [(X,Y);N]) -> Self {
        Vector2D(value.into_iter().map(|x| x.into()).collect())
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(untagged)]
pub enum ChartData<X,Y>{
    Vector2D(Vec<DataElement<X,Y>>) 
}



#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct DataElement<X,Y>{
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
    plugins: Option<Vec<Plugin>>,
}

impl Default for ChartOptions{
    fn default() -> Self {
        ChartOptions{
            scales: None,
            aspect_ratio: None,
            plugins: None,
        }
    }
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

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(untagged)]
pub enum Plugin{
    Title(Title)
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "lowercase")]
enum Position{
    Top,
    Left,
    Bottom,
    Right
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Title{
    ///Is the title shown?
    display: bool,

    ///Marks that this box should take the full width/height of the canvas. If false, the box is sized and placed above/beside the chart area.
    full_size: bool,

    ///Title text to display. If specified as an array, text is rendered on multiple lines.
    text: Vec<String>,

    ///Padding to apply around the title. Only top and bottom are implemented.
    #[serde(skip_serializing_if = "Option::is_none")]
    padding: Option<Padding>,

    ///position of the title
    #[serde(skip_serializing_if = "Option::is_none")]
    position: Option<Position>

}



