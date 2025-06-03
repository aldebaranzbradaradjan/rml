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
    Color(Color),
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

impl From<Color> for AbstractValue {
    fn from(val: Color) -> Self {
        AbstractValue::Color(val)
    }
}

impl std::fmt::Display for AbstractValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.to_string())
    }
}

impl AbstractValue {
    pub fn to_string(&self) -> String {
        match self {
            AbstractValue::Bool(b) => b.to_string(),
            AbstractValue::String(s) => s.clone(),
            AbstractValue::Number(n) => n.to_string(),
            AbstractValue::Color(c) => format!("rgba({}, {}, {}, {})", c.r, c.g, c.b, c.a),
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
            AbstractValue::Color(c) => Some(*c),
            AbstractValue::String(s) => {
                Some( decompose_color_string(s) )
            }
            _ => None,
        }
    }
}