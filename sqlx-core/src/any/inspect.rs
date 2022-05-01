use crate::any::{Any, AnyKind};
use crate::inspect::InspectDatabase;
use futures_core::future::BoxFuture;
use std::str::FromStr;
impl InspectDatabase for Any {
    fn output_schema(uri: &str) -> BoxFuture<'_, Result<(), crate::error::Error>> {
        Box::pin(async move {
            match AnyKind::from_str(uri)? {
                #[cfg(feature = "postgres")]
                AnyKind::Postgres => crate::postgres::Postgres::output_schema(uri).await,

                #[cfg(feature = "sqlite")]
                AnyKind::Sqlite => Ok(()),

                #[cfg(feature = "mysql")]
                AnyKind::MySql => Ok(()),

                #[cfg(feature = "mssql")]
                AnyKind::Mssql => unimplemented!(),
            }
        })
    }
}
