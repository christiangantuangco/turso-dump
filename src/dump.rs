use anyhow::{Context, Result};
use turso::{Connection, Value};

use crate::catalog;
use crate::output;

pub(crate) async fn dump_all(conn: &Connection) -> Result<()> {
    let names = catalog::get_table_names(conn).await?;

    if names.is_empty() {
        println!("-- no tables");
        return Ok(());
    }

    for name in names {
        let mut stmt = conn
            .prepare(&format!("SELECT * FROM \"{name}\""))
            .await
            .context("failed to prepare the table query")?;
        let columns = stmt.column_names();
        let mut rows = stmt.query(()).await.context("failed to query the table")?;

        let mut cells: Vec<Vec<String>> = Vec::new();
        let mut numeric_columns = vec![true; columns.len()];
        while let Some(row) = rows.next().await.context("failed to read a row")? {
            let mut values: Vec<String> = Vec::with_capacity(columns.len());
            for (index, numeric_column) in numeric_columns.iter_mut().enumerate() {
                let value = row
                    .get_value(index)
                    .context("failed to read a column value")?;
                if !matches!(value, Value::Null) {
                    *numeric_column &= output::is_numeric_value(&value);
                }
                values.push(output::format_value(&value));
            }
            cells.push(values);
        }

        let alignments: Vec<output::Alignment> = numeric_columns
            .iter()
            .map(|&numeric| {
                if numeric {
                    output::Alignment::Right
                } else {
                    output::Alignment::Left
                }
            })
            .collect();

        println!("\n=== {name} ===");
        print!("{}", output::render_table(&columns, &cells, &alignments));
        println!("-- {} row(s)", cells.len());
    }

    Ok(())
}
