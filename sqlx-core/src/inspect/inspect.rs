use crate::error::Error;
use futures_core::future::BoxFuture;
use std::fmt::{self, Display, Formatter, Write};

/// This trait defines high level inspection methods for a DB backend
/// It is implemented for all DB types that SQLx supports
pub trait InspectDatabase {
    fn output_schema(uri: &str) -> BoxFuture<'_, Result<(), Error>>;
}

/// This trait defines low-level inspect methods for a DB backend
pub trait Inspect {
    fn list_table_names(&mut self) -> BoxFuture<'_, Result<Vec<String>, Error>>;

    fn load_table_data(&mut self, table_name: String) -> BoxFuture<'_, Result<TableData, Error>>;
}

#[derive(Debug)]
pub struct TableData {
    pub name: String,
    pub column_data: Vec<ColumnData>,
}

impl Display for TableData {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        writeln!(f, "Table: {} {{", self.name);
        for col in &self.column_data {
            write!(f, "{}", col);
        }
        writeln!(f, "}}");
        Ok(())
    }
}

#[derive(Debug)]
pub struct ColumnData {
    pub name: String,
    pub column_type: String,
}

impl Display for ColumnData {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        writeln!(f, "  {} -> {}", self.name, self.column_type);
        Ok(())
    }
}
