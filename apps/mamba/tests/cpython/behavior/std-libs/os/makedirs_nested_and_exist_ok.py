# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "os"
# dimension = "behavior"
# case = "makedirs_nested_and_exist_ok"
# subject = "os.makedirs"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""os.makedirs: os.makedirs creates a full nested chain; re-creating raises FileExistsError by default but exist_ok=True suppresses it"""
import os
import tempfile

with tempfile.TemporaryDirectory() as base:
    # makedirs creates the full nested chain.
    deep = os.path.join(base, "dir1", "dir2", "dir3")
    os.makedirs(deep)
    assert os.path.isdir(deep), f"missing nested dir: {deep!r}"

    # Re-creating an existing directory raises FileExistsError by default.
    raised = False
    try:
        os.makedirs(deep)
    except FileExistsError:
        raised = True
    assert raised, "makedirs on existing dir should raise"

    # exist_ok=False is the explicit form of the default and still raises.
    raised = False
    try:
        os.makedirs(deep, exist_ok=False)
    except FileExistsError:
        raised = True
    assert raised, "exist_ok=False should raise"

    # exist_ok=True suppresses the error.
    os.makedirs(deep, exist_ok=True)

    # makedirs onto an existing regular file raises (it is not a dir).
    filepath = os.path.join(base, "plain.txt")
    with open(filepath, "w", encoding="utf-8") as f:
        f.write("abc")
    raised = False
    try:
        os.makedirs(filepath, exist_ok=True)
    except FileExistsError:
        raised = True
    assert raised, "makedirs onto a file should raise"
print("makedirs_nested_and_exist_ok OK")
