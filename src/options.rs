use std::cmp::PartialEq;
use crate::common::{Padding, Rgb, Size};
use crate::options::ChartData::Vector2D;
use crate::render::Chart;
use crate::ChartData::VectorWithRadius;
use ndarray::{Array1, Array2};
use ndarray_linalg::error::LinalgError;
use ndarray_linalg::LeastSquaresSvd;
use ndarray_linalg::{Lapack, Scalar};
use serde::{Deserialize, Serialize};
use serde::ser::SerializeSeq;
use uuid::Uuid;

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ChartConfig<X,Y> {

    data: ChartDataSection<X,Y>,
    
    options: ChartOptions
}

impl<X, Y> ChartConfig<X, Y> where ChartConfig<X, Y>:Serialize {

    pub fn new(options: ChartOptions) -> Self {
        Self {
            data: ChartDataSection::default(),
            options
        }
    }

    pub fn set_x_axis(mut self, conf: ScaleConfig) -> Self {
        let mut scales = self.options.scales;
        if scales.is_none() {
            scales = Some(ScalingConfig{
                x: Some(conf),
                y: None
            });
        }else {
            scales.as_mut().unwrap().x = Some(conf);
        }
        self.options.scales = scales;
        self
    }

    pub fn set_y_axis(mut self, conf: ScaleConfig) -> Self {
        let mut scales = self.options.scales;
        if scales.is_none() {
            scales = Some(ScalingConfig{
                x: None,
                y: Some(conf)
            });
        }else {
            scales.as_mut().unwrap().y = Some(conf);
        }
        self.options.scales = scales;
        self
    }


    pub fn title_str(mut self, text: String) -> Self {
        self.options.plugins.title = Some(Title{
                display: true,
                full_size: false,
                text: vec![text],
                padding: None,
                position: None,
        });
        self
    }


    pub fn add_series<T: Into<ChartData<X,Y>>>(mut self, r#type: ChartType, title:String, data: T)->Self{
        self.data.datasets.push(Dataset{
            r#type,
            label: title,
            data: data.into(),
            fill: None,
            border_color: None,
            background_color: None,
        });
        self
    }


    pub fn enable_legend(mut self) -> Self{
        if self.options.plugins.legend.is_none() {
            self.options.plugins.legend = Some(Legend {
                display: true,
                full_size: false,
                position: None,
                align: None
            });
        }else {
            self.options.plugins.legend.as_mut().unwrap().display = true;
        }
        self
    }
    
    pub fn build(self, width: Size, height: Size) -> Chart<X,Y>{
        Chart::new(Uuid::new_v4().to_string(), width, height, self)
    }
    
}



impl<X> ChartConfig<X, X> where ChartConfig<X, X>:Serialize, X:Scalar + Lapack + Clone + Into<f64> {
    
    pub fn add_linear_regression_series<T: Into<ChartData<X,X>>>(self, title: &str, data: T) -> Result<Self, LinalgError> {
        let data:Vec<(X,X)>  = data.into().into();
        let n = data.len();
        let config_with_scatter = self.add_series(ChartType::Scatter, title.to_string(), data.clone());
        
        // Allocate design matrix with shape (n, 2)
        let mut x_matrix = Array2::<X>::zeros((n, 2));
        let mut y_array = Array1::<X>::zeros(n);

        for (i, (x, y)) in data.into_iter().enumerate() {
            x_matrix[[i, 0]] = X::one(); // intercept term
            x_matrix[[i, 1]] = x;
            y_array[i] = y;
        }
        
        let beta = x_matrix.least_squares(&y_array)?.solution;

        let y_pred = x_matrix.dot(&beta);
        let r2 = Self::r_squared(&y_array, &y_pred);
        let mut reg_data = Vec::<(X,X)>::with_capacity(n);
        
        y_pred.into_iter().enumerate().for_each(|(i, x)| {
            reg_data.push((x_matrix[[i,1]],x));
        });
        
        let config_with_both_charts = 
            config_with_scatter.add_series(ChartType::Line, format!("{} regression(R^2 = {:.4})", title, r2), reg_data);
        
        Ok(config_with_both_charts)
    }


    fn r_squared(y_true: &Array1<X>, y_pred: &Array1<X>) -> f64
    {
        let n = y_true.len();
        if n == 0 {
            return 0.0; // or maybe panic/error, depending on your use case
        }
        let n_f = n as f64;

        // Calculate mean once
        let y_mean = y_true.iter().map(|&v| v.into()).sum::<f64>() / n_f;

        // Sum of squared residuals (errors)
        let ss_res = y_true
            .iter()
            .zip(y_pred.iter())
            .map(|(&y, &y_hat)| {
                let y_f = y.into();
                let y_hat_f = y_hat.into();
                let diff = y_f - y_hat_f;
                diff * diff
            })
            .sum::<f64>();

        // Total sum of squares
        let ss_tot = y_true
            .iter()
            .map(|&y| {
                let y_f = y.into();
                let diff = y_f - y_mean;
                diff * diff
            })
            .sum::<f64>();
        
        if ss_tot == 0.0 {
            // All y_true are constant
            return if ss_res == 0.0 {
                1.0  // Perfect fit
            } else {
                0.0  // Model fails to match - treat as no explanatory power
            }
        }
        
        1.0 - ss_res / ss_tot
    }
}


