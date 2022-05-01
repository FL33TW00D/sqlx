use crate::error::Error;
use futures_core::future::BoxFuture;

/// This trait defines high level inspection methods for a DB backend
/// It is implemented for all DB types that SQLx supports
pub trait InspectDatabase {
    fn output_schema(uri: &str) -> BoxFuture<'_, Result<(), Error>>;
}

/// This trait defines low-level inspect methods for a DB backend
pub trait Inspect {
    fn list_table_names(&mut self) -> BoxFuture<'_, Result<Vec<String>, Error>>;

    fn load_table_data(
        &mut self,
        table_names: Vec<String>,
    ) -> BoxFuture<'_, Result<Vec<TableData>, Error>>;
}

#[derive(Debug, Clone)]
pub struct TableData {
    pub name: String,
    pub column_data: Vec<ColumnData>,
}

#[derive(Debug)]
pub struct ColumnData {
    pub sql_name: String,
    pub column_type: String,
}

