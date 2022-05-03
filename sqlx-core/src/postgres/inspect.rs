use crate::connection::ConnectOptions;
use crate::error::Error;
use crate::inspect::{ColumnData, Inspect, InspectDatabase, TableData};
use crate::postgres::{PgConnectOptions, Postgres};
use crate::query::query;
use crate::query_as::query_as;
use futures_core::future::BoxFuture;
use std::str::FromStr;

use super::PgConnection;

impl InspectDatabase for Postgres {
    fn output_schema(uri: &str) -> BoxFuture<'_, Result<(), Error>> {
        Box::pin(async move {
            let mut options = PgConnectOptions::from_str(uri)?;
            options.database = Some("diesel_lab".into());
            let mut conn: PgConnection = options.connect().await?;

            let table_names = conn.list_table_names().await?;
            let table_data = conn.load_table_data(table_names[1].clone()).await?;
            println!("{}", table_data);
            Ok(())
        })
    }
}

impl Inspect for PgConnection {
    fn list_table_names(&mut self) -> BoxFuture<'_, Result<Vec<String>, Error>> {
        Box::pin(async move {
            let rows: Vec<(String,)> = query_as(
                r#"SELECT table_name FROM information_schema.tables 
                WHERE (table_schema = $1) AND (table_name NOT LIKE $2) 
                AND (table_type LIKE $3)"#,
            )
            .bind("public")
            //Omit the _sql_migrations table
            .bind("\\_sqlx\\_%")
            .bind("BASE TABLE")
            .fetch_all(self)
            .await?;

            let table_names = rows.into_iter().map(|(table_name,)| table_name).collect();
            Ok(table_names)
        })
    }

    fn load_table_data(&mut self, table_name: String) -> BoxFuture<'_, Result<TableData, Error>> {
        Box::pin(async move {
            //Here we are using udt_name and udt_schema
            //This should be an associated type for different backends
            let rows: Vec<(String, String, String, String)> = query_as(
                r#"SELECT column_name, udt_name, udt_schema, is_nullable FROM information_schema.columns 
                WHERE (table_name = $1) AND (table_schema = $2)"#,
            )
            //these binds are bad
            .bind(table_name.clone())
            .bind("public")
            .fetch_all(self)
            .await?;

            let column_data = rows
                .into_iter()
                .map(|c| ColumnData {
                    name: c.0,
                    column_type: c.1,
                })
                .collect();

            Ok(TableData {
                name: table_name.clone(),
                column_data,
            })
        })
    }
}
