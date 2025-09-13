use std::cmp::PartialEq;
use std::time::{Instant, SystemTime};
use crate::common::{Padding, Rgb, Size};
use crate::options::ChartData::Vector2D;
use crate::render::Chart;
use crate::ChartData::{VectorWithRadius, VectorWithText};
use ndarray::{Array1, Array2};
use ndarray_linalg::error::LinalgError;
use ndarray_linalg::LeastSquaresSvd;
use ndarray_linalg::{Lapack, Scalar};
use serde::{Deserialize, Serialize, Serializer};
use serde::ser::SerializeSeq;
use uuid::Uuid;
use crate::serde::{ValueSerializeWrapper, WithTypeAndSerializer};

const DISPLAY_FN: &'static str = "
                        function(context){
                            context = context[0];
                            let ttp = context.raw.tooltip || '';
                            if(ttp) return ttp;
                        }";

#[derive(Debug, Clone)]
pub struct ChartConfig<X,Y>
where X:WithTypeAndSerializer, Y:WithTypeAndSerializer {
    pub data: ChartDataSection<X,Y>,
    pub options: ChartOptions<X,Y>
}

impl<X, Y> ChartConfig<X, Y> where X:WithTypeAndSerializer, Y:WithTypeAndSerializer{

    pub fn new(options: ChartOptions<X,Y>) -> Self {
        Self {
            data: ChartDataSection::default(),
            options
        }
    }

