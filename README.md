# Minit: A Rust Implementation of Git

A minimal Git implementation in Rust, inspired by [Write Yourself a Git](https://wyag.thb.lt/).

## Building

```bash
cargo build --release
```

## Usage

```bash
minit <command> [options]
```

## Implemented Commands

| Command | Description |
|---------|-------------|
| `init [path]` | Initialize a new, empty minit repository |
| `cat-file <object> [type]` | Display contents of repository objects |
| `hash-object [-w] <file>` | Compute object ID and optionally write blob to database |
| `log [commit]` | Display commit history (defaults to HEAD) |
| `ls-tree <tree> [recursive]` | Print contents of a tree object |
| `checkout <commit> <directory>` | Checkout a commit into an empty directory |
| `tag [-a] [name] [object]` | Create or list tags |
| `show-ref` | List references |
| `rev-parse -n <name> [-t type]` | Resolve a name to an object SHA |

## Object Types

Minit supports the four core Git object types:
- **Blob** - File contents
- **Tree** - Directory listings
- **Commit** - Snapshots with metadata
- **Tag** - Named references to objects

## Repository Structure

Minit creates a `.minit` directory with:
```
.minit/
├── branches/
├── objects/
├── refs/
│   ├── heads/
│   └── tags/
├── config
├── description
└── HEAD
```

## Dependencies

- `clap` - CLI argument parsing
- `flate2` - Zlib compression
- `sha2` - SHA-256 hashing
- `configparser` - INI config file support
- `indexmap` - Ordered hash maps
- `regex` - Reference resolution
