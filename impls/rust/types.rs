use std::collections::{HashMap, VecDeque};
use std::fmt;
use std::fmt::Display;

extern crate thiserror;
use self::thiserror::Error;
use crate::reader::TokenType;
#[derive(Error, Debug, PartialEq, Clone)]
pub enum Value {
    // TODO distinguish between integer and float
    Number(f64),
    String(String),
    Boolean(bool),
    List(VecDeque<Value>),
    Vec(Vec<Value>),
    Map(HashMap<String, Value>),
    Symbol(String),
    Keyword(TokenType),
    Null,
    Error(String),
}
