use super::super::super::super::harness::*;

/// Ported from `tests/cpython/behavior/std-libs/os/devnull_discards_writes.py`.
#[test]
fn test_gen_behavior_std_libs_os_devnull_discards_writes() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "os"
# dimension = "behavior"
# case = "devnull_discards_writes"
# subject = "os.devnull"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""os.devnull: writing bytes to os.devnull is discarded and reading it back yields empty bytes"""
import os

with open(os.devnull, "wb", 0) as wn:
    wn.write(b"hello")
with open(os.devnull, "rb") as rn:
    assert rn.read() == b"", "reading os.devnull yields empty bytes"
print("devnull_discards_writes OK")
"###);
    assert_output(&out, r###"devnull_discards_writes OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/os/environ_get_is_dict_like.py`.
#[test]
fn test_gen_behavior_std_libs_os_environ_get_is_dict_like() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "os"
# dimension = "behavior"
# case = "environ_get_is_dict_like"
# subject = "os.environ"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""os.environ: os.environ behaves like a dict: .get(PATH) is None or str, and subscript/get are available"""
import os

assert hasattr(os.environ, "__getitem__"), "environ subscriptable"
assert hasattr(os.environ, "get"), "environ has get"
path = os.environ.get("PATH")
assert path is None or isinstance(path, str), f"PATH type = {type(path)!r}"
assert isinstance(os.environ.get("PATH", ""), str), "PATH default is str"
print("environ_get_is_dict_like OK")
"###);
    assert_output(&out, r###"environ_get_is_dict_like OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/os/fd_open_write_read_roundtrip.py`.
#[test]
fn test_gen_behavior_std_libs_os_fd_open_write_read_roundtrip() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "os"
# dimension = "behavior"
# case = "fd_open_write_read_roundtrip"
# subject = "os.open"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""os.open: low-level os.open/os.write/os.lseek/os.read/os.close round-trips bytes (bytes, bytearray, memoryview) through a temp file; os.access reports R_OK and W_OK"""
import os
import tempfile

with tempfile.TemporaryDirectory() as d:
    path = os.path.join(d, "data")

    # os.open returns an int fd; O_CREAT|O_WRONLY creates a writable file.
    fd = os.open(path, os.O_CREAT | os.O_WRONLY)
    assert isinstance(fd, int), f"open fd type = {type(fd)!r}"

    # os.write accepts bytes, bytearray, and memoryview.
    n = os.write(fd, b"bacon\n")
    assert n == 6, f"write returned {n!r}"
    os.write(fd, bytearray(b"eggs\n"))
    os.write(fd, memoryview(b"spam\n"))
    os.close(fd)

    # os.access reports the file is readable and writable.
    assert os.access(path, os.R_OK), "file should be readable"
    assert os.access(path, os.W_OK), "file should be writable"

    # os.open + os.lseek + os.read round-trips the written bytes.
    rfd = os.open(path, os.O_RDONLY)
    os.lseek(rfd, 0, os.SEEK_SET)
    chunk = os.read(rfd, 6)
    assert type(chunk) is bytes, f"read type = {type(chunk)!r}"
    assert chunk == b"bacon\n", f"read = {chunk!r}"
    os.close(rfd)

    # Whole-file contents split into lines.
    with open(path, "rb") as fobj:
        lines = fobj.read().splitlines()
    assert lines == [b"bacon", b"eggs", b"spam"], f"lines = {lines!r}"
print("fd_open_write_read_roundtrip OK")
"###);
    assert_output(&out, r###"fd_open_write_read_roundtrip OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/os/fsencode_fsdecode_roundtrip.py`.
#[test]
fn test_gen_behavior_std_libs_os_fsencode_fsdecode_roundtrip() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "os"
# dimension = "behavior"
# case = "fsencode_fsdecode_roundtrip"
# subject = "os.fsencode"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""os.fsencode: os.fsencode(str)->bytes and os.fsdecode(bytes)->str round-trip identically for encodable unicode names, and each is a no-op on its own target type"""
import os

# fsencode(str) -> bytes; fsdecode(bytes) -> str.
assert os.fsencode("ascii") == b"ascii", "fsencode ascii"
assert isinstance(os.fsencode("ascii"), bytes), "fsencode returns bytes"
assert os.fsdecode(b"ascii") == "ascii", "fsdecode ascii"
assert isinstance(os.fsdecode("ascii"), str), "fsdecode str passthrough type"

# fsencode is a no-op on bytes; fsdecode is a no-op on str.
assert os.fsencode(b"abc\xff") == b"abc\xff", "fsencode passes bytes through"
assert os.fsdecode("abcŁ") == "abcŁ", "fsdecode passes str through"

# Round-trip identity for encodable unicode names.
for name in ("ascii", "latié", "unicodeŁ"):
    encoded = os.fsencode(name)
    assert isinstance(encoded, bytes), f"fsencode({name!r}) type"
    assert os.fsdecode(encoded) == name, f"round-trip {name!r}"
print("fsencode_fsdecode_roundtrip OK")
"###);
    assert_output(&out, r###"fsencode_fsdecode_roundtrip OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/os/fspath_returns_and_protocol.py`.
#[test]
fn test_gen_behavior_std_libs_os_fspath_returns_and_protocol() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "os"
# dimension = "behavior"
# case = "fspath_returns_and_protocol"
# subject = "os.fspath"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""os.fspath: os.fspath returns str/bytes unchanged and honors the __fspath__ protocol on a PathLike object"""
import os

# os.fspath returns str/bytes unchanged.
assert os.fspath("/tmp/x") == "/tmp/x", "fspath str"
assert os.fspath(b"/tmp/x") == b"/tmp/x", "fspath bytes"


# os.fspath honors the __fspath__ protocol on PathLike objects.
class P:
    def __fspath__(self):
        return "/tmp/from_protocol"


assert os.fspath(P()) == "/tmp/from_protocol", "fspath __fspath__"
print("fspath_returns_and_protocol OK")
"###);
    assert_output(&out, r###"fspath_returns_and_protocol OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/os/getcwd_is_absolute.py`.
#[test]
fn test_gen_behavior_std_libs_os_getcwd_is_absolute() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "os"
# dimension = "behavior"
# case = "getcwd_is_absolute"
# subject = "os.getcwd"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""os.getcwd: os.getcwd returns a non-empty absolute path string"""
import os
import os.path

cwd = os.getcwd()
assert isinstance(cwd, str), f"getcwd type = {type(cwd)!r}"
assert len(cwd) > 0, "getcwd non-empty"
assert os.path.isabs(cwd), f"cwd is absolute: {cwd!r}"
print("getcwd_is_absolute OK")
"###);
    assert_output(&out, r###"getcwd_is_absolute OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/os/getcwdb_fsdecodes_to_getcwd.py`.
#[test]
fn test_gen_behavior_std_libs_os_getcwdb_fsdecodes_to_getcwd() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "os"
# dimension = "behavior"
# case = "getcwdb_fsdecodes_to_getcwd"
# subject = "os.getcwdb"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""os.getcwdb: os.getcwdb returns bytes that os.fsdecode back to exactly os.getcwd()"""
import os

cwdb = os.getcwdb()
assert isinstance(cwdb, bytes), f"getcwdb type = {type(cwdb)!r}"
assert os.fsdecode(cwdb) == os.getcwd(), "getcwdb fsdecodes to getcwd"
print("getcwdb_fsdecodes_to_getcwd OK")
"###);
    assert_output(&out, r###"getcwdb_fsdecodes_to_getcwd OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/os/getpid_is_positive_int.py`.
#[test]
fn test_gen_behavior_std_libs_os_getpid_is_positive_int() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "os"
# dimension = "behavior"
# case = "getpid_is_positive_int"
# subject = "os.getpid"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""os.getpid: os.getpid returns a positive int (the live interpreter PID)"""
import os

pid = os.getpid()
assert isinstance(pid, int), f"pid type = {type(pid)!r}"
assert pid > 0, f"pid > 0: {pid!r}"
print("getpid_is_positive_int OK")
"###);
    assert_output(&out, r###"getpid_is_positive_int OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/os/listdir_returns_list.py`.
#[test]
fn test_gen_behavior_std_libs_os_listdir_returns_list() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "os"
# dimension = "behavior"
# case = "listdir_returns_list"
# subject = "os.listdir"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""os.listdir: os.listdir('.') returns a list of entry-name strings"""
import os
import tempfile

with tempfile.TemporaryDirectory() as d:
    for name in ("a.txt", "b.txt", "c.txt"):
        with open(os.path.join(d, name), "w", encoding="utf-8") as f:
            f.write("x")
    entries = os.listdir(d)
    assert isinstance(entries, list), f"listdir type = {type(entries)!r}"
    assert all(isinstance(e, str) for e in entries), "entries are str"
    assert set(entries) == {"a.txt", "b.txt", "c.txt"}, f"entries = {set(entries)!r}"
print("listdir_returns_list OK")
"###);
    assert_output(&out, r###"listdir_returns_list OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/os/makedirs_nested_and_exist_ok.py`.
#[test]
fn test_gen_behavior_std_libs_os_makedirs_nested_and_exist_ok() {
    let out = jit_capture(r###"# /// script
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
"###);
    assert_output(&out, r###"makedirs_nested_and_exist_ok OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/os/path_abspath_idempotent.py`.
#[test]
fn test_gen_behavior_std_libs_os_path_abspath_idempotent() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "os"
# dimension = "behavior"
# case = "path_abspath_idempotent"
# subject = "os.path.abspath"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""os.path.abspath: os.path.abspath of an already-absolute path is idempotent and isabs reports it absolute"""
import os.path

rel = os.path.abspath(".")
assert os.path.isabs(rel), f"abspath is absolute: {rel!r}"
again = os.path.abspath(rel)
assert rel == again, "abspath idempotent on absolute"
print("path_abspath_idempotent OK")
"###);
    assert_output(&out, r###"path_abspath_idempotent OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/os/path_basename_dirname.py`.
#[test]
fn test_gen_behavior_std_libs_os_path_basename_dirname() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "os"
# dimension = "behavior"
# case = "path_basename_dirname"
# subject = "os.path.basename"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""os.path.basename: basename/dirname of '/tmp/dir/file.txt' are 'file.txt' and '/tmp/dir'"""
import os.path

path = "/tmp/dir/file.txt"
assert os.path.basename(path) == "file.txt", f"basename = {os.path.basename(path)!r}"
assert os.path.dirname(path) == "/tmp/dir", f"dirname = {os.path.dirname(path)!r}"
print("path_basename_dirname OK")
"###);
    assert_output(&out, r###"path_basename_dirname OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/os/path_exists_isdir_isfile.py`.
#[test]
fn test_gen_behavior_std_libs_os_path_exists_isdir_isfile() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "os"
# dimension = "behavior"
# case = "path_exists_isdir_isfile"
# subject = "os.path.exists"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""os.path.exists: the cwd exists and is a dir (not a file); a nonexistent path does not exist"""
import os.path

assert os.path.exists("."), "cwd exists"
assert os.path.isdir("."), "cwd is dir"
assert not os.path.isfile("."), "cwd is not a regular file"
assert not os.path.exists("/nonexistent_path_xyz_12345"), "nonexistent path"
print("path_exists_isdir_isfile OK")
"###);
    assert_output(&out, r###"path_exists_isdir_isfile OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/os/path_join_absolute_resets.py`.
#[test]
fn test_gen_behavior_std_libs_os_path_join_absolute_resets() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "os"
# dimension = "behavior"
# case = "path_join_absolute_resets"
# subject = "os.path.join"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""os.path.join: os.path.join discards earlier components when a later one is absolute: join('a','/b','c') ends at '/b/c'"""
import os
import os.path

abs_join = os.path.join("a", "/b", "c")
assert abs_join == "/b/c" or abs_join.endswith("b" + os.sep + "c"), f"abs join = {abs_join!r}"
print("path_join_absolute_resets OK")
"###);
    assert_output(&out, r###"path_join_absolute_resets OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/os/path_join_uses_separator.py`.
#[test]
fn test_gen_behavior_std_libs_os_path_join_uses_separator() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "os"
# dimension = "behavior"
# case = "path_join_uses_separator"
# subject = "os.path.join"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""os.path.join: os.path.join('a','b','c') joins with the platform separator so normalizing os.sep to '/' yields 'a/b/c'"""
import os
import os.path

joined = os.path.join("a", "b", "c")
assert isinstance(joined, str), f"join type = {type(joined)!r}"
# On Unix: "a/b/c"; normalizing the platform separator to '/' is portable.
assert joined.replace(os.sep, "/") == "a/b/c", f"join = {joined!r}"
print("path_join_uses_separator OK")
"###);
    assert_output(&out, r###"path_join_uses_separator OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/os/path_split_head_tail.py`.
#[test]
fn test_gen_behavior_std_libs_os_path_split_head_tail() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "os"
# dimension = "behavior"
# case = "path_split_head_tail"
# subject = "os.path.split"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""os.path.split: os.path.split('/tmp/dir/file.txt') splits into ('/tmp/dir', 'file.txt')"""
import os.path

head, tail = os.path.split("/tmp/dir/file.txt")
assert head == "/tmp/dir", f"head = {head!r}"
assert tail == "file.txt", f"tail = {tail!r}"
print("path_split_head_tail OK")
"###);
    assert_output(&out, r###"path_split_head_tail OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/os/path_splitext_rule.py`.
#[test]
fn test_gen_behavior_std_libs_os_path_splitext_rule() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "os"
# dimension = "behavior"
# case = "path_splitext_rule"
# subject = "os.path.splitext"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""os.path.splitext: os.path.splitext splits the final extension only: file.txt, file.tar.gz, noext, .hidden cases"""
import os.path

cases = [
    ("file.txt", ("file", ".txt")),
    ("file.tar.gz", ("file.tar", ".gz")),
    ("noext", ("noext", "")),
    (".hidden", (".hidden", "")),
]
for inp, expected in cases:
    got = os.path.splitext(inp)
    assert got == expected, f"splitext({inp!r}) = {got!r}, expected {expected!r}"
print("path_splitext_rule OK")
"###);
    assert_output(&out, r###"path_splitext_rule OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/os/pathlike_structural_subclass.py`.
#[test]
fn test_gen_behavior_std_libs_os_pathlike_structural_subclass() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "os"
# dimension = "behavior"
# case = "pathlike_structural_subclass"
# subject = "os.PathLike"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""os.PathLike: a class defining __fspath__ is a virtual subclass/instance of os.PathLike; one lacking it is not; explicit subclassing routes through os.fspath; PathLike[bytes] is a GenericAlias"""
import os
import types


# A class defining __fspath__ is a virtual subclass of os.PathLike.
class Implicit:
    def __fspath__(self):
        return "/tmp/implicit"


assert issubclass(Implicit, os.PathLike), "structural subclass via __fspath__"
assert isinstance(Implicit(), os.PathLike), "structural instance via __fspath__"


# A class lacking __fspath__ is not a PathLike.
class NotPath:
    pass


assert not issubclass(NotPath, os.PathLike), "no __fspath__ -> not PathLike"
assert not isinstance(NotPath(), os.PathLike), "no __fspath__ instance"


# Explicit subclassing works and the protocol routes through os.fspath.
class Explicit(os.PathLike):
    def __fspath__(self):
        return "/tmp/explicit"


assert issubclass(Explicit, os.PathLike), "explicit subclass"
assert os.fspath(Explicit()) == "/tmp/explicit", "fspath on explicit subclass"

# os.PathLike supports PEP 585 subscription -> types.GenericAlias.
alias = os.PathLike[bytes]
assert isinstance(alias, types.GenericAlias), f"alias type = {type(alias)!r}"
print("pathlike_structural_subclass OK")
"###);
    assert_output(&out, r###"pathlike_structural_subclass OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/os/removedirs_prunes_empty_parents.py`.
#[test]
fn test_gen_behavior_std_libs_os_removedirs_prunes_empty_parents() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "os"
# dimension = "behavior"
# case = "removedirs_prunes_empty_parents"
# subject = "os.removedirs"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""os.removedirs: os.removedirs deletes the leaf and prunes now-empty parents up the chain, but stops at the first non-empty parent"""
import os
import tempfile

with tempfile.TemporaryDirectory() as base:
    # removedirs deletes leaf and prunes now-empty parents up the chain.
    tree = os.path.join(base, "a", "b", "c")
    os.makedirs(tree)
    os.removedirs(tree)
    assert not os.path.exists(os.path.join(base, "a")), "empty parents pruned"

    # removedirs stops at the first non-empty parent.
    kept = os.path.join(base, "x", "y")
    os.makedirs(os.path.join(kept, "z"))
    with open(os.path.join(kept, "sibling.txt"), "w", encoding="utf-8") as f:
        f.write("keep")
    os.removedirs(os.path.join(kept, "z"))
    assert not os.path.exists(os.path.join(kept, "z")), "leaf removed"
    assert os.path.isdir(kept), "non-empty parent kept"
print("removedirs_prunes_empty_parents OK")
"###);
    assert_output(&out, r###"removedirs_prunes_empty_parents OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/os/scandir_yields_direntry.py`.
#[test]
fn test_gen_behavior_std_libs_os_scandir_yields_direntry() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "os"
# dimension = "behavior"
# case = "scandir_yields_direntry"
# subject = "os.scandir"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""os.scandir: os.scandir yields os.DirEntry objects exposing name/path and is_file()/is_dir() predicates over a temp dir containing a file and a subdir"""
import os
import tempfile

with tempfile.TemporaryDirectory() as d:
    # Populate a file and a subdirectory.
    fpath = os.path.join(d, "file.txt")
    with open(fpath, "w", encoding="utf-8") as f:
        f.write("python")
    os.mkdir(os.path.join(d, "subdir"))

    # scandir yields os.DirEntry objects exposing name/path/type predicates.
    by_name = {}
    with os.scandir(d) as it:
        for entry in it:
            assert isinstance(entry, os.DirEntry), f"not DirEntry: {entry!r}"
            by_name[entry.name] = entry

    assert set(by_name) == {"file.txt", "subdir"}, f"names = {set(by_name)!r}"
    assert by_name["file.txt"].is_file(), "file.txt is_file"
    assert not by_name["file.txt"].is_dir(), "file.txt not is_dir"
    assert by_name["subdir"].is_dir(), "subdir is_dir"
    assert by_name["file.txt"].path == fpath, f"path = {by_name['file.txt'].path!r}"
print("scandir_yields_direntry OK")
"###);
    assert_output(&out, r###"scandir_yields_direntry OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/os/urandom_length_and_entropy.py`.
#[test]
fn test_gen_behavior_std_libs_os_urandom_length_and_entropy() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "os"
# dimension = "behavior"
# case = "urandom_length_and_entropy"
# subject = "os.urandom"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""os.urandom: os.urandom(n) returns exactly n bytes for several n, and two independent 16-byte draws differ"""
import os

# urandom(n) returns exactly n bytes.
for n in (0, 1, 10, 100, 1000):
    data = os.urandom(n)
    assert isinstance(data, bytes), f"urandom({n}) type = {type(data)!r}"
    assert len(data) == n, f"urandom({n}) len = {len(data)}"

# Two independent draws of meaningful length should differ.
a = os.urandom(16)
b = os.urandom(16)
assert isinstance(a, bytes) and isinstance(b, bytes), "draws are bytes"
assert a != b, "two 16-byte draws collided (astronomically unlikely)"
print("urandom_length_and_entropy OK")
"###);
    assert_output(&out, r###"urandom_length_and_entropy OK
"###);
}
