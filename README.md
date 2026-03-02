# inv

`inv` is a Rust CLI for managing electronics inventory as code.

- Source of truth: JSON (`inventory.json`)
- Validation: strict JSON Schema + semantic checks
- Storage: atomic writes with deterministic formatting/order

## Quick start

```bash
# initialize inventory.json in the current directory
inv init

# interactively add an item
inv add

# list all items
inv list

# inspect one item
inv show <uuid>

# validate schema + semantics
inv validate
```

## Data file and schema

`inv` stores data in a single JSON document:

- top-level: `version`, `items`
- each item includes UUID `id`, `name`, `quantity`, `unit`, timestamps, and optional metadata fields

Schema file:

- `schema/inventory.schema.json`

Unknown fields are rejected (`additionalProperties: false`), and semantic validation rejects duplicate IDs and invalid `source_url` values.

## Commands

```text
inv init
inv add
inv update <id>
inv search <query> [--json]
inv show <id> [--json]
inv list [--json]
inv remove <id> [--yes]
inv qr <id> [--out <path>]
inv label <id> [--json]
inv validate
inv ios-setup [--url <https-url>]
```

Notes:

- `add` and `update` are interactive.
- `search` matches only `name` and `description` (case-insensitive).
- `remove` requires confirmation in non-interactive usage (`--yes`).
- `label` is a **v1 terminal placeholder** (no printer integration yet).

## iOS setup

Use:

```bash
inv ios-setup
```

This prints:

1. short setup instructions
2. a terminal QR code for the iOS Shortcut URL
3. the raw fallback URL

You can override the URL via `--url` or environment variable.

## Configuration

### Database path

Resolution order:

1. `--db-path <path>`
2. `INV_DB_PATH`
3. `./inventory.json`

Example:

```bash
INV_DB_PATH=./data/inventory.json inv list
# or
inv --db-path ./data/inventory.json list
```

### iOS Shortcut URL

Resolution order:

1. `inv ios-setup --url <https-url>`
2. `INV_IOS_SHORTCUT_URL`
3. built-in default iCloud Shortcut URL

Only valid absolute `https://` URLs are accepted.

## Development

```bash
devenv shell -- cargo run -- --help
devenv shell -- cargo test
devenv test
```
