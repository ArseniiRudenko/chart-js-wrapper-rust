use serde::ser::SerializeSeq;
use serde::{Deserialize, Serialize};
use crate::data::ChartData::{Vector2D, VectorWithRadius, VectorWithText};
use crate::serde::{ValueSerializeWrapper, WithTypeAndSerializer};

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