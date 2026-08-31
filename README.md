# turso-dump

An interactive SQL shell for a local [Turso](https://github.com/tursodatabase/turso) database, with a
non-interactive full-table dump as a side mode.

```
$ cargo run -- node.db
turso-dump interactive session
connected to node.db
enter SQL terminated by ';', or .help for commands, .quit to exit

turso-dump > SELECT * FROM nodes;
┌────┬────────────────┬───────┬─────────────┐
│ id │ name           │ score │ payload     │
├────┼────────────────┼───────┼─────────────┤
│ 1  │ alice          │ 9.5   │ x'deadbeef' │
│ 2  │ bob;with;semis │ 3.14  │ NULL        │
└────┴────────────────┴───────┴─────────────┘
-- 2 row(s) in 0.22ms
```

## Build

```bash
cargo build --release
```

The binary lands at `target/release/turso-dump`.

## Usage

```
usage: turso-dump [options] <database>

  database        path to the database file, created if it does not exist
  -d, --dump      dump every table and exit instead of opening a session
  -h, --help      show this help
```

The database path is required - there is no default. Running with no `--dump` opens the interactive
session. The database file is created if it does not exist.

```bash
cargo run -- some.db          # interactive session
cargo run -- -d some.db       # dump every table, then exit
```

## The session

Type SQL and terminate it with `;`. Anything unterminated carries over to a continuation prompt, so
statements can span as many lines as you like:

```
turso-dump > SELECT id, name
        ... FROM nodes
        ... WHERE id > 1;
```

Statements that return columns (`SELECT`, `PRAGMA`, …) are rendered as a table with a row count and
elapsed time. Everything else reports the number of rows affected:

```
turso-dump > UPDATE nodes SET score = 9.5 WHERE id = 1;
-- 1 row(s) affected in 0.38ms
```

Several statements on one line run in order. Semicolons inside string literals, quoted identifiers,
and comments do not split a statement, so `INSERT INTO t VALUES ('a;b');` works as written.

### Commands

| Command | Effect |
| --- | --- |
| `.help` | List the commands |
| `.tables` | List the tables in the database |
| `.schema [name]` | Show the schema, optionally for one table |
| `.dump` | Dump every table with its rows |
| `.quit` / `.exit` | Leave the session |

### Keys

| Key | Effect |
| --- | --- |
| `↑` / `↓` | Walk the history |
| `Ctrl-C` | Discard the partial statement, or print a hint when there is none |
| `Ctrl-D` | Leave the session |

History persists across runs in `~/.turso_dump_history`.

## Errors

A failing statement prints to stderr and the session continues - only an unreadable terminal ends it:

```
turso-dump > SELECT bad syntax;
Error: failed to prepare the statement: Parse error: no such column: bad
```

## Layout

| File | Responsibility |
| --- | --- |
| `src/main.rs` | Argument parsing, opening the database, choosing session or dump |
| `src/repl.rs` | The session loop, dot-commands, running one statement |
| `src/sql.rs` | Splitting input into statements, quote- and comment-aware |
| `src/output.rs` | Rendering values, tables, and elapsed times |
| `src/catalog.rs` | `sqlite_master` queries backing `.tables` and `.schema` |
| `src/dump.rs` | The full-database dump |

## Dependencies

| Crate | Why |
| --- | --- |
| `turso` | The database engine |
| `tokio` | Async runtime - the turso API is async |
| `rustyline` | Line editing, history, and the prompt |
| `anyhow` | Error handling |
