use sqlx::any::Any;
use sqlx::inspect::InspectDatabase;

pub async fn run(uri: &str, schema: &str) -> anyhow::Result<()> {
    Any::output_schema(uri).await;
    Ok(())
}
