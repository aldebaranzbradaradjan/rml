use macroquad::color::Color;
use crate::decompose_color_string;

pub struct Property {
    pub value: AbstractValue,
}

impl std::fmt::Debug for Property {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Property")
            .field("value", &self.value)
            //.field("callbacks", &format!("callbacks ommited, count : {}", &self.callbacks.len()))
            .finish()
    }
}

impl Property {
    pub fn new(initial_value: AbstractValue) -> Self {
        Self {
            value: initial_value,
        }
    }

    pub fn set(&mut self, new_value: AbstractValue) {
        self.value = new_value;
    }

    pub fn get(&self) -> AbstractValue {
        self.value.clone()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum AbstractValue {
    Bool(bool),
    String(String),
    Number(f32),
    Array(Vec<AbstractValue>),
    Null,
}

impl From<u32> for AbstractValue {
    fn from(val: u32) -> Self {
        AbstractValue::Number(val as f32)
    }
}

impl From<f32> for AbstractValue {
    fn from(val: f32) -> Self {
        AbstractValue::Number(val)
    }
}
    
impl From<bool> for AbstractValue {
    fn from(val: bool) -> Self {
        AbstractValue::Bool(val)
    }
}
    
impl From<&str> for AbstractValue {
    fn from(val: &str) -> Self {
        AbstractValue::String(val.to_string())
    }
}
    
impl From<String> for AbstractValue {
    fn from(val: String) -> Self {
        AbstractValue::String(val)
    }
}
    
impl From<Vec<AbstractValue>> for AbstractValue {
    fn from(val: Vec<AbstractValue>) -> Self {
        AbstractValue::Array(val)
    }
}
    
impl From<()> for AbstractValue {
    fn from(_: ()) -> Self {
        AbstractValue::Null
    }
}

impl AbstractValue {
    pub fn to_string(&self) -> String {
        match self {
            AbstractValue::Bool(b) => b.to_string(),
            AbstractValue::String(s) => s.clone(),
            AbstractValue::Number(n) => n.to_string(),
            AbstractValue::Array(arr) => format!("{:?}", arr),
            AbstractValue::Null => "null".to_string(),
        }
    }

    pub fn to_number(&self) -> Option<f32> {
        match self {
            AbstractValue::Number(n) => Some(*n),
            _ => None,
        }
    }

    pub fn to_bool(&self) -> Option<bool> {
        match self {
            AbstractValue::Bool(b) => Some(*b),
            _ => None,
        }
    }

    pub fn to_color(&self) -> Option<Color> {
        match self {
            AbstractValue::String(s) => {
                let color_tuple = decompose_color_string(s);
                Some(Color::new(color_tuple.0, color_tuple.1, color_tuple.2, color_tuple.3))

                // let s = s.trim_start_matches("rgba(").trim_end_matches(")");
                // let parts: Vec<&str> = s.split(',').map(|s| s.trim()).collect();

                // //print!("parts: {:?}", parts);
                // if parts.len() == 4 {
                //     let r = parts[0].parse::<f32>().ok()?;
                //     let g = parts[1].parse::<f32>().ok()?;
                //     let b = parts[2].parse::<f32>().ok()?;
                //     let a = parts[3].parse::<f32>().ok()?;
                //     Some(Color::new(r, g, b, a))
                // } else {
                //     None
                // }
            }
            _ => None,
        }
    }
}