# Docker

Runs `turso-dump` in a container. By default it opens `/var/lib/opensff/mm/node.db`.

The database's folder is bind-mounted (a host folder shared into the container), so the database
stays on the host and is never lost when the container is removed.

Run everything from the repository root, with Docker running.

## First run

```bash
docker compose build
docker compose run --rm turso-dump
```

`run --rm` opens an interactive session and deletes the container on exit.

## Choosing the database

| Variable | Default | Meaning |
|---|---|---|
| `DB_DIR` | `/var/lib/opensff/mm` | Host folder mounted at `/data` |
| `DB_FILE` | `node.db` | File inside that folder |

| Goal | Command |
|---|---|
| Default database | `docker compose run --rm turso-dump` |
| Another file | `DB_FILE=other.db docker compose run --rm turso-dump` |
| Another folder and file | `DB_DIR=/srv/backups DB_FILE=snap.db docker compose run --rm turso-dump` |
| Dump every table, then exit | `docker compose run --rm turso-dump -d /data/node.db` |

Put `DB_DIR` and `DB_FILE` in a `.env` file next to `compose.yaml` to avoid retyping them. On a Mac,
set `DB_DIR` to a local folder, since the default path is a Linux location.

## I changed something, what do I do?

| What changed | Command |
|---|---|
| Rust code, `Cargo.toml`, or `Dockerfile` | `docker compose build` |
| `compose.yaml`, `DB_DIR`, `DB_FILE`, or `.env` | Nothing, the next `run` picks it up |

## Good to know

- Sessions write to the real database file. Work on a copy when unsure.
- The container runs as root, so files it creates are root-owned.
