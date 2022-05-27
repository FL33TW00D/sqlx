use crate::connection::ConnectOptions;
use crate::error::Error;
use crate::inspect::{ColumnData, ForeignKeyConstraint, Inspect, InspectDatabase, TableData};
use crate::postgres::{PgConnectOptions, Postgres};
use crate::query_as::query_as;
use futures_core::future::BoxFuture;
use std::str::FromStr;

use super::PgConnection;

impl InspectDatabase for Postgres {
    //Output schema needs to do more, instead of just calling the methods
    //it should handle writing out and combining
    fn output_schema(uri: &str) -> BoxFuture<'_, Result<(), Error>> {
        Box::pin(async move {
            let options = PgConnectOptions::from_str(uri)?;
            let mut conn: PgConnection = options.connect().await?;

            //We need a method to obtain the foreign keys
            let table_names = conn.list_table_names().await?;
            for tn in table_names {
                println!("{}", conn.load_table_data(tn.clone()).await?);
            }

            let fks = conn.load_foreign_keys().await?;
            for fk in fks {
                println!("{}", fk);
            }

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
            //here we need to do a default schema check
            .bind("public")
            //Omit the _sql_migrations table
            .bind("\\_sqlx\\_%")
            .bind("BASE TABLE")
            .fetch_all(self)
            .await?;

            Ok(rows.into_iter().map(|(table_name,)| table_name).collect())
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

    fn load_foreign_keys(&mut self) -> BoxFuture<'_, Result<Vec<ForeignKeyConstraint>, Error>> {
        Box::pin(async move {
            let constraint_names: Vec<(String, String, String, String)> = query_as(
                r#"
                    SELECT referential_constraints.constraint_schema, referential_constraints.constraint_name,
                    referential_constraints.unique_constraint_schema, referential_constraints.unique_constraint_name
                    FROM information_schema.table_constraints INNER JOIN information_schema.referential_constraints
                    ON (table_constraints.constraint_schema = referential_constraints.constraint_schema)
                    AND (table_constraints.constraint_name = referential_constraints.constraint_name)
                    WHERE (constraint_type = $1) AND (table_schema = $2)
                "#,
            )
            //these binds are bad
            .bind("FOREIGN KEY")
            .bind("public")
            .fetch_all(&mut *self)
            .await?;

            //the below can be cleaned up massively
            let mut result = Vec::with_capacity(constraint_names.len());
            for cn in constraint_names {
                let (fks, fkn, pks, pkn) = cn;
                let (mut foreign_key_table, foreign_key_column): (String, String) = query_as(
                        r#"
                            SELECT key_column_usage.table_name, key_column_usage.table_schema, key_column_usage.column_name
                            FROM information_schema.key_column_usage
                            WHERE (key_column_usage.constraint_schema = $1)
                            AND (key_column_usage.constraint_name = $2)
                        "#
                        )
                        .bind(fks)
                        .bind(fkn)
                        .fetch_one(&mut *self).await?;

                let (mut primary_key_table, primary_key_column): (String, String) = query_as(
                        r#"
                            SELECT key_column_usage.table_name, key_column_usage.table_schema, key_column_usage.column_name
                            FROM information_schema.key_column_usage
                            WHERE (key_column_usage.constraint_schema = $1)
                            AND (key_column_usage.constraint_name = $2)
                        "#
                        )
                        .bind(pks)
                        .bind(pkn)
                        .fetch_one(&mut *self).await?;

                result.push(ForeignKeyConstraint {
                    child_table: foreign_key_table,
                    parent_table: primary_key_table,
                    foreign_key: foreign_key_column.clone(),
                    primary_key: primary_key_column,
                });
            }
            Ok(result)
        })
    }
}

//TODO
//Work out cool display method for FK relationships
//Default schema handling
//passing schema in via parameter
//Remove static binds
//Neaten up
//Testing
