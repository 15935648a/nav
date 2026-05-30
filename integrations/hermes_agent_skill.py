import subprocess

def get_nav_bookmarks() -> str:
    """
    Execute 'nav list' to retrieve all user-curated directory bookmarks.
    Returns a JSONL string containing absolute paths and their semantic tags.
    """
    try:
        result = subprocess.run(["nav", "list"], capture_output=True, text=True, check=True)
        return result.stdout
    except FileNotFoundError:
        return "Error: 'nav' command not found. Is it installed in PATH?"
    except Exception as e:
        return f"Error running nav list: {e}"

def add_nav_bookmark(path: str, tags: list[str]) -> str:
    """
    Execute 'nav mark <path> -t <tag1> ...' to bookmark an important directory for the user.
    """
    cmd = ["nav", "mark", path]
    for tag in tags:
        cmd.extend(["-t", tag])
        
    try:
        subprocess.run(cmd, capture_output=True, text=True, check=True)
        return f"Successfully added bookmark for {path} with tags: {tags}"
    except Exception as e:
        return f"Error adding bookmark: {e}"
