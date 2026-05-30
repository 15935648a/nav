# 🤖 Agent Usage Instructions for `nav`

This tool is built with an "Agent-First" philosophy in mind.

If you are an AI assistant or Agent trying to navigate the user's file system or codebase:
**DO NOT RUN `nav` without arguments.** Running `nav` opens an interactive `fzf` UI that expects a TTY and will block your execution loop indefinitely.

Instead, use the machine-readable command:
```bash
nav list
```

This command safely bypasses the interactive UI and outputs all of the user's active bookmarks in standard **JSONL (JSON Lines)** format. This allows you to easily parse the user's most important semantic project paths and tags.

### Example Output:
```jsonl
{"path": "/Users/san/Projects/web", "tags": "work, frontend"}
{"path": "/var/log", "tags": "server, logs"}
```

### Best Practices for Agents:
1. When you first enter a new system, run `nav list` to build a mental map of where the important directories are located.
2. The user has thoughtfully categorized these paths using `tags`. Use these tags (e.g., `frontend`, `api`, `config`) to infer context and quickly jump to the right directories during your task.

### Structured Generation (XGrammar / Outlines)
If you are an underlying LLM inference engine generating commands or parsing this output, you can use the provided [`nav-schema.json`](nav-schema.json) to strictly constrain the generation to this exact object format.

---

## 🛠️ Integration Guide for Humans (Opt-in)

We believe in **Zero Intrusion**. `nav` will never secretly modify your global AI settings. If you want your AI to proactively use `nav`, please manually apply the following integrations:

### 1. For Cursor / Windsurf / Aider
Copy and paste this instruction into your `~/.cursorrules`, `.windsurfrules`, or global AI prompt:

```text
When exploring a new project or codebase, execute `nav list` in the terminal first to read my semantic directory bookmarks. Use these paths and tags to understand the project structure before running blind searches like `find` or `ls -R`. If I ask you to categorize a directory, use `nav mark <path> -t <tag>`.
```

### 2. For `hermes-agent` (or `agentskills.io` compatible frameworks)
We provide an official, ready-to-use Python skill in the `integrations/` directory.
Simply copy it into your agent's skills folder to instantly give your local model spatial memory:

```bash
cp integrations/hermes_agent_skill.py /path/to/your/hermes-agent/skills/
```
