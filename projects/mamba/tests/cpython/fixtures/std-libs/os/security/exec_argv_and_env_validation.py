# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "os"
# dimension = "security"
# case = "exec_argv_and_env_validation"
# subject = "os.execve"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_os.py"
# status = "filled"
# ///
"""os.execve: the exec* family validates arguments before replacing the process: empty argv -> ValueError, missing program -> OSError, and env keys/values with embedded NUL or '=' in the key -> ValueError"""
import os
import sys

# Every call below is rejected during argument validation, so the process is
# never actually replaced.

# os.execv requires a non-empty argv whose first element is non-empty.
for bad_argv in ((), [], ("",), [""]):
    raised = False
    try:
        os.execv("dummy", bad_argv)
    except ValueError:
        raised = True
    assert raised, f"execv({bad_argv!r}) should raise ValueError"

# os.execvpe enforces the same argv rule.
for bad_argv in ([], [""]):
    raised = False
    try:
        os.execvpe("dummy", bad_argv, {})
    except ValueError:
        raised = True
    assert raised, f"execvpe({bad_argv!r}) should raise ValueError"

# os.execvpe on a program that cannot be found raises OSError
# (FileNotFoundError is an OSError subclass).
raised = False
try:
    os.execvpe("no_such_program_xyzzy-", ["no_such_program_xyzzy-"], None)
except OSError:
    raised = True
assert raised, "execvpe(missing program) should raise OSError"

# os.execve rejects environment keys/values containing embedded NUL or '='
# in the key, with ValueError, before exec is attempted.
args = [sys.executable, "-c", "pass"]
for key, value in (
    ("FRUIT\x00VEGETABLE", "cabbage"),         # NUL in key
    ("FRUIT", "orange\x00VEGETABLE=cabbage"),  # NUL in value
    ("FRUIT=ORANGE", "lemon"),                 # '=' in key
):
    env = os.environ.copy()
    env[key] = value
    raised = False
    try:
        os.execve(args[0], args, env)
    except ValueError:
        raised = True
    assert raised, f"execve with bad env {key!r} should raise ValueError"
print("exec_argv_and_env_validation OK")
