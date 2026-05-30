# Agent Usage Guide

**Do not run `nav` without arguments.** It opens an interactive fzf UI that expects a TTY and will block indefinitely.

Use the machine-readable command instead:

```bash
nav list
```

This outputs all bookmarks defined in the repo's `.navmarks` file as JSONL:

```jsonl
{"path": "/Users/san/Projects/myapp/src/api", "rel": "src/api", "tags": "api, entrypoint", "note": "main event loop, start here"}
{"path": "/Users/san/Projects/myapp/src/auth", "rel": "src/auth", "tags": "auth, security", "note": "JWT validation — touch carefully"}
{"path": "/Users/san/Projects/myapp/tests", "rel": "tests", "tags": "tests"}
```

The `note` field is optional. When present, it contains human-written context about the path — use it to understand intent and caveats before diving in.

**Best practices:**
1. Run `nav list` when entering a project to get a semantic map of the important directories.
2. Use the tags (`api`, `frontend`, `tests`, etc.) to infer context before running broad searches like `find` or `ls -R`.
3. When a `note` is present, read it before editing files in that path.
4. To bookmark a path on the user's behalf: `nav mark <path> -t <tag> -n "context note"`.

## .navmarks format

The `.navmarks` file lives at the repo root and is meant to be committed to git:

```
src/api       api,entrypoint    main event loop, start here
src/auth      auth,security     JWT validation — touch carefully
src/frontend  frontend,react
tests         tests
```

Each line is `<path>\t<tag1,tag2>\t<optional note>`. Paths are relative to the git root. The note column is freetext and may contain spaces.

---

## Integrations

### Cursor / Windsurf / Aider

Add to `~/.cursorrules`, `.windsurfrules`, or your global AI prompt:

```
When exploring a project, run `nav list` first to read the semantic directory bookmarks defined in .navmarks. Use the paths and tags to understand the project structure before running broad searches like `find` or `ls -R`. To bookmark a directory: `nav mark <path> -t <tag>`.
```

### hermes-agent / agentskills.io

```bash
cp integrations/hermes_agent_skill.py /path/to/your/hermes-agent/skills/
```
