use anyhow::{Context, Result};
use turso::Connection;

const TABLE_NAMES_SQL: &str = "SELECT name FROM sqlite_master WHERE type='table' \
     AND name NOT LIKE 'sqlite_%' ORDER BY name";

const SCHEMA_SQL: &str = "SELECT sql FROM sqlite_master WHERE sql IS NOT NULL \
     AND name NOT LIKE 'sqlite_%' ORDER BY name";

const SCHEMA_FOR_NAME_SQL: &str = "SELECT sql FROM sqlite_master WHERE sql IS NOT NULL \
     AND (name = ? OR tbl_name = ?) ORDER BY name";

pub(crate) async fn get_table_names(conn: &Connection) -> Result<Vec<String>> {
    let mut rows = conn
        .query(TABLE_NAMES_SQL, ())
        .await
        .context("failed to list the tables")?;

    let mut names: Vec<String> = Vec::new();
    while let Some(row) = rows.next().await.context("failed to read a table name")? {
        let value = row.get_value(0).context("failed to read a table name")?;
        if let Some(name) = value.as_text() {
            names.push(name.clone());
        }
    }

    Ok(names)
}

pub(crate) async fn get_schema(conn: &Connection, name: Option<&str>) -> Result<Vec<String>> {
    let mut rows = match name {
        Some(name) => conn.query(SCHEMA_FOR_NAME_SQL, [name, name]).await,
        None => conn.query(SCHEMA_SQL, ()).await,
    }
    .context("failed to read the schema")?;

    let mut definitions: Vec<String> = Vec::new();
    while let Some(row) = rows.next().await.context("failed to read a schema entry")? {
        let value = row.get_value(0).context("failed to read a schema entry")?;
        if let Some(sql) = value.as_text() {
            definitions.push(sql.clone());
        }
    }

    Ok(definitions)
}
