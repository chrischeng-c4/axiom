# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "shlex"
# dimension = "real_world"
# case = "join_split_command_roundtrip"
# subject = "shlex.join"
# kind = "semantic"
# xfail = "round-trip relies on shlex.split processing the quotes shlex.join emits; mamba split does not process quotes (repo-memory project_mamba_stdlib_stub_audit_2026_05_26)"
# mem_carveout = ""
# source = "Lib/test/test_shlex.py"
# status = "filled"
# ///
"""shlex.join: a build-tool argv with a spaced path round-trips: shlex.split(shlex.join(argv)) reproduces the original argv list"""
import shlex

# A realistic compiler invocation whose -o target lives under a path with a
# space — exactly the case a naive " ".join() would corrupt and shlex.join
# protects by quoting.
argv = ["cc", "-O2", "-o", "out dir/app", "main.c", "lib util.c"]
command_line = shlex.join(argv)
# The spaced arguments must come back quoted so a shell sees them as one word.
assert "'out dir/app'" in command_line, command_line
assert shlex.split(command_line) == argv, (command_line, shlex.split(command_line))
print("join_split_command_roundtrip OK")
