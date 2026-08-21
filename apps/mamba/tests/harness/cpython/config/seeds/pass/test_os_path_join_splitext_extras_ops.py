# Operational AssertionPass seed for `os.path.join` and
# `os.path.splitext` edges not covered by `test_os_path_ops` or
# `test_os_path.py`. Surface: `os.path.join("a", "/b")` honors the
# absolute-path override — a later argument whose body starts with
# `/` discards the accumulated head. `os.path.join` accepts >2
# arguments and concatenates with the path separator. The
# single-segment form is a no-op. `os.path.splitext` keeps the
# inner dot of "file.tar.gz" inside the stem and returns ".gz" as
# the extension. A leading-dot filename (".hidden") and a trailing
# dot ("file.") return `""` as the extension — neither produces
# a phantom split. `os.path.basename` on a string with no slash
# returns the input unchanged; `dirname` returns `""`.
import os.path
_ledger: list[int] = []

# join: absolute-path override
assert os.path.join("a", "/b") == "/b"; _ledger.append(1)
assert os.path.join("/x/y", "/z") == "/z"; _ledger.append(1)
assert os.path.join("a/b/c", "/d") == "/d"; _ledger.append(1)

# join: 3+ arguments
assert os.path.join("a", "b", "c") == "a/b/c"; _ledger.append(1)
assert os.path.join("/a", "b", "c", "d") == "/a/b/c/d"; _ledger.append(1)

# join: single-arg no-op
assert os.path.join("solo") == "solo"; _ledger.append(1)
assert os.path.join("/abs") == "/abs"; _ledger.append(1)

# splitext: keeps inner dot
assert os.path.splitext("file.tar.gz") == ("file.tar", ".gz"); _ledger.append(1)
assert os.path.splitext("a.b.c.d") == ("a.b.c", ".d"); _ledger.append(1)

# splitext: leading-dot filename has no extension
assert os.path.splitext(".hidden") == (".hidden", ""); _ledger.append(1)
assert os.path.splitext(".bashrc") == (".bashrc", ""); _ledger.append(1)

# splitext: no extension
assert os.path.splitext("README") == ("README", ""); _ledger.append(1)
assert os.path.splitext("noext") == ("noext", ""); _ledger.append(1)

# basename / dirname: slashless input
assert os.path.basename("file.txt") == "file.txt"; _ledger.append(1)
assert os.path.dirname("file.txt") == ""; _ledger.append(1)

# basename: nested deep path
assert os.path.basename("/a/b/c/d/e.txt") == "e.txt"; _ledger.append(1)
assert os.path.dirname("/a/b/c/d/e.txt") == "/a/b/c/d"; _ledger.append(1)

print(f"MAMBA_ASSERTION_PASS: test_os_path_join_splitext_extras_ops {sum(_ledger)} asserts")
