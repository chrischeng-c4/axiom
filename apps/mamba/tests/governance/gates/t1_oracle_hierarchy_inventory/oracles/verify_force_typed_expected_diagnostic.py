import hashlib
import json
import os
import pathlib
import sys


def find_repo_root() -> pathlib.Path:
    script_path = pathlib.Path(__file__).resolve()
    curr = script_path.parent
    while curr != curr.parent:
        if (curr / "apps" / "mamba").is_dir() and (
            (curr / "aw.toml").exists() or (curr / ".git").exists()
        ):
            return curr
        curr = curr.parent
    raise RuntimeError(
        "Repository root containing 'apps/mamba' and ('aw.toml' or '.git') not found"
    )



def main() -> None:
    if len(sys.argv) != 2:
        sys.exit(1)

    arg = sys.argv[1]

    # Reject backslashes
    if "\\" in arg:
        sys.exit(1)

    # Reject absolute paths
    if os.path.isabs(arg) or arg.startswith("/") or pathlib.Path(arg).is_absolute():
        sys.exit(1)

    # Reject '.', '..', and empty path segments
    parts = arg.split("/")
    if any(p in ("", ".", "..") for p in parts):
        sys.exit(1)

    script_path = pathlib.Path(__file__).resolve()
    json_path = script_path.parent / "force_typed_expected_diagnostics.json"

    if not json_path.is_file():
        sys.exit(1)

    try:
        with open(json_path, "r", encoding="utf-8") as f:
            data = json.load(f)
    except Exception:
        sys.exit(1)

    if not isinstance(data, dict):
        sys.exit(1)

    if arg not in data:
        sys.exit(1)

    rec = data[arg]
    if not isinstance(rec, dict):
        sys.exit(1)

    required_fields = {
        "fixture_sha256",
        "diagnostic_class",
        "diagnostic_span",
        "message_anchor",
    }
    if set(rec.keys()) != required_fields:
        sys.exit(1)

    if not all(isinstance(v, str) for v in rec.values()):
        sys.exit(1)

    repo_root = find_repo_root().resolve()
    fixture_path = (repo_root / arg).resolve()

    # Ensure fixture_path does not escape repo_root
    try:
        fixture_path.relative_to(repo_root)
    except ValueError:
        sys.exit(1)

    if not fixture_path.is_file():
        sys.exit(1)

    hasher = hashlib.sha256()
    try:
        with open(fixture_path, "rb") as f:
            while chunk := f.read(65536):
                hasher.update(chunk)
    except Exception:
        sys.exit(1)

    computed_sha = hasher.hexdigest().lower()
    pinned_sha = rec["fixture_sha256"].lower()

    if computed_sha != pinned_sha:
        sys.exit(1)

    output = (
        f"EXPECTED_FORCE_TYPED_DIAGNOSTIC "
        f"class={rec['diagnostic_class']} "
        f"span={rec['diagnostic_span']} "
        f"message={rec['message_anchor']}"
    )
    print(output)


if __name__ == "__main__":
    main()
