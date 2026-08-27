mod catalog;
mod dump;
mod output;
mod repl;
mod sql;

use anyhow::{bail, Context, Result};
use turso::Builder;

const DEFAULT_DB_PATH: &str = "node.db";

#[tokio::main]
async fn main() -> Result<()> {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let mut db_path: Option<String> = None;
    let mut dump_only = false;

    for arg in args {
        match arg.as_str() {
            "-h" | "--help" => {
                print_usage();
                return Ok(());
            }
            "-d" | "--dump" => dump_only = true,
            other if other.starts_with('-') => {
                eprintln!("unknown option: {other}");
                bail!("unknown option");
            }
            other => db_path = Some(other.to_string()),
        }
    }

    let db_path = db_path.unwrap_or_else(|| DEFAULT_DB_PATH.to_string());

    let db = Builder::new_local(&db_path)
        .build()
        .await
        .context("failed to open the database")?;
    let conn = db.connect().context("failed to connect")?;

    if dump_only {
        dump::dump_all(&conn).await
    } else {
        repl::run(&conn, &db_path).await
    }
}

fn print_usage() {
    println!("usage: turso-dump [options] [database]");
    println!();
    println!("  database        path to the database file (default: {DEFAULT_DB_PATH})");
    println!("  -d, --dump      dump every table and exit instead of opening a session");
    println!("  -h, --help      show this help");
}
