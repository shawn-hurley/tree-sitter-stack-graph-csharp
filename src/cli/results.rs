use std::{any::Any, collections::HashMap};


#[derive(Debug)]
pub struct Result {
    pub file_uri: String,
    pub line_number: usize,
    pub variables: HashMap<String, Box<dyn Any>>,
    pub code_location: Location,

}

#[derive(Debug)]
pub struct Position {
    pub line: usize,
    pub character: usize, 
}

#[derive(Debug)]
pub struct Location {
    pub start_position: Position,
    pub end_position: Position,
}