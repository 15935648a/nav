use clap::{Parser, Subcommand};
use rusqlite::{params, Connection, Result};
use serde_json::json;
use std::io::Write;
use std::process::{Command, Stdio};

#[derive(Parser)]
#[command(
    name = "nav",
    disable_help_subcommand = true,
    help_template = "\
usage: nav [options] [command]

  COMMANDS
    mark      Mark current or specified path as a bookmark
    remove    Remove a bookmark manually
    list      List all active bookmarks in JSONL format (Agent-friendly mode)
    init      Generate shell initialization script

  OPTIONS
    --zoxide  Load zoxide history
    --help    Print this message
"
)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,

    /// Load zoxide history
    #[arg(long, env = "NAV_WITH_ZOXIDE")]
    zoxide: bool,
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
    },
    /// List all active bookmarks in JSONL format (Agent-friendly mode)
    List,
    /// Remove a bookmark manually
    Remove {
        /// Path to remove from bookmarks
        path: String,
    },
    /// Generate shell initialization script
    Init {
        /// Shell name (e.g., zsh, bash)
        shell: String,
    },
}

fn init_db() -> Result<Connection> {
    let db_path = format!(
        "{}/.nav_bookmarks.db",
        std::env::var("HOME").unwrap_or_else(|_| ".".to_string())
    );
    let conn = Connection::open(db_path)?;
    conn.execute(
        "CREATE TABLE IF NOT EXISTS bookmarks (
            id INTEGER PRIMARY KEY,
            path TEXT UNIQUE NOT NULL,
            tag TEXT,
            added_at DATETIME DEFAULT CURRENT_TIMESTAMP,
            score INTEGER DEFAULT 0
        )",
        [],
    )?;
    
    // Schema migration: add score column if it doesn't exist
    let mut has_score = false;
    {
        let mut stmt = conn.prepare("PRAGMA table_info(bookmarks)")?;
        let mut rows = stmt.query([])?;
        while let Some(row) = rows.next()? {
            let name: String = row.get(1)?;
            if name == "score" {
                has_score = true;
                break;
            }
        }
    }
    if !has_score {
        conn.execute("ALTER TABLE bookmarks ADD COLUMN score INTEGER DEFAULT 0", [])?;
    }
    Ok(conn)
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    let conn = init_db()?;

    match &cli.command {
        Some(Commands::Mark { path, tag }) => {
            let abs_path = std::fs::canonicalize(path)
                .map(|p| p.to_string_lossy().to_string())
                .unwrap_or_else(|_| path.to_string());

            // Read existing tags if any
            let existing_tag_str: Result<String, _> = conn.query_row(
                "SELECT tag FROM bookmarks WHERE path = ?1",
                params![abs_path],
                |row| row.get(0),
            );

            let mut all_tags: Vec<String> = existing_tag_str
                .unwrap_or_default()
                .split(',')
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty() && s != "general")
                .collect();

            for t in tag {
                if !all_tags.contains(t) {
                    all_tags.push(t.clone());
                }
            }

            let final_tags = if all_tags.is_empty() {
                "general".to_string()
            } else {
                all_tags.join(", ")
            };

            conn.execute(
                "INSERT INTO bookmarks (path, tag) VALUES (?1, ?2) 
                 ON CONFLICT(path) DO UPDATE SET tag = ?2",
                params![abs_path, final_tags],
            )?;
            // Rule of Silence: Do not print anything on success.
        }
        Some(Commands::Init { shell }) => {
            let script = match shell.as_str() {
                "zsh" | "bash" => {
                    r#"
function nav() {
    local target=$(command nav "$@")
    if [ -n "$target" ] && [ -d "$target" ]; then
        cd "$target"
    fi
}
"#
                }
                "fish" => {
                    r#"
function nav
    set target (command nav $argv)
    if test -n "$target" -a -d "$target"
        cd "$target"
    end
end
"#
                }
                _ => {
                    eprintln!("Unsupported shell: {}", shell);
                    std::process::exit(1);
                }
            };
            print!("{}", script);
        }
        Some(Commands::Remove { path }) => {
            let abs_path = std::fs::canonicalize(path)
                .map(|p| p.to_string_lossy().to_string())
                .unwrap_or_else(|_| path.to_string());
            conn.execute("DELETE FROM bookmarks WHERE path = ?1", params![abs_path])?;
        }
        Some(Commands::List) => {
            // Machine-readable output for AI Agents
            let mut stmt = conn.prepare("SELECT path, tag FROM bookmarks ORDER BY score DESC, added_at DESC")?;
            let bookmark_iter = stmt.query_map([], |row| {
                Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
            })?;
            for (path, tag) in bookmark_iter.flatten() {
                if std::path::Path::new(&path).exists() {
                    println!("{}", json!({ "path": path, "tags": tag }));
                }
            }
        }
        None => {
            let mut combined_input = String::new();
            let mut stmt = conn.prepare("SELECT path, tag FROM bookmarks ORDER BY score DESC, added_at DESC")?;
            let bookmark_iter = stmt.query_map([], |row| {
                Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
            })?;
            
            let mut paths_to_delete = Vec::new();

            for (path, tag) in bookmark_iter.flatten() {
                if std::path::Path::new(&path).exists() {
                    let formatted_tags: Vec<String> = tag.split(',').map(|t| format!("#{}", t.trim())).collect();
                    let tag_str = formatted_tags.join(" ");
                    combined_input.push_str(&format!("[{}] {}\n", tag_str, path));
                } else {
                    paths_to_delete.push(path);
                }
            }

            // Self-healing: Delete dead links from the database silently
            for dead_path in paths_to_delete {
                let _ = conn.execute("DELETE FROM bookmarks WHERE path = ?1", params![dead_path]);
            }
            
            if cli.zoxide {
                if let Ok(zoxide_output) = Command::new("zoxide").arg("query").arg("-l").output() {
                    for z in String::from_utf8_lossy(&zoxide_output.stdout).lines() {
                        if !z.is_empty() {
                            combined_input.push_str(&format!("[#zoxide] {}\n", z));
                        }
                    }
                }
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
                    eprintln!("nav requires fzf to work interactively. Please install it.");
                    std::process::exit(1);
                }
            };
                
            if let Some(mut stdin) = fzf.stdin.take() {
                let _ = stdin.write_all(combined_input.as_bytes());
            }
            let output = fzf.wait_with_output().expect("fzf failed");
            let selected = String::from_utf8_lossy(&output.stdout).trim().to_string();
            
            if !selected.is_empty() {
                let actual_path = if let Some(idx) = selected.find(']') {
                    selected[idx + 1..].trim().to_string()
                } else {
                    selected.clone()
                };

                // Increment score for SQL bookmarks (ignore zoxide entries)
                if !selected.starts_with("[#zoxide]") {
                    let _ = conn.execute("UPDATE bookmarks SET score = score + 1 WHERE path = ?1", params![actual_path]);
                }

                println!("{}", actual_path);
            }
        }
    }
    Ok(())
}
