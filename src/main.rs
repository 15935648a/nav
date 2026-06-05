use clap::{Parser, Subcommand};
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
    add       Bookmark a project directory (defaults to current dir)
    rm        Remove a bookmarked project
    list      Print all bookmarks
    edit      Open the bookmarks file in $EDITOR
    init      Generate shell init script: nav init <zsh|bash|fish>

  Run `nav` with no command to fuzzy-jump with fzf.
  Type a #tag in the picker to filter (e.g. #rust).

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
    /// Bookmark a project directory (defaults to current dir)
    Add {
        /// Path to bookmark (defaults to current directory)
        #[arg(default_value = ".")]
        path: String,
        /// Tags for the bookmark (can be used multiple times).
        /// If omitted, nav tries to auto-detect the language.
        #[arg(short, long)]
        tag: Vec<String>,
    },
    /// Remove a bookmarked project
    Rm {
        /// Path to remove (defaults to current directory)
        #[arg(default_value = ".")]
        path: String,
    },
    /// Print all bookmarks
    List,
    /// Open the bookmarks file in $EDITOR
    Edit,
    /// Generate shell init script: nav init <zsh|bash|fish>
    Init {
        /// Shell name (zsh, bash, fish)
        shell: String,
    },
}

struct Bookmark {
    path: String,
    tags: Vec<String>,
}

/// Global bookmarks file: $XDG_CONFIG_HOME/nav/marks or ~/.config/nav/marks
fn marks_file() -> PathBuf {
    let base = std::env::var_os("XDG_CONFIG_HOME")
        .map(PathBuf::from)
        .or_else(|| std::env::var_os("HOME").map(|h| PathBuf::from(h).join(".config")))
        .unwrap_or_else(|| PathBuf::from("."));
    base.join("nav").join("marks")
}

fn read_marks() -> Vec<Bookmark> {
    let content = fs::read_to_string(marks_file()).unwrap_or_default();
    content
        .lines()
        .filter(|l| {
            let t = l.trim();
            !t.is_empty() && !t.starts_with('#')
        })
        .map(|line| {
            let (path, tags_str) = match line.split_once('\t') {
                Some((p, t)) => (p.trim().to_string(), t.trim()),
                None => (line.trim().to_string(), ""),
            };
            let tags: Vec<String> = tags_str
                .split(',')
                .map(|t| t.trim().to_string())
                .filter(|t| !t.is_empty())
                .collect();
            Bookmark { path, tags }
        })
        .collect()
}

fn write_marks(bookmarks: &[Bookmark]) {
    let file = marks_file();
    if let Some(dir) = file.parent() {
        let _ = fs::create_dir_all(dir);
    }
    let content: String = bookmarks
        .iter()
        .map(|b| format!("{}\t{}", b.path, b.tags.join(",")))
        .collect::<Vec<_>>()
        .join("\n");
    let _ = fs::write(file, content + "\n");
}

/// Guess a language tag from marker files in the directory.
fn detect_language(dir: &Path) -> Option<&'static str> {
    let markers: &[(&str, &str)] = &[
        ("Cargo.toml", "rust"),
        ("go.mod", "go"),
        ("package.json", "node"),
        ("pyproject.toml", "python"),
        ("setup.py", "python"),
        ("requirements.txt", "python"),
        ("pom.xml", "java"),
        ("build.gradle", "java"),
        ("Gemfile", "ruby"),
        ("composer.json", "php"),
        ("CMakeLists.txt", "cpp"),
        ("Package.swift", "swift"),
        ("mix.exs", "elixir"),
    ];
    markers
        .iter()
        .find(|(file, _)| dir.join(file).exists())
        .map(|(_, lang)| *lang)
}

fn main() {
    let cli = Cli::parse();

    match &cli.command {
        Some(Commands::Add { path, tag }) => {
            let abs = fs::canonicalize(path).unwrap_or_else(|_| PathBuf::from(path));
            let abs_str = abs.to_string_lossy().to_string();

            let mut bookmarks = read_marks();

            let tags = if !tag.is_empty() {
                tag.clone()
            } else if let Some(lang) = detect_language(&abs) {
                vec![lang.to_string()]
            } else {
                vec!["general".to_string()]
            };

            if let Some(existing) = bookmarks.iter_mut().find(|b| b.path == abs_str) {
                for t in &tags {
                    if !existing.tags.contains(t) {
                        existing.tags.push(t.clone());
                    }
                }
                eprintln!("updated {}  [{}]", abs_str, existing.tags.join(", "));
            } else {
                eprintln!("added {}  [{}]", abs_str, tags.join(", "));
                bookmarks.push(Bookmark { path: abs_str, tags });
            }

            write_marks(&bookmarks);
        }
        Some(Commands::Rm { path }) => {
            let abs = fs::canonicalize(path).unwrap_or_else(|_| PathBuf::from(path));
            let abs_str = abs.to_string_lossy().to_string();
            let before = read_marks();
            let after: Vec<_> = before.into_iter().filter(|b| b.path != abs_str).collect();
            write_marks(&after);
        }
        Some(Commands::List) => {
            for b in read_marks() {
                let tag_str = b.tags.iter().map(|t| format!("#{}", t)).collect::<Vec<_>>().join(" ");
                println!("{}\t{}", tag_str, b.path);
            }
        }
        Some(Commands::Edit) => {
            let editor = std::env::var("EDITOR").unwrap_or_else(|_| "vi".to_string());
            let file = marks_file();
            if !file.exists() {
                if let Some(dir) = file.parent() {
                    let _ = fs::create_dir_all(dir);
                }
                let _ = fs::write(&file, "");
            }
            if let Err(e) = Command::new(&editor).arg(&file).status() {
                eprintln!("Error opening editor '{}': {}", editor, e);
                std::process::exit(1);
            }
        }
        Some(Commands::Init { shell }) => {
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
        None => {
            let bookmarks: Vec<Bookmark> = read_marks()
                .into_iter()
                .filter(|b| Path::new(&b.path).is_dir())
                .collect();

            if bookmarks.is_empty() {
                eprintln!("No bookmarks. Run `nav add` in a project to bookmark it.");
                std::process::exit(0);
            }

            let mut input = String::new();
            for b in &bookmarks {
                let tag_str = b.tags.iter().map(|t| format!("#{}", t)).collect::<Vec<_>>().join(" ");
                input.push_str(&format!("{}  {}\n", tag_str, b.path));
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
                // path is the last whitespace-separated field
                let path = selected.rsplit("  ").next().unwrap_or(&selected).trim();
                println!("{}", path);
            }
        }
    }
}
