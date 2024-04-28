use thiserror::Error;

#[derive(Error, Debug)]
pub enum ModelError {
    #[error("couldn't parse fit entry: {0}")]
    ParseError(String),
    #[error("couldn't insert entry into database: {0}")]
    InsertError(String),
}

#[derive(Debug, Clone)]
pub struct DatabaseEntry<S: DatabaseState, T> {
    pub state: Box<T>,
    pub extra: S,
}

#[derive(Debug, Clone)]
pub struct New;
#[non_exhaustive]
#[derive(Debug, Clone)]
pub struct Stored {
    pub activity_id: i64,
}

pub trait DatabaseState {}
impl DatabaseState for New {}
impl DatabaseState for Stored {}

#[derive(Debug, Clone)]
pub struct Coordinates {
    pub latitude: i32,
    pub longitude: i32,
}
