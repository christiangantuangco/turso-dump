use std::path::PathBuf;
use std::time::Instant;

use anyhow::{Context, Result};
use rustyline::error::ReadlineError;
use rustyline::DefaultEditor;
use turso::Connection;

use crate::catalog;
use crate::dump;
use crate::output;
use crate::sql;

const PROMPT: &str = "turso-dump > ";
const CONTINUATION_PROMPT: &str = "        ... ";
const HISTORY_FILE: &str = ".turso_dump_history";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Flow {
    Continue,
    Quit,
}

pub(crate) async fn run(conn: &Connection, db_path: &str) -> Result<()> {
    let mut editor =
        DefaultEditor::new().context("failed to start the interactive session reader")?;
    let history = get_history_path();
    if let Some(path) = &history {
        let _ = editor.load_history(path);
    }

    println!("turso-dump interactive session");
    println!("connected to {db_path}");
    println!("enter SQL terminated by ';', or .help for commands, .quit to exit\n");

    let mut buffer = String::new();

    loop {
        let prompt = if buffer.trim().is_empty() {
            PROMPT
        } else {
            CONTINUATION_PROMPT
        };

        match editor.readline(prompt) {
            Ok(line) => {
                if buffer.trim().is_empty() {
                    let trimmed = line.trim();
                    if trimmed.is_empty() {
                        continue;
                    }
                    if trimmed.starts_with('.') {
                        let _ = editor.add_history_entry(trimmed);
                        match run_dot_command(conn, trimmed).await {
                            Ok(Flow::Continue) => {}
                            Ok(Flow::Quit) => break,
                            Err(err) => eprintln!("Error: {err:#}"),
                        }
                        continue;
                    }
                }

                buffer.push_str(&line);
                buffer.push('\n');

                let split = sql::split_statements(&buffer);
                buffer = split.remainder;
                for statement in split.statements {
                    let _ = editor.add_history_entry(statement.as_str());
                    if let Err(err) = run_statement(conn, &statement).await {
                        eprintln!("Error: {err:#}");
                    }
                }
            }
            Err(ReadlineError::Interrupted) => {
                if buffer.trim().is_empty() {
                    println!("(use .quit or Ctrl-D to exit)");
                } else {
                    buffer.clear();
                }
            }
            Err(ReadlineError::Eof) => break,
            Err(err) => {
                return Err(err).context("failed to read from the interactive session");
            }
        }
    }

    if let Some(path) = &history {
        let _ = editor.save_history(path);
    }

    Ok(())
}

async fn run_statement(conn: &Connection, statement: &str) -> Result<()> {
    let started = Instant::now();
    let mut stmt = conn
        .prepare(statement)
        .await
        .context("failed to prepare the statement")?;

    if stmt.column_count() == 0 {
        let affected = stmt
            .execute(())
            .await
            .context("failed to execute the statement")?;
        println!(
            "-- {affected} row(s) affected in {}",
            output::format_elapsed(started.elapsed())
        );
        return Ok(());
    }

    let columns = stmt.column_names();
    let mut rows = stmt.query(()).await.context("failed to run the query")?;

    let mut cells: Vec<Vec<String>> = Vec::new();
    while let Some(row) = rows.next().await.context("failed to read a row")? {
        let mut values: Vec<String> = Vec::with_capacity(columns.len());
        for index in 0..columns.len() {
            let value = row
                .get_value(index)
                .context("failed to read a column value")?;
            values.push(output::format_value(&value));
        }
        cells.push(values);
    }

    print!("{}", output::render_table(&columns, &cells));
    println!(
        "-- {} row(s) in {}",
        cells.len(),
        output::format_elapsed(started.elapsed())
    );

    Ok(())
}

async fn run_dot_command(conn: &Connection, line: &str) -> Result<Flow> {
    let mut parts = line.splitn(2, char::is_whitespace);
    let command = parts.next().unwrap_or_default();
    let argument = parts.next().map(str::trim).unwrap_or_default();

    match command {
        ".help" => print_help(),
        ".tables" => print_tables(conn).await?,
        ".schema" => print_schema(conn, argument).await?,
        ".dump" => dump::dump_all(conn).await?,
        ".quit" | ".exit" => return Ok(Flow::Quit),
        _ => eprintln!("unknown command: {command} (try .help)"),
    }

    Ok(Flow::Continue)
}

async fn print_tables(conn: &Connection) -> Result<()> {
    let names = catalog::get_table_names(conn).await?;
    if names.is_empty() {
        println!("-- no tables");
        return Ok(());
    }
    for name in &names {
        println!("{name}");
    }
    Ok(())
}

async fn print_schema(conn: &Connection, argument: &str) -> Result<()> {
    let name = if argument.is_empty() {
        None
    } else {
        Some(argument)
    };
    let definitions = catalog::get_schema(conn, name).await?;

    if definitions.is_empty() {
        println!("-- no matching schema");
        return Ok(());
    }
    for definition in &definitions {
        println!("{definition};");
    }
    Ok(())
}

fn print_help() {
    println!(".help              show this help");
    println!(".tables            list the tables in the database");
    println!(".schema [name]     show the schema, optionally for one table");
    println!(".dump              dump every table with its rows");
    println!(".quit / .exit      leave the session (Ctrl-D also works)");
    println!();
    println!("anything else is run as SQL once terminated by ';'");
}

fn get_history_path() -> Option<PathBuf> {
    let home = std::env::var_os("HOME").or_else(|| std::env::var_os("USERPROFILE"))?;
    Some(PathBuf::from(home).join(HISTORY_FILE))
}