impl<X,Y> Default  for ChartDataSection<X,Y>{
    fn default() -> Self {
        ChartDataSection {
            datasets: vec![],
        }
    }
}

impl<X:WithScaleType, Y:WithScaleType> Default for ChartConfig<X,Y>{
    fn default() -> Self {
        ChartConfig{
            data: ChartDataSection::default(),
            options: ChartOptions{
                scales: Some(ScalingConfig{
                    x:Some(ScaleConfig{
                        r#type: Some(X::scale_type()),
                        ..ScaleConfig::default()
                    }),
                    y:Some(ScaleConfig{
                        r#type: Some(Y::scale_type()),
                        reverse: Y::scale_type() == ScaleType::Category,
                        ..ScaleConfig::default()
                    }),
                }),
                aspect_ratio: None,
                plugins: Plugins::default(),
            },
        }
    }
}


trait WithScaleType{
    fn scale_type()->ScaleType;
}

macro_rules! impl_scale_type {
    ($type:ident for $($t:ty)*) => ($(
        impl WithScaleType for $t {
            fn scale_type() -> ScaleType {
                ScaleType::$type
            }
        }
    )*)
}

impl_scale_type!(Linear for u8 u16 u32 u64 u128 usize i8 i16 i32 i64 i128 isize f32 f64);
impl_scale_type!(Category for &str String);


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


impl<X,Y> From<DataElement<X,Y>> for (X,Y){
    fn from(value: DataElement<X,Y>) -> Self {
        (value.x, value.y)
    }
}


impl<X,Y> From<ChartData<X,Y>> for Vec<(X,Y)>{
    fn from(value: ChartData<X, Y>) -> Self {
        match value {
            Vector2D(v) => v,
            VectorWithRadius(val)=> val.into_iter().map(|v| v.into()).collect()
        }
    }
}

impl<X,Y> From<Vec<(X,Y)>> for ChartData<X,Y>{
    fn from(value: Vec<(X, Y)>) -> Self {
       Vector2D(value)
    }
}

impl<const N: usize,X,Y> From<[(X,Y);N]> for ChartData<X,Y> where X:Clone, Y:Clone {
    fn from(value: [(X,Y);N]) -> Self {
        Vector2D(value.to_vec())
    }
}

#[derive(Deserialize, Debug, Clone)]
#[serde(untagged)]
pub enum ChartData<X,Y>{
    Vector2D(Vec<(X,Y)>),
    VectorWithRadius(Vec<DataElement<X,Y>>)
}


impl<X,Y> Serialize for ChartData<X,Y> where X:Serialize, Y:Serialize{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error> where S: serde::Serializer {
        match self {
            Vector2D(v) => {
                let mut v_ser = serializer.serialize_seq(Some(v.len()))?;
                for (x,y) in v {
                    let  point = DataPoint{x,y};
                    v_ser.serialize_element(&point)?;
                }
                v_ser.end()
            },
            VectorWithRadius(v) => v.serialize(serializer)
        }
    }
}


#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct DataPoint<X,Y>{
    x: X,
    y: Y
}



#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct DataElement<X,Y>{
    x: X,
    y: Y,
    r: u32
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

    #[serde(skip_serializing_if = "Option::is_none")]
    above: Option<Rgb>,

    #[serde(skip_serializing_if = "Option::is_none")]
    below: Option<Rgb>
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "lowercase")]
enum AxisName{
    X,
    Y
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ChartOptions{
    #[serde(skip_serializing_if = "Option::is_none")]
    scales: Option<ScalingConfig>,

    #[serde(skip_serializing_if = "Option::is_none")]
    aspect_ratio: Option<u8>,

    plugins: Plugins,
}

impl Default for ChartOptions{
    fn default() -> Self {
        ChartOptions{
            scales: None,
            aspect_ratio: None,
            plugins: Plugins::default(),
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

#[derive(Serialize, Deserialize, Debug, Clone,Default)]
#[serde(rename_all = "camelCase")]
pub struct ScaleConfig{

    #[serde(skip_serializing_if = "Option::is_none")]
    pub r#type: Option<ScaleType>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub labels: Option<Vec<String>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub align_to_pixels: Option<bool>,

    pub reverse: bool,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub max: Option<f64>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub min: Option<f64>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<AxisTitle>
}

impl ScaleConfig{

    pub fn new_category(reverse:bool,values: Vec<String>) -> Self {
        Self {
            r#type: Some(ScaleType::Category),
            labels: Some(values),
            reverse,
            ..ScaleConfig::default()
        }
    }
}


#[derive(Serialize, Deserialize, Debug, Clone,PartialEq)]
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
#[serde(rename_all = "lowercase")]
#[derive(Default)]
struct Plugins{
    #[serde(skip_serializing_if = "Option::is_none")]
    title: Option<Title>,

    #[serde(skip_serializing_if = "Option::is_none")]
    subtitle: Option<Title>,
    
    #[serde(skip_serializing_if = "Option::is_none")]
    legend: Option<Legend>
    
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Legend{
    display: bool,

    #[serde(skip_serializing_if = "Option::is_none")]
    position: Option<Position>,

    #[serde(skip_serializing_if = "Option::is_none")]
    align: Option<Alignment>,

    pub full_size: bool,
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
#[serde(rename_all = "lowercase")]
enum Align{
    Start,
    Center,
    End
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