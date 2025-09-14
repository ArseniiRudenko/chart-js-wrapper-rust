use std::cmp::PartialEq;
use crate::common::{Padding, Rgb, Size};
use crate::render::Chart;
use ndarray::{Array1, Array2};
use ndarray_linalg::error::LinalgError;
use ndarray_linalg::LeastSquaresSvd;
use ndarray_linalg::{Lapack, Scalar};
use serde::Serialize;
use serde::Deserialize;
use uuid::Uuid;
use crate::data::ChartData;
use crate::serde::{ValueSerializeWrapper, WithTypeAndSerializer};

const DISPLAY_FN: &'static str = "
                        function(context){
                            context = context[0];
                            let ttp = context.raw.tooltip || '';
                            if(ttp) return ttp;
                        }";

#[derive(Debug, Clone)]
pub struct ChartConfig<X:WithTypeAndSerializer+Serialize,Y:WithTypeAndSerializer+Serialize>
{
    pub data: ChartDataSection<X,Y>,
    pub options: ChartOptions<X,Y>
}

impl<X, Y> ChartConfig<X, Y>
where X:WithTypeAndSerializer+Serialize, Y:WithTypeAndSerializer+Serialize{

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


    pub fn with_elements(mut self, elements: ElementsConfig) -> Self {
        self.options.elements = Some(elements);
        self
    }

    pub fn with_aspect_ratio(mut self, ratio: f32) -> Self {
        self.options.aspect_ratio = Some(ratio);
        self
    }

    pub fn add_series_direct(mut self, series:Dataset<X,Y>) -> Self{
        self.data.datasets.push(series);
        self
    }

    pub fn add_series_with_config<T: Into<ChartData<X,Y>>>(mut self, r#type: ChartType, title:String, config:ElementsConfig, data: T)->Self{
        self.data.datasets.push(Dataset {
            r#type,
            label: title,
            data: data.into(),
            elements: Some(config)
        });
        self
    }

    pub fn add_series<T: Into<ChartData<X,Y>>>(mut self, r#type: ChartType, title:String, data: T)->Self{
        self.data.datasets.push(Dataset {
            r#type,
            label: title,
            data: data.into(),
            elements: None
        });
        self
    }

    pub fn enable_legend(mut self) -> Self{
        let legend = self.options.plugins.legend.get_or_insert_default();
        legend.display = true;
        self
    }

    pub fn with_title(mut self, title:Title) -> Self{
        self.options.plugins.title = Some(title);
        self
    }

    pub fn with_legend(mut self, legend:Legend) -> Self{
        self.options.plugins.legend = Some(legend);
        self
    }

    pub fn with_tooltip(mut self, tooltip:Tooltip) -> Self{
        self.options.plugins.tooltip = Some(tooltip);
        self
    }

    pub fn with_subtitle(mut self, subtitle:Title) -> Self{
        self.options.plugins.subtitle = Some(subtitle);
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

impl<X: WithTypeAndSerializer+Serialize, Y: WithTypeAndSerializer+Serialize> Default for ChartConfig<X,Y> {
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
                elements: None,
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
    elements: Option<ElementsConfig>
}

#[derive(Serialize, Debug, Clone,Default)]
#[serde(rename_all = "camelCase")]
pub struct ElementsConfig{
    #[serde(skip_serializing_if = "Option::is_none")]
    line:  Option<LineConfig>,
    #[serde(skip_serializing_if = "Option::is_none")]
    point: Option<PointConfig>

}

impl ElementsConfig {

    pub fn with_line_config(mut self, conf: LineConfig) -> Self{
        self.line = Some(conf);
        self
    }

    pub fn with_point_config(mut self, conf: PointConfig) -> Self{
        self.point = Some(conf);
        self
    }

}

#[derive(Serialize, Debug, Clone)]
#[serde(rename_all = "lowercase")]
pub enum CubicInterpolationMode{
    Default,
    Monotone
}

#[derive(Serialize, Debug, Clone,Default)]
pub struct LineConfig {
    ///how much bezier rounding to use, default is 0 - no bezier
    #[serde(skip_serializing_if = "Option::is_none")]
    tension: Option<f32>,

    #[serde(skip_serializing_if = "Option::is_none")]
    cubic_interpolation_mode: Option<CubicInterpolationMode>,

    #[serde(skip_serializing_if = "Option::is_none")]
    fill: Option<Fill>,

    #[serde(skip_serializing_if = "Option::is_none")]
    border_color: Option<Rgb>,

    #[serde(skip_serializing_if = "Option::is_none")]
    background_color: Option<Rgb>,

    stepped: bool

}

impl LineConfig {

    pub fn with_tension(mut self, tension: f32) -> Self{
        self.tension = Some(tension);
        self.stepped = false;
        self
    }

    pub fn with_stepped(mut self, stepped: bool) -> Self{
        self.stepped = stepped;
        self.tension = None;
        self
    }

    pub fn with_cubic_interpolation_mode(mut self, mode: CubicInterpolationMode) -> Self{
        self.cubic_interpolation_mode = Some(mode);
        self
    }

    pub fn with_fill(mut self, fill: Fill) -> Self{
        self.fill = Some(fill);
        self
    }

    pub fn with_border_color(mut self, color: Rgb) -> Self{
        self.border_color = Some(color);
        self
    }

    pub fn with_background_color(mut self, color: Rgb) -> Self{
        self.background_color = Some(color);
        self
    }
}


#[derive(Serialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct PointConfig{

    #[serde(skip_serializing_if = "Option::is_none")]
    border_color: Option<Rgb>,

    #[serde(skip_serializing_if = "Option::is_none")]
    background_color: Option<Rgb>,

    point_style: PointStyle,

    ///point rotation in degrees
    rotation: u16,

    radius: u16,

    border_width: u16,

    hover_radius: u16,

    hit_radius: u16,

    hover_border_width: u16

}

impl Default for PointConfig {
    fn default() -> Self {
        PointConfig{
            radius: 3,
            point_style: PointStyle::Circle,
            rotation: 0,
            border_color: None,
            background_color: None,
            border_width: 1,
            hover_radius: 4,
            hit_radius: 1,
            hover_border_width: 1
        }
    }
}

impl PointConfig {
    pub fn with_radius(mut self, radius: u16) -> Self{
        self.radius = radius;
        self
    }

    pub fn with_point_style(mut self, style: PointStyle) -> Self{
        self.point_style = style;
        self
    }
    pub fn with_rotation(mut self, rotation: u16) -> Self{
        self.rotation = rotation;
        self
    }
    pub fn with_border_color(mut self, color: Rgb) -> Self{
        self.border_color = Some(color);
        self
    }
    pub fn with_background_color(mut self, color: Rgb) -> Self{
        self.background_color = Some(color);
        self
    }

    pub fn with_border_width(mut self, width: u16) -> Self{
        self.border_width = width;
        self
    }
    pub fn with_hover_radius(mut self, radius: u16) -> Self{
        self.hover_radius = radius;
        self
    }
    pub fn with_hit_radius(mut self, radius: u16) -> Self{
        self.hit_radius = radius;
        self
    }

    pub fn with_hover_border_width(mut self, width: u16) -> Self{
        self.hover_border_width = width;
        self
    }
}


#[derive(Serialize, Debug, Clone)]
pub enum PointStyle{
    Circle,
    Cross,
    CrossRot,
    Dash,
    Line,
    Rect,
    RectRounded,
    RectRot,
    Star,
    Triangle
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(untagged)]
pub enum FillVariant{
    AbsIndex(u8),
    RelativeIndex(String),
    Boundary(Boundary),
    AxisValue(AxisValue)
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct AxisValue{
    value: AxisValueVariant
}

impl AxisValue{
    pub fn new(value: AxisValueVariant) -> Self{
        Self{
            value
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum AxisValueVariant{
    Str(String),
    Num(f64)
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum Boundary{
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


impl Fill {

    pub fn with_target(mut self, target: FillVariant) -> Self{
        self.target = target;
        self
    }
    pub fn with_above(mut self, color: Rgb) -> Self{
        self.above = Some(color);
        self
    }

    pub fn with_below(mut self, color: Rgb) -> Self{
        self.below = Some(color);
        self
    }

}


#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "lowercase")]
enum AxisName{
    X,
    Y
}

#[derive(Debug, Clone)]
pub struct ChartOptions<X,Y> where X:WithTypeAndSerializer, Y:WithTypeAndSerializer{
    scales: Option<ScalingConfig<X,Y>>,
    aspect_ratio: Option<f32>,
    elements: Option<ElementsConfig>,
    plugins: Plugins,
}

impl<X,Y> Default for ChartOptions<X,Y> where X:WithTypeAndSerializer, Y:WithTypeAndSerializer{
    fn default() -> Self {
        ChartOptions{
            scales: None,
            aspect_ratio: None,
            plugins: Plugins::default(),
            elements: None
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
        AxisTitle::new(value.to_string())
    }
}

impl From<String> for AxisTitle{
    fn from(value: String) -> Self {
        AxisTitle::new(value)
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
pub struct Plugins{
    title: Option<Title>,
    subtitle: Option<Title>,
    legend: Option<Legend>,
    tooltip: Option<Tooltip>
}

impl Default for Plugins{
    fn default() -> Self {
        Plugins{
            title: None,
            subtitle: None,
            legend: None,
            tooltip: Some(Tooltip::default()),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Legend{
    display: bool,

    position: Position,

    align: Alignment,

    full_size: bool,
}


impl Default for Legend{
    fn default() -> Self {
        Legend{
            display: true,
            position: Position::Bottom,
            align: Alignment::Center,
            full_size: false
        }
    }
}

impl Legend {
    pub fn new_position(position: Position) -> Self {
        Self {
            display: true,
            position,
            align: Alignment::Center,
            full_size: false
        }
    }

    pub fn  with_align(mut self, align: Alignment) -> Self {
        self.align = align;
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
        self.position = position;
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

impl From<String> for Title {
    fn from(value: String) -> Self {
        Title::new(value)
    }
}

impl From<&str> for Title {
    fn from(value: &str) -> Self {
        Title::from(value.to_string())
    }
}

impl Title{
    pub fn new(text: String) -> Self {
        Self{
            display: true,
            full_size: false,
            text: vec![text],
            padding: None,
            position: None
        }
    }

    pub fn from_str(text: &str) -> Self {
        Self{
            display: true,
            full_size: false,
            text: vec![text.to_string()],
            padding: None,
            position: None
        }
    }

    pub fn from_array(text: Vec<String>) -> Self {
        Self{
            display: true,
            full_size: false,
            text,
            padding: None,
            position: None
        }
    }

    pub fn with_display(mut self, display: bool) -> Self {
        self.display = display;
        self
    }
    pub fn with_full_size(mut self, full_size: bool) -> Self {
        self.full_size = full_size;
        self
    }
    pub fn with_text(mut self, text: Vec<String>) -> Self {
        self.text = text;
        self
    }
    pub fn with_padding(mut self, padding: Padding) -> Self {
        self.padding = Some(padding);
        self
    }
    pub fn with_position(mut self, position: Position) -> Self {
        self.position = Some(position);
        self
    }
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
            callbacks: Some(TooltipCallbacks::default())
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

#[derive(Debug, Clone)]
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

impl Default for TooltipCallbacks{
    fn default() -> Self {
        TooltipCallbacks{
            before_title: None,
            title: Some(JsExpr(DISPLAY_FN)),
            after_title: None,
            before_body: None,
            before_label: None,
            label: None,
            label_color: None,
            label_text_color: None,
            after_label: None,
            after_body: None,
            before_footer: None,
            footer: None,
            after_footer: None,
        }
    }
}


#[derive(Debug, Clone)]
pub struct JsExpr(pub &'static str);

impl std::fmt::Display for JsExpr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // Print raw JS, no quotes
        f.write_str(&self.0)
    }
}

