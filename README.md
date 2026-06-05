# nav

A fast, tag-based terminal bookmark manager for jumping between projects.

Bookmark your projects with tags like `#rust` or `#python`, then fuzzy-jump
to any of them from anywhere in your terminal.

```bash
nav add -t rust       # bookmark the current project
nav                   # fuzzy-jump with fzf — type #rust to filter
```

## Install

Requires [`fzf`](https://github.com/junegunn/fzf).

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

Then add the shell hook to `~/.zshrc`, `~/.bashrc`, or `~/.config/fish/config.fish`:

```bash
eval "$(nav init zsh)"   # or bash / fish
```

The hook is what lets `nav` change your shell's directory on select.

## Usage

```bash
nav add                      # bookmark current dir (auto-detects language tag)
nav add ~/Projects/foo       # bookmark a specific path
nav add -t rust -t cli       # bookmark with explicit tags
nav                          # open fzf picker, cd on select
nav list                     # print all bookmarks
nav rm ~/Projects/foo        # remove a bookmark
nav edit                     # open the bookmarks file in $EDITOR
```

In the picker, just type a tag (e.g. `rust`) to narrow the list — fzf matches
against the `#tags` shown on each line.

### Auto-detected tags

When you run `nav add` without `-t`, nav guesses a language tag from the files
in the directory:

| Marker file        | Tag      |
|--------------------|----------|
| `Cargo.toml`       | `rust`   |
| `go.mod`           | `go`     |
| `package.json`     | `node`   |
| `pyproject.toml` / `requirements.txt` / `setup.py` | `python` |
| `pom.xml` / `build.gradle` | `java` |
| `Gemfile`          | `ruby`   |
| `composer.json`    | `php`    |
| `CMakeLists.txt`   | `cpp`    |
| `Package.swift`    | `swift`  |
| `mix.exs`          | `elixir` |

If nothing matches, the bookmark gets the `general` tag. You can always add
more with another `nav add -t <tag>`.

## Storage

Bookmarks live in a single global file at `~/.config/nav/marks`
(`$XDG_CONFIG_HOME/nav/marks` if set). Each line is `<absolute-path>\t<tags>`:

```
/Users/san/Projects/nav        rust,cli
/Users/san/Projects/api        python,backend
/Users/san/Projects/site       node,frontend
```

It's plain text — edit it by hand with `nav edit` whenever you like.

## License

MIT
