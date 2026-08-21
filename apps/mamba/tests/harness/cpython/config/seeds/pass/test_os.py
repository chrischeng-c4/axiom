# Promoted from the upstream unittest port to an executable AssertionPass seed.
# Surface: os — POSIX constants, getcwd / getpid / getppid, mkdir / makedirs /
# rmdir, rename / remove, listdir, stat (st_size).
# os.environ is exposed as an empty dict-like stub on mamba today (HOME is not
# populated) and is intentionally NOT exercised here; tracked separately.
# os.path is covered by test_os_path (#3396).
import os
import tempfile
import shutil

_ledger: list[int] = []

# On POSIX, os.name is 'posix'
assert os.name == "posix", "os.name == 'posix'"
_ledger.append(1)

# Path separator on POSIX is '/'
assert os.sep == "/", "os.sep == '/'"
_ledger.append(1)

# Line separator on POSIX is '\n'
assert os.linesep == "\n", "os.linesep == '\\n'"
_ledger.append(1)

# PATH separator on POSIX is ':'
assert os.pathsep == ":", "os.pathsep == ':'"
_ledger.append(1)

# getcwd returns a non-empty absolute path
_cwd = os.getcwd()
assert isinstance(_cwd, str) and len(_cwd) > 0 and _cwd.startswith("/"), (
    "getcwd() returns a non-empty absolute POSIX path"
)
_ledger.append(1)

# getpid returns a positive int
_pid = os.getpid()
assert isinstance(_pid, int) and _pid > 0, "getpid() returns a positive int"
_ledger.append(1)

# getppid returns a positive int (any process has a parent)
_ppid = os.getppid()
assert isinstance(_ppid, int) and _ppid > 0, "getppid() returns a positive int"
_ledger.append(1)

# listdir('/') returns a non-empty list of strings
_entries = os.listdir("/")
assert isinstance(_entries, list) and len(_entries) > 0, (
    "listdir('/') returns a non-empty list"
)
_ledger.append(1)

# Set up a temp workspace for filesystem mutations
_d = tempfile.mkdtemp()

# mkdir creates a new directory
_sub = os.path.join(_d, "sub")
os.mkdir(_sub)
assert os.path.isdir(_sub), "os.mkdir creates a directory"
_ledger.append(1)

# makedirs creates an entire nested chain
_chain = os.path.join(_d, "a", "b", "c")
os.makedirs(_chain)
assert os.path.isdir(_chain), "os.makedirs creates a nested directory chain"
_ledger.append(1)

# rmdir removes an empty directory
os.rmdir(_sub)
assert not os.path.exists(_sub), "os.rmdir removes an empty directory"
_ledger.append(1)

# rename relocates a file
_src = os.path.join(_d, "src.txt")
_fw = open(_src, "w")
_fw.write("x")
_fw.close()
_dst = os.path.join(_d, "dst.txt")
os.rename(_src, _dst)
assert not os.path.exists(_src), "os.rename removes the source path"
_ledger.append(1)

assert os.path.exists(_dst), "os.rename creates the destination path"
_ledger.append(1)

# os.stat reports the file size of a 1-byte file as 1
_st = os.stat(_dst)
assert _st.st_size == 1, f"os.stat.st_size == 1, got {_st.st_size}"
_ledger.append(1)

# remove deletes a file
os.remove(_dst)
assert not os.path.exists(_dst), "os.remove deletes the file"
_ledger.append(1)

# Cleanup workspace
shutil.rmtree(_d)

print(f"MAMBA_ASSERTION_PASS: test_os {sum(_ledger)} asserts")
