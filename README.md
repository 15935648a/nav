# nav

A terminal directory bookmark manager with semantic tags.

```bash
nav mark -t work -t api   # bookmark current dir
nav                       # fuzzy-jump with fzf
nav list                  # JSONL output for scripts and agents
```

## Install

Requires `fzf`.

### Via Cargo (Recommended)

```bash
cargo install nav-cli
```

### From Source

```bash
git clone https://github.com/15935648a/nav.git
cd nav
cargo build --release
cp target/release/nav ~/.cargo/bin/
```

Add to `~/.zshrc` or `~/.bashrc`:

```bash
eval "$(nav init zsh)"
```

## Usage

```bash
nav mark -t work -t api   # tag and bookmark current directory
nav mark ~/some/path      # bookmark a specific path
nav remove .              # remove a bookmark
nav                       # open fzf picker, cd on select
nav --zoxide              # merge zoxide history into picker
nav list                  # print bookmarks as JSONL
```

Bookmarks are stored in `~/.nav_bookmarks.db` (SQLite). Dead links are pruned automatically.

### Help

```
usage: nav [options] [command]

  COMMANDS
    mark      Mark current or specified path as a bookmark
    remove    Remove a bookmark manually
    list      List all active bookmarks in JSONL format (Agent-friendly mode)
    init      Generate shell initialization script

  OPTIONS
    --zoxide  Load zoxide history
    --help    Print this message
```

## Zoxide Integration

```bash
nav --zoxide
# or persist with: export NAV_WITH_ZOXIDE=1
```

## AI Agent Integration

`nav list` bypasses fzf and outputs JSONL — safe for scripts and AI tools:

```jsonl
{"path": "/Users/san/Projects/web", "tags": "work, frontend"}
{"path": "/var/log", "tags": "server, logs"}
```

See [agent.md](agent.md) for details.

## License

MIT
