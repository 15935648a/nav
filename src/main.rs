use clap::{Parser, Subcommand};
use serde_json::json;
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

#[derive(Parser)]
#[command(
    name = "nav",
    disable_help_subcommand = true,
    help_template = "\
usage: nav [options] [command]

  COMMANDS
    init      Auto-generate .navmarks from directory conventions
    init      Generate shell init script: nav init <zsh|bash|fish>
    mark      Mark current or specified path as a bookmark
    remove    Remove a bookmark manually
    list      List all active bookmarks in JSONL format (Agent-friendly mode)

  OPTIONS
    --help    Print this message
"
)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// Mark current or specified path as a bookmark
    Mark {
        /// Path to bookmark (defaults to current directory)
        #[arg(default_value = ".")]
        path: String,
        /// Tags for the bookmark (can be used multiple times)
        #[arg(short, long)]
        tag: Vec<String>,
        /// Optional note for agents (e.g. "main event loop, start here")
        #[arg(short, long)]
        note: Option<String>,
    },
    /// List all active bookmarks in JSONL format (Agent-friendly mode)
    List,
    /// Remove a bookmark manually
    Remove {
        /// Path to remove from bookmarks
        path: String,
    },
    /// Auto-generate .navmarks from conventions, or generate shell init script
    Init {
        /// Shell name for shell integration (zsh, bash, fish).
        /// Omit to auto-generate .navmarks from directory conventions.
        shell: Option<String>,
    },
}

struct Bookmark {
    abs_path: PathBuf,
    rel_path: String,
    tags: Vec<String>,
    note: Option<String>,
}

fn find_project_root() -> PathBuf {
    let cwd = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    let mut current = cwd.clone();
    loop {
        if current.join(".git").exists() || current.join(".navmarks").exists() {
            return current;
        }
        if !current.pop() {
            return cwd;
        }
    }
}

fn read_navmarks(root: &Path) -> Vec<Bookmark> {
    let content = fs::read_to_string(root.join(".navmarks")).unwrap_or_default();
    content
        .lines()
        .filter(|l| {
            let t = l.trim();
            !t.is_empty() && !t.starts_with('#')
        })
        .filter_map(|line| {
            // First separator: tab or space (backward compat)
            let (raw_rel, rest) = line.split_once(|c: char| c == '\t' || c == ' ')?;
            let rel = raw_rel.trim().to_string();

            // Second separator: tab only (note may contain spaces)
            let (tags_str, note) = match rest.split_once('\t') {
                Some((t, n)) => (t.trim(), Some(n.trim().to_string()).filter(|s| !s.is_empty())),
                None => (rest.trim(), None),
            };

            let tags: Vec<String> = tags_str
                .split(',')
                .map(|t| t.trim().to_string())
                .filter(|t| !t.is_empty())
                .collect();

            let abs_path = if rel == "." { root.to_path_buf() } else { root.join(&rel) };
            Some(Bookmark { abs_path, rel_path: rel, tags, note })
        })
        .collect()
}

fn write_navmarks(root: &Path, bookmarks: &[Bookmark]) {
    let content: String = bookmarks
        .iter()
        .map(|b| match &b.note {
            Some(note) if !note.is_empty() => {
                format!("{}\t{}\t{}", b.rel_path, b.tags.join(","), note)
            }
            _ => format!("{}\t{}", b.rel_path, b.tags.join(",")),
        })
        .collect::<Vec<_>>()
        .join("\n");
    let _ = fs::write(root.join(".navmarks"), content + "\n");
}

fn to_rel(abs: &Path, root: &Path) -> String {
    abs.strip_prefix(root)
        .map(|p| {
            let s = p.to_string_lossy().to_string();
            if s.is_empty() { ".".to_string() } else { s }
        })
        .unwrap_or_else(|_| abs.to_string_lossy().to_string())
}

fn detect_conventions(root: &Path) -> Vec<(&'static str, Vec<&'static str>)> {
    let conventions: &[(&str, &[&str])] = &[
        ("src",        &["source"]),
        ("lib",        &["lib"]),
        ("test",       &["tests"]),
        ("tests",      &["tests"]),
        ("spec",       &["tests"]),
        ("docs",       &["docs"]),
        ("doc",        &["docs"]),
        ("config",     &["config"]),
        ("configs",    &["config"]),
        ("scripts",    &["scripts"]),
        ("script",     &["scripts"]),
        ("api",        &["api"]),
        ("frontend",   &["frontend"]),
        ("web",        &["frontend"]),
        ("ui",         &["frontend"]),
        ("backend",    &["backend"]),
        ("pkg",        &["packages"]),
        ("packages",   &["packages"]),
        ("cmd",        &["cli"]),
        ("internal",   &["internal"]),
        ("examples",   &["examples"]),
        ("migrations", &["db"]),
        ("db",         &["db"]),
        ("contrib",    &["completions", "contrib"]),
        ("man",        &["docs"]),
        ("completions",&["completions"]),
        ("shell",      &["completions"]),
        ("bin",        &["scripts"]),
        ("plugin",     &["plugin"]),
        ("plugins",    &["plugin"]),
        ("example",    &["examples"]),
        ("bench",      &["benchmarks"]),
        ("benches",    &["benchmarks"]),
        ("include",    &["headers"]),
        ("vendor",     &["vendor"]),
        ("dist",       &["dist"]),
        ("templates",  &["templates"]),
        ("nix",        &["packaging", "nix"]),
        ("devtools",   &["scripts", "dev"]),
    ];

    conventions
        .iter()
        .filter(|(dir, _)| root.join(dir).is_dir())
        .map(|(dir, tags)| (*dir, tags.to_vec()))
        .collect()
}

