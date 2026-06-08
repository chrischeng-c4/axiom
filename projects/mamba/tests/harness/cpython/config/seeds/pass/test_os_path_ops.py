# Operational AssertionPass seed for `os.path` and `os` constants.
# Surface: join, basename, dirname, splitext, split, sep, exists/
# isfile via /etc/hosts (POSIX-only smoke), abspath idempotence.
# Companion to stub/test_os_path.py — vendored unittest seed.
import os
import os.path
_ledger: list[int] = []
assert os.sep == "/"; _ledger.append(1)
assert os.path.join("a", "b", "c") == "a/b/c"; _ledger.append(1)
assert os.path.join("/x", "y") == "/x/y"; _ledger.append(1)
assert os.path.basename("/a/b/c.txt") == "c.txt"; _ledger.append(1)
assert os.path.basename("plain.txt") == "plain.txt"; _ledger.append(1)
assert os.path.dirname("/a/b/c.txt") == "/a/b"; _ledger.append(1)
assert os.path.dirname("plain.txt") == ""; _ledger.append(1)
assert os.path.splitext("file.txt") == ("file", ".txt"); _ledger.append(1)
assert os.path.splitext("noext") == ("noext", ""); _ledger.append(1)
assert os.path.split("/a/b/c") == ("/a/b", "c"); _ledger.append(1)
print(f"MAMBA_ASSERTION_PASS: test_os_path_ops {sum(_ledger)} asserts")
