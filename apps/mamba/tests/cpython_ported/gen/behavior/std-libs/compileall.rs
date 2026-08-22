use super::super::super::super::harness::*;

/// Ported from `tests/cpython/behavior/std-libs/compileall/compile_dir_compiles_all_py_files.py`.
#[test]
fn test_gen_behavior_std_libs_compileall_compile_dir_compiles_all_py_files() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "compileall"
# dimension = "behavior"
# case = "compile_dir_compiles_all_py_files"
# subject = "compileall.compile_dir"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_compileall.py"
# status = "filled"
# ///
"""compileall.compile_dir: compile_dir returns True and produces one .pyc under __pycache__ for every .py file in the directory"""
import compileall
import os
import tempfile

with tempfile.TemporaryDirectory() as d:
    names = ["alpha.py", "beta.py", "gamma.py"]
    for name in names:
        with open(os.path.join(d, name), "w") as f:
            f.write("x = %d\n" % len(name))
    ok = compileall.compile_dir(d, quiet=2)
    assert ok, "compile_dir returns True"
    cache = os.path.join(d, "__pycache__")
    pycs = [f for f in os.listdir(cache) if f.endswith(".pyc")]
    assert len(pycs) == 3, pycs
print("compile_dir_compiles_all_py_files OK")
"###);
    assert_output(&out, r###"compile_dir_compiles_all_py_files OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/compileall/compile_dir_rx_skips_matching_files.py`.
#[test]
fn test_gen_behavior_std_libs_compileall_compile_dir_rx_skips_matching_files() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "compileall"
# dimension = "behavior"
# case = "compile_dir_rx_skips_matching_files"
# subject = "compileall.compile_dir"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_compileall.py"
# status = "filled"
# ///
"""compileall.compile_dir: compile_dir with rx=<compiled regex> skips files whose path matches the pattern and compiles only the rest"""
import compileall
import os
import re
import tempfile

with tempfile.TemporaryDirectory() as d:
    for fn in ["include.py", "skip_me.py", "skip_too.py"]:
        with open(os.path.join(d, fn), "w") as f:
            f.write("pass\n")
    compileall.compile_dir(d, quiet=2, rx=re.compile(r"skip.*\.py"))
    cache = os.path.join(d, "__pycache__")
    pycs = [f for f in os.listdir(cache) if f.endswith(".pyc")]
    assert all("skip" not in f for f in pycs), pycs
    assert any("include" in f for f in pycs), pycs
print("compile_dir_rx_skips_matching_files OK")
"###);
    assert_output(&out, r###"compile_dir_rx_skips_matching_files OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/compileall/compile_file_creates_pyc_in_pycache.py`.
#[test]
fn test_gen_behavior_std_libs_compileall_compile_file_creates_pyc_in_pycache() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "compileall"
# dimension = "behavior"
# case = "compile_file_creates_pyc_in_pycache"
# subject = "compileall.compile_file"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_compileall.py"
# status = "filled"
# ///
"""compileall.compile_file: compile_file on a valid .py returns True and creates a matching .pyc under __pycache__ whose name starts with the module name"""
import compileall
import os
import tempfile

with tempfile.TemporaryDirectory() as d:
    src = os.path.join(d, "hello.py")
    with open(src, "w") as f:
        f.write("print('hello')\n")
    ok = compileall.compile_file(src, quiet=2)
    assert ok, "compile_file returns True"
    cache = os.path.join(d, "__pycache__")
    assert os.path.isdir(cache), "__pycache__ created"
    pycs = [f for f in os.listdir(cache) if f.endswith(".pyc")]
    assert len(pycs) >= 1, pycs
    # The cache file name is derived from the module name.
    assert any(f.startswith("hello.") for f in pycs), pycs
print("compile_file_creates_pyc_in_pycache OK")
"###);
    assert_output(&out, r###"compile_file_creates_pyc_in_pycache OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/compileall/compile_file_legacy_writes_pyc_next_to_source.py`.
#[test]
fn test_gen_behavior_std_libs_compileall_compile_file_legacy_writes_pyc_next_to_source() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "compileall"
# dimension = "behavior"
# case = "compile_file_legacy_writes_pyc_next_to_source"
# subject = "compileall.compile_file"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""compileall.compile_file: compile_file with legacy=True writes the .pyc next to the .py source (legacy layout) instead of under __pycache__"""
import compileall
import os
import tempfile

with tempfile.TemporaryDirectory() as d:
    src = os.path.join(d, "legacy_test.py")
    with open(src, "w") as f:
        f.write("z = 99\n")
    ok = compileall.compile_file(src, quiet=2, legacy=True)
    assert ok, "legacy compile succeeds"
    # Legacy layout places the cache file beside the source, not in __pycache__.
    pyc_next_to = os.path.join(d, "legacy_test.pyc")
    assert os.path.exists(pyc_next_to), os.listdir(d)
    assert not os.path.exists(os.path.join(d, "__pycache__")), os.listdir(d)
print("compile_file_legacy_writes_pyc_next_to_source OK")
"###);
    assert_output(&out, r###"compile_file_legacy_writes_pyc_next_to_source OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/compileall/compile_path_returns_bool.py`.
#[test]
fn test_gen_behavior_std_libs_compileall_compile_path_returns_bool() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "compileall"
# dimension = "behavior"
# case = "compile_path_returns_bool"
# subject = "compileall.compile_path"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_compileall.py"
# status = "filled"
# ///
"""compileall.compile_path: compile_path walks sys.path entries and returns a bool/int verdict without raising"""
import compileall

# compile_path walks sys.path; some entries may be unwriteable, so the verdict
# is True or False depending on the environment, but it never raises and is
# always a bool/int.
ok = compileall.compile_path(quiet=2)
assert isinstance(ok, (bool, int)), type(ok)
print("compile_path_returns_bool OK")
"###);
    assert_output(&out, r###"compile_path_returns_bool OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/compileall/maxlevels_bounds_recursion_depth.py`.
#[test]
fn test_gen_behavior_std_libs_compileall_maxlevels_bounds_recursion_depth() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "compileall"
# dimension = "behavior"
# case = "maxlevels_bounds_recursion_depth"
# subject = "compileall.compile_dir"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_compileall.py"
# status = "filled"
# ///
"""compileall.compile_dir: maxlevels bounds compile_dir recursion: a depth-3 source is left uncompiled at maxlevels=2 but compiled at maxlevels=3"""
import compileall
import importlib.util
import os
import shutil
import tempfile

with tempfile.TemporaryDirectory() as d:
    top = os.path.join(d, "top.py")
    with open(top, "w") as f:
        f.write("x = 1\n")
    path = d
    for i in range(1, 4):
        path = os.path.join(path, "dir_%d" % i)
        os.mkdir(path)
        shutil.copyfile(top, os.path.join(path, "script.py"))
    deep_src = os.path.join(path, "script.py")
    deep_pyc = importlib.util.cache_from_source(deep_src)

    compileall.compile_dir(d, quiet=True, maxlevels=2)
    assert not os.path.isfile(deep_pyc), "depth-3 file untouched at maxlevels=2"

    compileall.compile_dir(d, quiet=True, maxlevels=3)
    assert os.path.isfile(deep_pyc), "depth-3 file compiled at maxlevels=3"
print("maxlevels_bounds_recursion_depth OK")
"###);
    assert_output(&out, r###"maxlevels_bounds_recursion_depth OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/compileall/non_py_file_skipped_no_pycache.py`.
#[test]
fn test_gen_behavior_std_libs_compileall_non_py_file_skipped_no_pycache() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "compileall"
# dimension = "behavior"
# case = "non_py_file_skipped_no_pycache"
# subject = "compileall.compile_file"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_compileall.py"
# status = "filled"
# ///
"""compileall.compile_file: compile_file on a non-.py file is a no-op: it never creates a __pycache__ directory"""
import compileall
import os
import tempfile

with tempfile.TemporaryDirectory() as d:
    data = os.path.join(d, "file")
    with open(data, "wb"):
        pass
    compileall.compile_file(data, quiet=2)
    assert not os.path.exists(os.path.join(d, "__pycache__")), os.listdir(d)
print("non_py_file_skipped_no_pycache OK")
"###);
    assert_output(&out, r###"non_py_file_skipped_no_pycache OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/compileall/pathlib_path_inputs_accepted.py`.
#[test]
fn test_gen_behavior_std_libs_compileall_pathlib_path_inputs_accepted() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "compileall"
# dimension = "behavior"
# case = "pathlib_path_inputs_accepted"
# subject = "compileall.compile_dir"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_compileall.py"
# status = "filled"
# ///
"""compileall.compile_dir: pathlib.Path arguments are accepted everywhere a str path is: compile_dir(Path(d), prependdir=Path(...)) and compile_file(Path(src), stripdir=Path(...)) both succeed and create the cache file"""
import compileall
import importlib.util
import os
import pathlib
import tempfile

# Path inputs to compile_dir + prependdir: still creates the normal cache file.
with tempfile.TemporaryDirectory() as d:
    src = os.path.join(d, "p.py")
    with open(src, "w") as f:
        f.write("x = 123\n")
    cache = importlib.util.cache_from_source(src)
    assert not os.path.isfile(cache), "cache absent before compile"
    ok = compileall.compile_dir(pathlib.Path(d),
                                prependdir=pathlib.Path("prepend_root"),
                                quiet=2)
    assert ok, "compile_dir with Path + prependdir succeeds"
    assert os.path.isfile(cache), "cache created"

# Path inputs to compile_file + stripdir on a single file.
with tempfile.TemporaryDirectory() as d:
    src = os.path.join(d, "s.py")
    with open(src, "w") as f:
        f.write("y = 1\n")
    cache = importlib.util.cache_from_source(src)
    ok = compileall.compile_file(pathlib.Path(src),
                                 stripdir=pathlib.Path("strip_root"),
                                 quiet=2)
    assert ok, "compile_file with Path + stripdir succeeds"
    assert os.path.isfile(cache), "cache created"
print("pathlib_path_inputs_accepted OK")
"###);
    assert_output(&out, r###"pathlib_path_inputs_accepted OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/compileall/workers_one_compiles_serially.py`.
#[test]
fn test_gen_behavior_std_libs_compileall_workers_one_compiles_serially() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "compileall"
# dimension = "behavior"
# case = "workers_one_compiles_serially"
# subject = "compileall.compile_dir"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_compileall.py"
# status = "filled"
# ///
"""compileall.compile_dir: compile_dir with workers=1 compiles every file in-process (serial, no process pool) and produces one .pyc per source"""
import compileall
import os
import tempfile

# workers=1 stays in-process (no ProcessPoolExecutor, which would be
# nondeterministic and unsafe to spawn from a bare script).
with tempfile.TemporaryDirectory() as d:
    for name in ("a.py", "b.py", "c.py"):
        with open(os.path.join(d, name), "w") as f:
            f.write("v = 1\n")
    ok = compileall.compile_dir(d, quiet=2, workers=1)
    assert ok, "workers=1 compiles"
    cache = os.path.join(d, "__pycache__")
    pycs = [f for f in os.listdir(cache) if f.endswith(".pyc")]
    assert len(pycs) == 3, pycs
print("workers_one_compiles_serially OK")
"###);
    assert_output(&out, r###"workers_one_compiles_serially OK
"###);
}
