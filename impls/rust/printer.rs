use std::fmt::{self, Display};

use crate::types::Value;

impl Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Value::Number(x) => write!(f, "{}", x),
            Value::String(x) => write!(f, "{}", x),
            Value::Boolean(x) => match x {
                true => write!(f, "true"),
                false => write!(f, "false"),
            },
            Value::List(x) => {
                write!(f, "(")?;
                for elem in x.iter() {
                    write!(f, "{}", elem)?;
                }
                write!(f, ")")
            }
            Value::Vec(x) => {
                write!(f, "[")?;
                for elem in x.iter() {
                    write!(f, "{}", elem)?;
                }
                write!(f, "]")
            }
            Value::Map(x) => {
                write!(f, "{{")?;
                for (k, v) in x.iter() {
                    write!(f, "{}:{}", k, v)?;
                }
                write!(f, "}}")
            }
            Value::Symbol(x) => write!(f, "{}", x),
            Value::Null => write!(f, "null"),
            Value::Error(x) => write!(f, "{}", x),
        }
    }
}
