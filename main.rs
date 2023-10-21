use std::collections::HashMap;
use std::iter::FromIterator;

#[derive(Clone)]
pub enum WeakType {
    String(String),
    Number(f64),
    Object(Object),
    Array(Array),
    Undefined,
}

use std::fmt::Formatter;
impl std::fmt::Display for WeakType {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), std::fmt::Error> {
        write!(f, "{}", self.coerce_to_string())
    }
}

impl std::ops::Add for &WeakType {
    type Output = WeakType;

    fn add(self, rhs: &WeakType) -> Self::Output {
        match self {
            WeakType::String(self_str) => match rhs {
                WeakType::String(rhs_str) => WeakType::from(self_str.to_owned() + &rhs_str),
                WeakType::Undefined => {
                    WeakType::from(self_str.to_owned() + &rhs.coerce_to_string())
                }
                WeakType::Object(_) => {
                    WeakType::from(self_str.to_owned() + &rhs.coerce_to_string())
                }

                WeakType::Number(rhs_num) => {
                    let self_coerced = self.coerce_to_number();
                    if self_coerced.is_nan() {
                        return WeakType::from(self_str.to_owned() + &rhs_num.to_string());
                    } else {
                        return WeakType::from(self_coerced + rhs_num);
                    }
                }
                WeakType::Array(_) => {
                    WeakType::from(self.coerce_to_string() + &rhs.coerce_to_string())
                }
            },

            WeakType::Number(self_num) => {
                let rhs_coerced = rhs.coerce_to_number();
                if rhs_coerced.is_nan() {
                    return WeakType::from(self.coerce_to_string() + &rhs.coerce_to_string());
                } else {
                    return WeakType::from(self_num + rhs_coerced);
                }
            }
            WeakType::Object(_) => match rhs {
                WeakType::String(rhs_str) => WeakType::from(self.coerce_to_string() + &rhs_str),
                WeakType::Number(_) => WeakType::from(f64::NAN),
                WeakType::Object(_) => {
                    WeakType::from(self.coerce_to_string() + &rhs.coerce_to_string())
                }
                WeakType::Undefined => {
                    WeakType::from(self.coerce_to_string() + &rhs.coerce_to_string())
                }
                WeakType::Array(_) => {
                    WeakType::from(self.coerce_to_string() + &rhs.coerce_to_string())
                }
            },
            WeakType::Undefined => match rhs {
                WeakType::String(rhs_str) => WeakType::from(self.coerce_to_string() + &rhs_str),
                WeakType::Number(_) => WeakType::from(f64::NAN),
                WeakType::Undefined => WeakType::from(f64::NAN),
                WeakType::Object(_) => {
                    WeakType::from(self.coerce_to_string() + &rhs.coerce_to_string())
                }
                WeakType::Array(_) => {
                    WeakType::from(self.coerce_to_string() + &rhs.coerce_to_string())
                }
            },
            WeakType::Array(_) => WeakType::from(self.coerce_to_string() + &rhs.coerce_to_string()),
        }
    }
}

impl std::ops::Index<&str> for WeakType {
    type Output = WeakType;
    fn index(&self, key: &str) -> &<Self as std::ops::Index<&str>>::Output {
        match self {
            WeakType::String(_) => &WeakType::Undefined,
            WeakType::Number(_) => &WeakType::Undefined,
            WeakType::Object(obj) => {
                if obj.0.contains_key(key) {
                    &obj.0[key]
                } else {
                    &WeakType::Undefined
                }
            }
            WeakType::Undefined => self,
            WeakType::Array(_) => &WeakType::Undefined,
        }
    }
}

impl WeakType {
    fn from<'a, T>(value: T) -> WeakType
    where
        T: IntoWeakType,
    {
        value.into()
    }

    fn coerce_to_number(&self) -> f64 {
        match self {
            WeakType::String(str) => {
                let parsing_result = str.parse::<f64>();
                parsing_result.unwrap_or(f64::NAN)
            }
            WeakType::Number(num) => *num,
            WeakType::Object(_) => f64::NAN,
            WeakType::Undefined => f64::NAN,
            WeakType::Array(_) => f64::NAN,
        }
    }

    fn coerce_to_string(&self) -> String {
        match self {
            WeakType::String(str) => str.to_string(),
            WeakType::Number(num) => num.to_string(),
            WeakType::Object(_) => String::from("[object Object]"),
            WeakType::Undefined => String::from("undefined"),
            WeakType::Array(_) => {
                fn reducer(val: &WeakType) -> String {
                    match val {
                        WeakType::String(_) => val.coerce_to_string(),
                        WeakType::Number(_) => val.coerce_to_string(),
                        WeakType::Object(_) => val.coerce_to_string(),
                        WeakType::Array(arr) => arr
                            .0
                            .iter()
                            .map(reducer)
                            .reduce(|prev, cur| prev + ", " + &cur)
                            .unwrap(),
                        WeakType::Undefined => val.coerce_to_string(),
                    }
                }
                reducer(self)
            }
        }
    }
}

pub trait IntoWeakType {
    fn into(self) -> WeakType;
}

impl IntoWeakType for f64 {
    fn into(self) -> WeakType {
        WeakType::Number(self)
    }
}

impl IntoWeakType for i32 {
    fn into(self) -> WeakType {
        WeakType::Number(f64::from(self))
    }
}

impl IntoWeakType for &str {
    fn into(self) -> WeakType {
        WeakType::String(self.to_string())
    }
}

impl IntoWeakType for String {
    fn into(self) -> WeakType {
        WeakType::String(self)
    }
}

impl IntoWeakType for HashMap<&'static str, WeakType> {
    fn into(self) -> WeakType {
        WeakType::Object(Object(self))
    }
}

impl IntoWeakType for Object {
    fn into(self) -> WeakType {
        WeakType::Object(self)
    }
}

impl IntoWeakType for Vec<WeakType> {
    fn into(self) -> WeakType {
        WeakType::Array(Array(self))
    }
}

impl IntoWeakType for Array {
    fn into(self) -> WeakType {
        WeakType::Array(self)
    }
}

#[derive(Clone)]
pub struct Array(Vec<WeakType>);

impl Array {
    fn from(arr: &[WeakType]) -> WeakType {
        WeakType::from(Array(arr.to_vec()))
    }
}

#[derive(Clone)]
pub struct Object(HashMap<&'static str, WeakType>);

pub trait FromValues<T> {
    fn from_values(value: T) -> WeakType;
}

impl<const N: usize> FromValues<[(&'static str, WeakType); N]> for Object {
    fn from_values(arr: [(&'static str, WeakType); N]) -> WeakType {
        WeakType::from(Object(HashMap::from_iter(arr)))
    }
}

#[allow(non_snake_case)]
fn Number(n: impl IntoWeakType) -> WeakType {
    WeakType::from(n.into().coerce_to_number())
}

#[allow(non_snake_case)]
fn String(s: impl IntoWeakType) -> WeakType {
    WeakType::from(s.into().coerce_to_string())
}

fn main() {
    let obj = Object::from_values([
        ("Mercury", Number(0.4)),
        ("Venus", Number(0.7)),
        ("Earth", Number(1.0)),
        ("Mars", Number(1.5)),
    ]);

    let arr = Array::from(&[
        String(1),
        Number("5"),
        Array::from(&[obj.clone(), String(20)]),
    ]);

    println!("{}", &arr + &obj)//: '1, 5, [object Object], 20[object Object]'

}
