# nav

A terminal directory bookmark manager scoped to your project.

Bookmarks live in a `.navmarks` file at your repo root — commit it to share with your team and AI tools.

```bash
nav init                   # auto-generate .navmarks from directory conventions
nav                        # fuzzy-jump with fzf
nav list                   # JSONL output for scripts and agents
```

## Install

Requires `fzf`.

### Via Cargo

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

Add to `~/.zshrc`, `~/.bashrc`, or `~/.config/fish/config.fish`:

```bash
eval "$(nav init zsh)"   # or bash / fish
```

## Quick Start

```bash
# In any repo:
nav init                  # detect src/, tests/, docs/, api/, etc. and write .navmarks
git add .navmarks && git commit -m "add nav bookmarks"

# From now on, anyone who clones the repo can:
nav                       # fuzzy-jump with fzf
nav list                  # print bookmarks as JSONL
```

## Usage

```bash
nav init                                        # auto-generate .navmarks from directory conventions
nav mark -t api                                 # manually bookmark current directory
nav mark src/api -t api -n "main event loop"    # bookmark with a note for agents
nav remove src/api                              # remove a bookmark
nav                                             # open fzf picker, cd on select
nav list                                        # print bookmarks as JSONL
```

nav resolves paths relative to your git root, so bookmarks stay portable across machines.

## .navmarks

Bookmarks are stored as a plain text file at your repo root:

```
src/api       api,entrypoint    main event loop, start here
src/auth      auth,security     JWT validation — touch carefully
src/frontend  frontend,react
tests         tests
```

Each line is `<path>\t<tags>\t<optional note>`. The note column is freetext — use it to leave context for agents and teammates that directory names alone can't convey.

`nav init` detects common directory names (`src`, `tests`, `docs`, `api`, `frontend`, `config`, `scripts`, and more) and writes this file automatically. Existing entries are never overwritten.

Commit it to give your teammates and AI agents an instant semantic map of the repo.

## AI Agent Integration

`nav list` bypasses fzf and outputs JSONL — safe for scripts and AI tools:

```jsonl
{"path": "/Users/san/Projects/myapp/src/api", "rel": "src/api", "tags": "api, entrypoint"}
{"path": "/Users/san/Projects/myapp/tests", "rel": "tests", "tags": "tests"}
```

See [agent.md](agent.md) for details.

## License

MIT
