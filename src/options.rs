use crate::common::{Padding, Rgb, Size};
use crate::options::ChartData::Vector2D;
use crate::render::Chart;
use ndarray::{Array1, Array2};
use ndarray_linalg::error::LinalgError;
use ndarray_linalg::LeastSquaresSvd;
use ndarray_linalg::{Lapack, Scalar};
use num_traits::One;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use crate::ChartData::VectorWithRadius;

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
        let mut options = self.options.unwrap_or_default();
        let mut plugins = options.plugins.unwrap_or_default();
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

    pub fn enable_legend(mut self) -> Self{
        
        let mut opts = self.options.unwrap_or_default();
        let mut plugins = opts.plugins.unwrap_or_default();
        plugins.push(Plugin::Legend(Legend{
            display: true,
            full_size: false,
            position: None,
            align: None
        }));
        opts.plugins = Some(plugins);
        self.options = Some(opts);
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

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(untagged)]
pub enum ChartData<X,Y>{
    Vector2D(Vec<(X,Y)>),
    VectorWithRadius(Vec<DataElement<X,Y>>)
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
    Title(Title),
    Legend(Legend)
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