    pub fn set_x_axis(mut self, conf: ScaleConfig<X>) -> Self {
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

    pub fn set_y_axis(mut self, conf: ScaleConfig<Y>) -> Self {
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

    pub fn with_aspect_ratio(mut self, ratio: f32) -> Self {
        self.options.aspect_ratio = Some(ratio);
        self
    }

    pub fn add_series<T: Into<ChartData<X,Y>>>(mut self, r#type: ChartType, title:String, data: T)->Self{
        let data = data.into();
        if let VectorWithText(d) = data {
            let ttp = self.options.plugins.tooltip.get_or_insert_default();
            let callbacks = ttp.callbacks.get_or_insert_default();
            callbacks.title = Some(JsExpr(DISPLAY_FN));
            self.data.datasets.push(Dataset {
                r#type,
                label: title,
                data: VectorWithText(d),
                fill: None,
                border_color: None,
                background_color: None,
            });
        }else {
            self.data.datasets.push(Dataset {
                r#type,
                label: title,
                data,
                fill: None,
                border_color: None,
                background_color: None,
            });
        }
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



impl<X> ChartConfig<X, X> where  X:WithTypeAndSerializer + Scalar + Lapack + Clone + Into<f64> {
    
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


impl<X,Y> Default  for ChartDataSection<X,Y> where X:WithTypeAndSerializer, Y:WithTypeAndSerializer{
    fn default() -> Self {
        ChartDataSection {
            datasets: vec![],
        }
    }
}

impl<X: WithTypeAndSerializer, Y: WithTypeAndSerializer> Default for ChartConfig<X,Y> where ChartDataSection<X,Y>:Serialize{
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

#[derive(Serialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ChartDataSection<X:WithTypeAndSerializer,Y:WithTypeAndSerializer>{
    datasets: Vec<Dataset<X,Y>>
}

#[derive(Serialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Dataset<X:WithTypeAndSerializer,Y:WithTypeAndSerializer>{

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


impl<X,Y> From<DataPointWithRadius<X,Y>> for (X, Y){
    fn from(value: DataPointWithRadius<X,Y>) -> Self {
        (value.x, value.y)
    }
}

impl<X,Y> From<DataPoint<X,Y>> for (X,Y){
    fn from(value: DataPoint<X,Y>) -> Self {
        (value.x, value.y)
    }
}

impl<X,Y> From<DataPointWithTooltip<X,Y>> for (X, Y){
    fn from(value: DataPointWithTooltip<X,Y>) -> Self {
        (value.x, value.y)
    }
}

impl<X,Y> From<DataPointWithTooltip<X,Y>> for (X, Y, String){
    fn from(value: DataPointWithTooltip<X,Y>) -> Self {
        (value.x, value.y, value.tooltip)
    }
}


impl<X,Y> From<(X, Y, String)> for DataPointWithTooltip<X,Y>{
    fn from(value: (X, Y, String)) -> Self {
        DataPointWithTooltip{
            x: value.0,
            y: value.1,
            tooltip: value.2
        }
    }
}

impl<X,Y> From<(X, Y, &str)> for DataPointWithTooltip<X,Y>{
    fn from(value: (X, Y, &str)) -> Self {
        DataPointWithTooltip{
            x: value.0,
            y: value.1,
            tooltip: value.2.to_string()
        }
    }
}

impl<X:WithTypeAndSerializer,Y:WithTypeAndSerializer> From<ChartData<X,Y>> for Vec<(X,Y)>{
    fn from(value: ChartData<X, Y>) -> Self {
        match value {
            Vector2D(v) => v.into_iter().map(|v| (v.0.0,v.1.0)).collect(),
            VectorWithRadius(val)=> val.into_iter().map(|v| (v.x.0,v.y.0)).collect(),
            VectorWithText(val)=> val.into_iter().map(|v|  (v.x.0,v.y.0)).collect()
        }
    }
}

impl<X,Y> From<Vec<(X,Y,String)>> for ChartData<X,Y> where X:WithTypeAndSerializer, Y:WithTypeAndSerializer{
    fn from(value: Vec<(X, Y, String)>) -> Self {
        VectorWithText(value.into_iter().map(|v| (v.0.into(),v.1.into(),v.2).into()).collect())
    }
}

impl<const N: usize,X,Y> From<[(X,Y,String);N]> for ChartData<X,Y> where X:WithTypeAndSerializer, Y:WithTypeAndSerializer{
    fn from(value: [(X,Y,String);N]) -> Self {
        VectorWithText(value.into_iter().map(|v| (v.0.into(),v.1.into(),v.2).into()).collect())
    }
}

impl<X,Y> From<Vec<(X,Y,&str)>> for ChartData<X,Y> where X:WithTypeAndSerializer, Y:WithTypeAndSerializer{
    fn from(value: Vec<(X, Y,&str)>) -> Self {
        VectorWithText(value.into_iter().map(|v|(v.0.into(),v.1.into(),v.2).into()).collect())
    }
}

impl<const N: usize,X,Y> From<[(X,Y,&str);N]> for ChartData<X,Y> where X:WithTypeAndSerializer, Y:WithTypeAndSerializer{
    fn from(value: [(X,Y,&str);N]) -> Self {
        VectorWithText(value.into_iter().map(|v|(v.0.into(),v.1.into(),v.2).into()).collect())
    }
}



impl<X,Y> From<Vec<(X,Y)>> for ChartData<X,Y> where X:WithTypeAndSerializer, Y:WithTypeAndSerializer{
    fn from(value: Vec<(X, Y)>) -> Self {
       Vector2D(value.into_iter().map(|v|(v.0.into(),v.1.into())).collect())
    }
}

impl<const N: usize,X,Y> From<[(X,Y);N]> for ChartData<X,Y> where X:WithTypeAndSerializer, Y:WithTypeAndSerializer {
    fn from(value: [(X,Y);N]) -> Self {
        Vector2D(value.into_iter().map(|v|(v.0.into(),v.1.into())).collect())
    }
}

#[derive(Debug, Clone)]
pub enum ChartData<X,Y> where X:WithTypeAndSerializer, Y:WithTypeAndSerializer{
    Vector2D(Vec<(ValueSerializeWrapper<X>,ValueSerializeWrapper<Y>)>),
    VectorWithRadius(Vec<DataPointWithRadius<ValueSerializeWrapper<X>,ValueSerializeWrapper<Y>>>),
    VectorWithText(Vec<DataPointWithTooltip<ValueSerializeWrapper<X>,ValueSerializeWrapper<Y>>>)
}


impl<X,Y> Serialize for ChartData<X,Y> where X:WithTypeAndSerializer, Y:WithTypeAndSerializer{
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
            VectorWithRadius(v) => v.serialize(serializer),
            VectorWithText(v) => v.serialize(serializer)
        }
    }
}


#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct DataPoint<X,Y>{
    x: X,
    y: Y
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct DataPointWithTooltip<X,Y>{
    x: X,
    y: Y,
    tooltip: String
}


#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct DataPointWithRadius<X,Y>{
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

#[derive(Debug, Clone)]
pub struct ChartOptions<X,Y> where X:WithTypeAndSerializer, Y:WithTypeAndSerializer{
    pub(crate) scales: Option<ScalingConfig<X,Y>>,
    pub(crate) aspect_ratio: Option<f32>,
    pub plugins: Plugins,
}

impl<X,Y> Default for ChartOptions<X,Y> where X:WithTypeAndSerializer, Y:WithTypeAndSerializer{
    fn default() -> Self {
        ChartOptions{
            scales: None,
            aspect_ratio: None,
            plugins: Plugins::default(),
        }
    }
}

#[derive(Serialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ScalingConfig<X,Y> where X:WithTypeAndSerializer, Y:WithTypeAndSerializer{
    #[serde(skip_serializing_if = "Option::is_none")]
    x: Option<ScaleConfig<X>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    y: Option<ScaleConfig<Y>>
}

#[derive(Serialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ScaleConfig<T> where T:WithTypeAndSerializer{

    #[serde(skip_serializing_if = "Option::is_none")]
    r#type: Option<ScaleType>,

    #[serde(skip_serializing_if = "Option::is_none")]
    labels: Option<Vec<ValueSerializeWrapper<T>>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    align_to_pixels: Option<bool>,

    reverse: bool,

    #[serde(skip_serializing_if = "Option::is_none")]
    max: Option<ValueSerializeWrapper<T>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    min: Option<ValueSerializeWrapper<T>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    title: Option<AxisTitle>
}


impl<T> Default for ScaleConfig<T> where T:WithTypeAndSerializer{
    fn default() -> Self {
        ScaleConfig{
            r#type: None,
            labels: None,
            align_to_pixels: None,
            reverse: false,
            max: None,
            min: None,
            title: None
        }
    }
}

impl<T> ScaleConfig<T> where T:WithTypeAndSerializer{

