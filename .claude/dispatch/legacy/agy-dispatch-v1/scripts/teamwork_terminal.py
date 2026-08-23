#!/usr/bin/env python3
"""Launch an interactive AGY teamwork-preview session in macOS Terminal."""
from __future__ import annotations

import argparse
import json
import os
import platform
import shlex
import shutil
import subprocess
import sys
from datetime import datetime, timezone
from pathlib import Path


TEMP_ROOT = Path("/tmp").resolve()


def agy_help() -> str:
    agy = shutil.which("agy")
    if not agy:
        return ""
    result = subprocess.run([agy, "--help"], text=True, capture_output=True, check=False)
    return result.stdout + result.stderr


def iterm_context() -> dict | None:
    if platform.system() != "Darwin" or not shutil.which("osascript"):
        return None
    script = '''tell application "iTerm2"
if not running then return ""
set currentWindow to current window
set currentSession to current session of current tab of currentWindow
return (name of currentWindow) & "|" & (tty of currentSession)
end tell'''
    result = subprocess.run(["osascript", "-e", script], text=True, capture_output=True, check=False)
    if result.returncode or not result.stdout.strip():
        return None
    name, tty = result.stdout.strip().split("|", 1)
    return {"window_name": name, "tty": tty}


def iterm_session_count() -> int:
    script = 'tell application "iTerm2" to return count of sessions of current tab of current window'
    result = subprocess.run(["osascript", "-e", script], text=True, capture_output=True, check=True)
    return int(result.stdout.strip())


def split_iterm_pane(command: str) -> None:
    before = iterm_session_count()
    script = f'''tell application "iTerm2"
set sourceSession to current session of current tab of current window
tell sourceSession
split vertically with default profile command "{apple_quote(command)}"
end tell
activate
end tell'''
    subprocess.run(["osascript", "-e", script], check=True)
    if iterm_session_count() > before:
        return
    fallback = 'tell application "System Events" to tell process "iTerm2" to click menu item "Split Vertically with Current Profile" of menu 1 of menu bar item "Shell" of menu bar 1'
    subprocess.run(["osascript", "-e", fallback], check=True)
    if iterm_session_count() <= before:
        raise SystemExit("iTerm2 did not create a pane; no interactive AGY session was launched")
    subprocess.run(["osascript", "-e", f'tell application "System Events" to tell process "iTerm2" to keystroke "{apple_quote(command)}"'], check=True)


def detect() -> dict:
    help_text = agy_help()
    agy = shutil.which("agy")
    return {
        "platform": platform.system(),
        "agy": agy,
        "interactive_supported": "--prompt-interactive" in help_text,
        "execution_modes": {"headless": "--print", "interactive": "--prompt-interactive"},
        "expect": shutil.which("expect"),
        "osascript": shutil.which("osascript"),
        "terminal_app": Path("/System/Applications/Utilities/Terminal.app").exists() or Path("/Applications/Utilities/Terminal.app").exists(),
        "iterm2_current": iterm_context(),
    }


def require_environment() -> None:
    facts = detect()
    missing = [name for name, value in facts.items() if name in ("agy", "expect", "osascript") and not value]
    if facts["platform"] != "Darwin" or not facts["interactive_supported"] or not facts["terminal_app"] or missing:
        raise SystemExit(f"interactive teamwork unavailable: {json.dumps(facts)}")


def load_profile(path: str) -> dict:
    profile = json.loads(Path(path).read_text())
    for key in ("root", "state_dir", "agy_project_id"):
        if key not in profile:
            raise SystemExit(f"profile missing {key}")
    if not Path(profile["root"]).is_dir():
        raise SystemExit(f"root is not a directory: {profile['root']}")
    state_dir = Path(profile["state_dir"]).resolve()
    if state_dir != TEMP_ROOT and not state_dir.is_relative_to(TEMP_ROOT):
        raise SystemExit("state_dir must be under /tmp/agy-dispatch")
    profile["state_dir"] = str(state_dir)
    return profile


def apple_quote(value: str) -> str:
    return value.replace("\\", "\\\\").replace('"', '\\"')


def tcl_quote(value: str) -> str:
    return "{" + value.replace("\\", "\\\\").replace("}", "\\}") + "}"


def launch(profile: dict, prompt_file: Path, dry_run: bool, target: str) -> None:
    if not prompt_file.is_file():
        raise SystemExit(f"prompt file does not exist: {prompt_file}")
    prompt = prompt_file.read_text()
    if not prompt.lstrip().startswith("/teamwork-preview"):
        raise SystemExit("teamwork prompt must begin with /teamwork-preview")
    state_dir = Path(profile["state_dir"]) / "interactive"
    state_dir.mkdir(parents=True, exist_ok=True)
    stamp = datetime.now(timezone.utc).strftime("%Y%m%dT%H%M%SZ")
    staged_prompt = state_dir / f"teamwork-{stamp}.prompt"
    expect_script = state_dir / f"teamwork-{stamp}.expect"
    launch_log = state_dir / f"teamwork-{stamp}.log"
    staged_prompt.write_text(prompt)
    agy = shutil.which("agy")
    expect_script.write_text(f'''#!/usr/bin/expect -f
set timeout -1
set prompt_file {tcl_quote(str(staged_prompt))}
log_file -a {tcl_quote(str(launch_log))}
set fh [open $prompt_file r]
set prompt [read $fh]
close $fh
spawn -noecho {tcl_quote(agy)} --project {tcl_quote(profile["agy_project_id"])} --prompt-interactive --model {tcl_quote(profile.get("model", "gemini-3.6-flash-high"))} --effort high
after 2000
send -- "$prompt\\r"
interact
''')
    expect_script.chmod(0o700)
    terminal_command = f"cd {shlex.quote(profile['root'])} && /usr/bin/expect {shlex.quote(str(expect_script))}; status=$?; printf '\\n[AGY teamwork launcher exited: %s]\\n' \"$status\"; exec /bin/zsh -i"
    if dry_run:
        print(json.dumps({"target": target, "iterm2_current": iterm_context(), "command": terminal_command}, indent=2))
        return
    use_iterm = target == "iterm2" or (target == "auto" and iterm_context() is not None)
    if use_iterm:
        split_iterm_pane(terminal_command)
        print(f"launched interactive teamwork in a new iTerm2 pane; log at {launch_log}")
    else:
        subprocess.run(["osascript", "-e", f'tell application "Terminal" to do script "{apple_quote(terminal_command)}"', "-e", 'tell application "Terminal" to activate'], check=True)
        print(f"launched interactive teamwork Terminal window; log at {launch_log}")


def main() -> None:
    parser = argparse.ArgumentParser()
    sub = parser.add_subparsers(dest="verb", required=True)
    sub.add_parser("detect")
    launch_parser = sub.add_parser("launch")
    launch_parser.add_argument("profile")
    launch_parser.add_argument("prompt_file")
    launch_parser.add_argument("--dry-run", action="store_true")
    launch_parser.add_argument("--target", choices=("auto", "iterm2", "terminal"), default="auto")
    args = parser.parse_args()
    if args.verb == "detect":
        print(json.dumps(detect(), indent=2))
        return
    require_environment()
    launch(load_profile(args.profile), Path(args.prompt_file), args.dry_run, args.target)


if __name__ == "__main__":
    main()
