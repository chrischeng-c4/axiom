use super::super::super::super::harness::*;

/// Ported from `tests/cpython/behavior/std-libs/glob/char_class_matches_set.py`.
#[test]
fn test_gen_behavior_std_libs_glob_char_class_matches_set() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "glob"
# dimension = "behavior"
# case = "char_class_matches_set"
# subject = "glob.glob"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_glob.py"
# status = "filled"
# ///
"""glob.glob: the character class [abc] matches any one of the listed chars: '[abc].txt' over {a,b,c,d}.txt yields a.txt,b.txt,c.txt and not d.txt"""
import glob
import os
import tempfile

with tempfile.TemporaryDirectory() as d:
    for name in ("a.txt", "b.txt", "c.txt", "d.txt"):
        with open(os.path.join(d, name), "w") as fh:
            fh.write("")
    results = sorted(glob.glob(os.path.join(d, "[abc].txt")))
    bases = [os.path.basename(p) for p in results]
    assert bases == ["a.txt", "b.txt", "c.txt"], f"[abc].txt = {bases!r}"

print("char_class_matches_set OK")
"###);
    assert_output(&out, r###"char_class_matches_set OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/glob/double_star_non_recursive_stays_one_level.py`.
#[test]
fn test_gen_behavior_std_libs_glob_double_star_non_recursive_stays_one_level() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "glob"
# dimension = "behavior"
# case = "double_star_non_recursive_stays_one_level"
# subject = "glob.glob"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_glob.py"
# status = "filled"
# ///
"""glob.glob: with recursive=False, '**/*.txt' behaves like a single '*' segment and does NOT reach the deep 'sub/deep/c.txt' entry"""
import glob
import os
import tempfile

with tempfile.TemporaryDirectory() as d:
    for rel in ("a.txt", os.path.join("sub", "b.txt"), os.path.join("sub", "deep", "c.txt")):
        full = os.path.join(d, rel)
        os.makedirs(os.path.dirname(full), exist_ok=True)
        with open(full, "w") as fh:
            fh.write("")
    rec = sorted(glob.glob(os.path.join(d, "**", "*.txt"), recursive=True))
    norec = sorted(glob.glob(os.path.join(d, "**", "*.txt"), recursive=False))
    assert len(rec) == 3, f"recursive count = {len(rec)!r}"
    deep = [p for p in norec if "deep" in p]
    assert deep == [], f"non-recursive no deep = {deep!r}"

print("double_star_non_recursive_stays_one_level OK")
"###);
    assert_output(&out, r###"double_star_non_recursive_stays_one_level OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/glob/double_star_recursive_descends_all_levels.py`.
#[test]
fn test_gen_behavior_std_libs_glob_double_star_recursive_descends_all_levels() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "glob"
# dimension = "behavior"
# case = "double_star_recursive_descends_all_levels"
# subject = "glob.glob"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_glob.py"
# status = "filled"
# ///
"""glob.glob: with recursive=True, '**/*.txt' descends every level: a tree {a.txt, sub/b.txt, sub/deep/c.txt} yields all three relative paths"""
import glob
import os
import tempfile

with tempfile.TemporaryDirectory() as d:
    for rel in ("a.txt", os.path.join("sub", "b.txt"), os.path.join("sub", "deep", "c.txt")):
        full = os.path.join(d, rel)
        os.makedirs(os.path.dirname(full), exist_ok=True)
        with open(full, "w") as fh:
            fh.write("")
    results = sorted(glob.glob(os.path.join(d, "**", "*.txt"), recursive=True))
    rels = sorted(os.path.relpath(p, d) for p in results)
    assert rels == sorted(["a.txt", os.path.join("sub", "b.txt"),
                           os.path.join("sub", "deep", "c.txt")]), f"recursive = {rels!r}"

print("double_star_recursive_descends_all_levels OK")
"###);
    assert_output(&out, r###"double_star_recursive_descends_all_levels OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/glob/empty_pattern_returns_empty_list.py`.
#[test]
fn test_gen_behavior_std_libs_glob_empty_pattern_returns_empty_list() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "glob"
# dimension = "behavior"
# case = "empty_pattern_returns_empty_list"
# subject = "glob.glob"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_glob.py"
# status = "filled"
# ///
"""glob.glob: glob('') returns [] (the empty pattern matches nothing)"""
import glob

assert glob.glob("") == [], "empty str pattern == []"
assert glob.glob(b"") == [], "empty bytes pattern == []"

print("empty_pattern_returns_empty_list OK")
"###);
    assert_output(&out, r###"empty_pattern_returns_empty_list OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/glob/escape_exact_strings.py`.
#[test]
fn test_gen_behavior_std_libs_glob_escape_exact_strings() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "glob"
# dimension = "behavior"
# case = "escape_exact_strings"
# subject = "glob.escape"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_glob.py"
# status = "filled"
# ///
"""glob.escape: escape wraps each glob metacharacter in a literal character class: escape('plain')=='plain', escape('a*')=='a[*]', escape('a?b')=='a[?]b', escape('file*.txt')=='file[*].txt'"""
import glob

for pattern, expected in [
    ("plain", "plain"),
    ("a*", "a[*]"),
    ("a?b", "a[?]b"),
    ("file*.txt", "file[*].txt"),
]:
    assert glob.escape(pattern) == expected, (pattern, glob.escape(pattern), expected)

print("escape_exact_strings OK")
"###);
    assert_output(&out, r###"escape_exact_strings OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/glob/escape_neutralizes_metachars.py`.
#[test]
fn test_gen_behavior_std_libs_glob_escape_neutralizes_metachars() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "glob"
# dimension = "behavior"
# case = "escape_neutralizes_metachars"
# subject = "glob.escape"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_glob.py"
# status = "filled"
# ///
"""glob.escape: escape() makes a name with literal glob metachars matchable: a real file 'file[1].txt' is found by glob(escape(path)) and nothing else"""
import glob
import os
import tempfile

with tempfile.TemporaryDirectory() as d:
    fname = "file[1].txt"
    with open(os.path.join(d, fname), "w") as fh:
        fh.write("")
    escaped = glob.escape(os.path.join(d, fname))
    results = glob.glob(escaped)
    assert len(results) == 1, f"escaped match count = {len(results)!r}"
    assert os.path.basename(results[0]) == fname, f"exact match = {results!r}"

print("escape_neutralizes_metachars OK")
"###);
    assert_output(&out, r###"escape_neutralizes_metachars OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/glob/glob0_literal_basename.py`.
#[test]
fn test_gen_behavior_std_libs_glob_glob0_literal_basename() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "glob"
# dimension = "behavior"
# case = "glob0_literal_basename"
# subject = "glob.glob0"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""glob.glob0: glob0(dirname, literal) returns [literal] when the named entry exists in dirname and [] when it does not (literal-basename helper)"""
import glob
import os
import tempfile

with tempfile.TemporaryDirectory() as d:
    with open(os.path.join(d, "alpha.txt"), "w") as fh:
        fh.write("")
    assert glob.glob0(d, "alpha.txt") == ["alpha.txt"], "literal hit"
    assert glob.glob0(d, "missing.zzz") == [], "literal miss"

print("glob0_literal_basename OK")
"###);
    assert_output(&out, r###"glob0_literal_basename OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/glob/glob1_pattern_in_dirname.py`.
#[test]
fn test_gen_behavior_std_libs_glob_glob1_pattern_in_dirname() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "glob"
# dimension = "behavior"
# case = "glob1_pattern_in_dirname"
# subject = "glob.glob1"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""glob.glob1: glob1(dirname, pattern) returns the basenames inside dirname matching pattern: '*.txt' yields only the .txt basenames, '*.rs' only the .rs ones"""
import glob
import os
import tempfile

with tempfile.TemporaryDirectory() as d:
    for name in ("alpha.txt", "beta.txt", "gamma.rs", "delta.md"):
        with open(os.path.join(d, name), "w") as fh:
            fh.write("")
    assert sorted(glob.glob1(d, "*.txt")) == ["alpha.txt", "beta.txt"], "glob1 *.txt"
    assert sorted(glob.glob1(d, "*.rs")) == ["gamma.rs"], "glob1 *.rs"

print("glob1_pattern_in_dirname OK")
"###);
    assert_output(&out, r###"glob1_pattern_in_dirname OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/glob/glob_returns_str_paths.py`.
#[test]
fn test_gen_behavior_std_libs_glob_glob_returns_str_paths() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "glob"
# dimension = "behavior"
# case = "glob_returns_str_paths"
# subject = "glob.glob"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_glob.py"
# status = "filled"
# ///
"""glob.glob: a str pattern yields str results: every element of glob('*') in a temp dir is a str"""
import glob
import os
import tempfile

with tempfile.TemporaryDirectory() as d:
    for name in ("a.txt", "b.py"):
        with open(os.path.join(d, name), "w") as fh:
            fh.write("")
    results = glob.glob(os.path.join(d, "*"))
    assert len(results) == 2, f"count = {len(results)!r}"
    assert {type(p).__name__ for p in results} == {"str"}, f"types = {results!r}"

print("glob_returns_str_paths OK")
"###);
    assert_output(&out, r###"glob_returns_str_paths OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/glob/has_magic_detects_wildcards.py`.
#[test]
fn test_gen_behavior_std_libs_glob_has_magic_detects_wildcards() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "glob"
# dimension = "behavior"
# case = "has_magic_detects_wildcards"
# subject = "glob.has_magic"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_glob.py"
# status = "filled"
# ///
"""glob.has_magic: has_magic is True iff the pattern contains a glob metachar (* ? [): True for 'a*', 'a?b', 'x[1]'; False for 'plain' and 'path/to/file'"""
import glob

for pattern, expected in [
    ("plain", False),
    ("path/to/file", False),
    ("a*", True),
    ("a?b", True),
    ("x[1]", True),
]:
    assert glob.has_magic(pattern) == expected, (pattern, glob.has_magic(pattern), expected)

print("has_magic_detects_wildcards OK")
"###);
    assert_output(&out, r###"has_magic_detects_wildcards OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/glob/iglob_is_iterator.py`.
#[test]
fn test_gen_behavior_std_libs_glob_iglob_is_iterator() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "glob"
# dimension = "behavior"
# case = "iglob_is_iterator"
# subject = "glob.iglob"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_glob.py"
# status = "filled"
# ///
"""glob.iglob: iglob returns a lazy iterator (has __iter__ and __next__) whose materialized results match glob() for the same pattern"""
import glob
import os
import tempfile

with tempfile.TemporaryDirectory() as d:
    for name in ("a.txt", "b.txt", "c.txt"):
        with open(os.path.join(d, name), "w") as fh:
            fh.write("")
    pattern = os.path.join(d, "*.txt")
    it = glob.iglob(pattern)
    assert hasattr(it, "__iter__"), "iglob iterable"
    assert hasattr(it, "__next__"), "iglob iterator"
    materialized = sorted(it)
    assert sorted(glob.glob(pattern)) == materialized, "iglob == glob results"
    assert len(materialized) == 3, f"iglob count = {len(materialized)!r}"

print("iglob_is_iterator OK")
"###);
    assert_output(&out, r###"iglob_is_iterator OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/glob/iglob_matches_glob_results.py`.
#[test]
fn test_gen_behavior_std_libs_glob_iglob_matches_glob_results() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "glob"
# dimension = "behavior"
# case = "iglob_matches_glob_results"
# subject = "glob.iglob"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_glob.py"
# status = "filled"
# ///
"""glob.iglob: iglob and glob produce the same set of paths for one pattern: sorted(glob('*.py')) == sorted(iglob('*.py')) over {x,y,z}.py"""
import glob
import os
import tempfile

with tempfile.TemporaryDirectory() as d:
    for name in ("x.py", "y.py", "z.py"):
        with open(os.path.join(d, name), "w") as fh:
            fh.write("")
    pattern = os.path.join(d, "*.py")
    g = sorted(glob.glob(pattern))
    ig = sorted(glob.iglob(pattern))
    assert g == ig, f"glob {g} vs iglob {ig}"
    assert len(g) == 3, f"count = {len(g)!r}"

print("iglob_matches_glob_results OK")
"###);
    assert_output(&out, r###"iglob_matches_glob_results OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/glob/no_match_returns_empty_list.py`.
#[test]
fn test_gen_behavior_std_libs_glob_no_match_returns_empty_list() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "glob"
# dimension = "behavior"
# case = "no_match_returns_empty_list"
# subject = "glob.glob"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""glob.glob: a pattern with no matching entry returns an empty list (not None, not a raise): glob('*.xyz') in a dir with no .xyz files == []"""
import glob
import os
import tempfile

with tempfile.TemporaryDirectory() as d:
    for name in ("a.txt", "b.py"):
        with open(os.path.join(d, name), "w") as fh:
            fh.write("")
    results = glob.glob(os.path.join(d, "*.xyz"))
    assert results == [], f"no match = {results!r}"

print("no_match_returns_empty_list OK")
"###);
    assert_output(&out, r###"no_match_returns_empty_list OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/glob/qmark_matches_exactly_one_char.py`.
#[test]
fn test_gen_behavior_std_libs_glob_qmark_matches_exactly_one_char() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "glob"
# dimension = "behavior"
# case = "qmark_matches_exactly_one_char"
# subject = "glob.glob"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_glob.py"
# status = "filled"
# ///
"""glob.glob: ? matches exactly one character, not zero or many: '?.txt' over {a.txt,ab.txt,abc.txt} matches only 'a.txt'"""
import glob
import os
import tempfile

with tempfile.TemporaryDirectory() as d:
    for name in ("a.txt", "ab.txt", "abc.txt"):
        with open(os.path.join(d, name), "w") as fh:
            fh.write("")
    results = sorted(glob.glob(os.path.join(d, "?.txt")))
    bases = [os.path.basename(p) for p in results]
    assert bases == ["a.txt"], f"?.txt = {bases!r}"

print("qmark_matches_exactly_one_char OK")
"###);
    assert_output(&out, r###"qmark_matches_exactly_one_char OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/glob/results_are_existing_paths.py`.
#[test]
fn test_gen_behavior_std_libs_glob_results_are_existing_paths() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "glob"
# dimension = "behavior"
# case = "results_are_existing_paths"
# subject = "glob.glob"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""glob.glob: every path glob returns actually exists on disk: os.path.exists is True for each result of glob('*.txt') in a populated temp dir"""
import glob
import os
import tempfile

with tempfile.TemporaryDirectory() as d:
    for name in ("a.txt", "b.txt"):
        with open(os.path.join(d, name), "w") as fh:
            fh.write("")
    results = glob.glob(os.path.join(d, "*.txt"))
    assert len(results) == 2, f"count = {len(results)!r}"
    assert all(os.path.exists(p) for p in results), f"all exist = {results!r}"

print("results_are_existing_paths OK")
"###);
    assert_output(&out, r###"results_are_existing_paths OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/glob/results_include_directories.py`.
#[test]
fn test_gen_behavior_std_libs_glob_results_include_directories() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "glob"
# dimension = "behavior"
# case = "results_include_directories"
# subject = "glob.glob"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""glob.glob: glob('*') includes both files and subdirectories: a temp dir with a file 'file.txt' and a subdir 'subdir' yields both basenames"""
import glob
import os
import tempfile

with tempfile.TemporaryDirectory() as d:
    with open(os.path.join(d, "file.txt"), "w") as fh:
        fh.write("")
    os.mkdir(os.path.join(d, "subdir"))
    results = sorted(glob.glob(os.path.join(d, "*")))
    bases = [os.path.basename(p) for p in results]
    assert "file.txt" in bases, f"file in results = {bases!r}"
    assert "subdir" in bases, f"dir in results = {bases!r}"

print("results_include_directories OK")
"###);
    assert_output(&out, r###"results_include_directories OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/glob/star_matches_extension.py`.
#[test]
fn test_gen_behavior_std_libs_glob_star_matches_extension() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "glob"
# dimension = "behavior"
# case = "star_matches_extension"
# subject = "glob.glob"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_glob.py"
# status = "filled"
# ///
"""glob.glob: glob('*.txt') returns a list of the matching paths; in a temp dir of {a.txt,b.txt,c.py} the *.txt pattern yields exactly the two .txt files (sorted basenames ['a.txt','b.txt'])"""
import glob
import os
import tempfile

with tempfile.TemporaryDirectory() as d:
    for name in ("a.txt", "b.txt", "c.py"):
        with open(os.path.join(d, name), "w") as fh:
            fh.write("")
    results = glob.glob(os.path.join(d, "*.txt"))
    assert isinstance(results, list), f"glob type = {type(results)!r}"
    bases = sorted(os.path.basename(p) for p in results)
    assert bases == ["a.txt", "b.txt"], f"*.txt = {bases!r}"

print("star_matches_extension OK")
"###);
    assert_output(&out, r###"star_matches_extension OK
"###);
}