fn main() {
    let cli = Cli::parse();
    let root = find_project_root();

    match &cli.command {
        Some(Commands::Mark { path, tag, note }) => {
            let abs = fs::canonicalize(path).unwrap_or_else(|_| PathBuf::from(path));
            let rel = to_rel(&abs, &root);

            let mut bookmarks = read_navmarks(&root);

            if let Some(existing) = bookmarks.iter_mut().find(|b| b.rel_path == rel) {
                for t in tag {
                    if !existing.tags.contains(t) {
                        existing.tags.push(t.clone());
                    }
                }
                if note.is_some() {
                    existing.note = note.clone();
                }
            } else {
                let tags = if tag.is_empty() {
                    vec!["general".to_string()]
                } else {
                    tag.clone()
                };
                bookmarks.push(Bookmark { abs_path: abs, rel_path: rel, tags, note: note.clone() });
            }

            write_navmarks(&root, &bookmarks);
        }
        Some(Commands::Remove { path }) => {
            let abs = fs::canonicalize(path).unwrap_or_else(|_| PathBuf::from(path));
            let rel = to_rel(&abs, &root);
            let bookmarks: Vec<_> = read_navmarks(&root)
                .into_iter()
                .filter(|b| b.rel_path != rel)
                .collect();
            write_navmarks(&root, &bookmarks);
        }
        Some(Commands::List) => {
            for b in read_navmarks(&root) {
                if b.abs_path.exists() {
                    let mut obj = json!({
                        "path": b.abs_path.to_string_lossy(),
                        "rel": b.rel_path,
                        "tags": b.tags.join(", ")
                    });
                    if let Some(note) = &b.note {
                        obj["note"] = json!(note);
                    }
                    println!("{}", obj);
                }
            }
        }
        Some(Commands::Init { shell: Some(shell) }) => {
            let script = match shell.as_str() {
                "zsh" | "bash" => r#"
function nav() {
    local target=$(command nav "$@")
    if [ -n "$target" ] && [ -d "$target" ]; then
        cd "$target"
    fi
}
"#,
                "fish" => r#"
function nav
    set target (command nav $argv)
    if test -n "$target" -a -d "$target"
        cd "$target"
    end
end
"#,
                _ => {
                    eprintln!("Unsupported shell: {}", shell);
                    std::process::exit(1);
                }
            };
            print!("{}", script);
        }
        Some(Commands::Init { shell: None }) => {
            let detected = detect_conventions(&root);

            if detected.is_empty() {
                eprintln!("No conventional directories found. Use `nav mark <path> -t <tag>` to add bookmarks manually.");
                std::process::exit(0);
            }

            let mut bookmarks = read_navmarks(&root);
            let existing_rels: Vec<String> = bookmarks.iter().map(|b| b.rel_path.clone()).collect();
            let mut added = 0;

            for (dir, tags) in &detected {
                if existing_rels.iter().any(|r| r == dir) {
                    continue;
                }
                let abs_path = root.join(dir);
                let tags: Vec<String> = tags.iter().map(|t| t.to_string()).collect();
                eprintln!("  + {}\t[{}]", dir, tags.join(", "));
                bookmarks.push(Bookmark { abs_path, rel_path: dir.to_string(), tags, note: None });
                added += 1;
            }

            if added == 0 {
                eprintln!("All detected directories already in .navmarks.");
            } else {
                write_navmarks(&root, &bookmarks);
                eprintln!("\nAdded {} bookmark(s) to .navmarks", added);
            }
        }
        None => {
            let bookmarks: Vec<Bookmark> = read_navmarks(&root)
                .into_iter()
                .filter(|b| b.abs_path.exists())
                .collect();

            if bookmarks.is_empty() {
                eprintln!("No bookmarks found. Run `nav init` to auto-detect, or `nav mark -t <tag>` to add one.");
                std::process::exit(0);
            }

            let mut input = String::new();
            for b in &bookmarks {
                let tag_str = b.tags.iter().map(|t| format!("#{}", t)).collect::<Vec<_>>().join(" ");
                input.push_str(&format!("[{}] {}\n", tag_str, b.abs_path.to_string_lossy()));
            }

            let mut fzf = match Command::new("fzf")
                .arg("--prompt=nav> ")
                .stdin(Stdio::piped())
                .stdout(Stdio::piped())
                .spawn()
            {
                Ok(child) => child,
                Err(_) => {
                    eprintln!("Error: fzf is not installed or not found in PATH.");
                    std::process::exit(1);
                }
            };

            if let Some(mut stdin) = fzf.stdin.take() {
                let _ = stdin.write_all(input.as_bytes());
            }
            let output = fzf.wait_with_output().expect("fzf failed");
            let selected = String::from_utf8_lossy(&output.stdout).trim().to_string();

            if !selected.is_empty() {
                let path = selected.find(']')
                    .map(|i| selected[i + 1..].trim().to_string())
                    .unwrap_or(selected);
                println!("{}", path);
            }
        }
    }
}