    pub fn new_category(reverse:bool,values: Vec<T>) -> Self {
        Self {
            r#type: Some(ScaleType::Category),
            labels: Some(values.into_iter().map(|v| v.into()).collect()),
            reverse,
            ..ScaleConfig::<T>::default()
        }
    }

    pub fn with_type(mut self, r#type: ScaleType) -> Self {
        self.r#type = Some(r#type);
        self
    }

    pub fn with_align_to_pixels(mut self, align_to_pixels: bool) -> Self {
        self.align_to_pixels = Some(align_to_pixels);
        self
    }

    pub fn with_max(mut self, max: T) -> Self {
        self.max = Some(max.into());
        self
    }

    pub fn with_min(mut self, min: T) -> Self {
        self.min = Some(min.into());
        self
    }

    pub fn with_str_title(mut self, title: &str) -> Self {
        self.title = Some(title.into());
        self
    }

    pub fn with_title(mut self, title: AxisTitle) -> Self {
        self.title = Some(title);
        self
    }

    pub fn with_labels(mut self, labels: Vec<T>) -> Self {
        self.labels = Some(labels.into_iter().map(|v| v.into()).collect());
        self
    }

    pub fn with_reverse(mut self, reverse: bool) -> Self {
        self.reverse = reverse;
        self
    }

}

#[derive(Serialize, Deserialize, Debug, Clone,PartialEq)]
#[serde(rename_all = "lowercase")]
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


impl From<&str> for AxisTitle {
    fn from(value: &str) -> Self {
        Self{
            display: true,
            text: value.to_string(),
            align: Alignment::Center
        }
    }
}

impl AxisTitle{
    pub fn new(text: String) -> Self {
        Self{
            display: true,
            text,
            align: Alignment::Center
        }
    }

    pub fn with_display(mut self, display: bool) -> Self {
        self.display = display;
        self
    }

    pub fn with_text(mut self, text: String) -> Self {
        self.text = text;
        self
    }

    pub fn with_align(mut self, align: Alignment) -> Self {
        self.align = align;
        self
    }

}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "lowercase")]
pub enum Alignment{
    Start,
    Center,
    End
}

#[derive(Debug, Clone)]
#[derive(Default)]
pub struct Plugins{
    pub title: Option<Title>,
    pub subtitle: Option<Title>,
    pub legend: Option<Legend>,
    pub tooltip: Option<Tooltip>
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


impl Default for Legend{
    fn default() -> Self {
        Legend{
            display: true,
            position: None,
            align: None,
            full_size: false
        }
    }
}

impl Legend {
    pub fn new_position(position: Position) -> Self {
        Self {
            display: true,
            position: Some(position),
            align: None,
            full_size: false
        }
    }

    pub fn  with_align(mut self, align: Alignment) -> Self {
        self.align = Some(align);
        self
    }

    pub fn  with_full_size(mut self, full_size: bool) -> Self {
        self.full_size = full_size;
        self
    }

    pub fn  with_display(mut self, display: bool) -> Self {
        self.display = display;
        self
    }

    pub fn  with_position(mut self, position: Position) -> Self {
        self.position = Some(position);
        self
    }

}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "lowercase")]
pub enum Position{
    Top,
    Left,
    Bottom,
    Right
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "lowercase")]
pub enum Align{
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


#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "lowercase")]
pub enum TooltipMode{
    Average,
    Nearest
}


impl Default for Tooltip {
    fn default() -> Self {
        Tooltip{
            enabled: true,
            mode: None,
            background_color: None,
            title_color: None,
            callbacks: None
        }
    }
}

#[derive(Debug, Clone)]
pub struct Tooltip{
    pub enabled: bool,
    pub mode: Option<TooltipMode>,
    pub background_color: Option<Rgb>,
    pub title_color: Option<Rgb>,
    pub callbacks: Option<TooltipCallbacks>
}

#[derive(Debug, Clone,Default)]
pub struct TooltipCallbacks{
    pub before_title: Option<JsExpr>,
    pub title: Option<JsExpr>,
    pub after_title: Option<JsExpr>,
    pub before_body: Option<JsExpr>,
    pub before_label: Option<JsExpr>,
    pub label: Option<JsExpr>,
    pub label_color: Option<JsExpr>,
    pub label_text_color: Option<JsExpr>,
    pub after_label: Option<JsExpr>,
    pub after_body: Option<JsExpr>,
    pub before_footer: Option<JsExpr>,
    pub footer: Option<JsExpr>,
    pub after_footer: Option<JsExpr>,
}


#[derive(Debug, Clone)]
pub struct JsExpr(pub &'static str);

impl std::fmt::Display for JsExpr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // Print raw JS, no quotes
        f.write_str(&self.0)
    }
}

