# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "code"
# dimension = "real_world"
# case = "repl_session_drives_console"
# subject = "code.InteractiveConsole"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""code.InteractiveConsole: a captured REPL session drives one InteractiveConsole over a shared namespace: feed a multi-line def then call it, an expression statement, a name rebind, and a deliberate error line; assert the namespace state, push() return values, and that the error was reported to stderr rather than raised"""
import code
import io
import contextlib

# A single console over a namespace the host keeps a handle to, like an
# embedded REPL/sandbox driving user-typed lines one at a time.
ns = {"seed": 3}
console = code.InteractiveConsole(ns)

err = io.StringIO()
with contextlib.redirect_stderr(err):
    # 1. A multi-line def: the header is incomplete, body line stays buffered,
    #    and the blank line completes the block.
    assert console.push("def twice(n):") is True, "def header incomplete"
    assert console.push("    return n * 2") is True, "indented body still buffering"
    assert console.push("") is False, "blank line completes the def"

    # 2. Call the freshly defined function, binding into the shared namespace.
    assert console.push("doubled = twice(seed)") is False, "assignment completes"

    # 3. A plain expression statement is complete (its value is discarded).
    assert console.push("twice(10) + 1") is False, "expression statement completes"

    # 4. Rebind an existing name.
    assert console.push("seed = seed + 4") is False, "rebind completes"

    # 5. A line that raises at runtime: the error is reported, not propagated,
    #    and push() still returns False (the statement was syntactically whole).
    assert console.push("boom = 1 / 0") is False, "erroring line still completes"

assert ns["doubled"] == 6, f"twice(3) == 6, got {ns['doubled']!r}"
assert ns["seed"] == 7, f"seed rebound to 7, got {ns['seed']!r}"
assert "boom" not in ns, "the failed assignment left no binding"
assert "ZeroDivisionError" in err.getvalue(), "runtime error surfaced on stderr"

print("repl_session_drives_console OK")
