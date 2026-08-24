use super::super::super::super::harness::*;

/// Ported from `tests/cpython/behavior/std-libs/mimetypes/add_type_persists.py`.
#[test]
fn test_gen_behavior_std_libs_mimetypes_add_type_persists() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "mimetypes"
# dimension = "behavior"
# case = "add_type_persists"
# subject = "mimetypes.add_type"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_mimetypes.py"
# status = "filled"
# ///
"""mimetypes.add_type: add_type registers a type so later guess_type calls see it: add_type('application/x-behavior-test', '.btest') then guess_type('document.btest') returns it"""
import mimetypes

mimetypes.add_type("application/x-behavior-test", ".btest")
t, e = mimetypes.guess_type("document.btest")
assert t == "application/x-behavior-test", f"custom type = {t!r}"
assert e is None, f"custom encoding = {e!r}"
print("add_type_persists OK")
"###);
    assert_output(&out, r###"add_type_persists OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/mimetypes/add_type_strict_flag_routes_table.py`.
#[test]
fn test_gen_behavior_std_libs_mimetypes_add_type_strict_flag_routes_table() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "mimetypes"
# dimension = "behavior"
# case = "add_type_strict_flag_routes_table"
# subject = "mimetypes.MimeTypes"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_mimetypes.py"
# status = "filled"
# ///
"""mimetypes.MimeTypes: add_type(strict=) routes to the strict vs common table: default add_type lands in the strict table, strict=False in the loose one, and guess_all_extensions(strict=False) sees both"""
import mimetypes

db = mimetypes.MimeTypes()

# add_type default lands in the strict table; strict=False in the loose one.
db.add_type("test-type", ".strict-ext")
db.add_type("test-type", ".non-strict-ext", strict=False)
assert db.guess_all_extensions("test-type") == [".strict-ext"], "default strict only"
assert db.guess_all_extensions("test-type", strict=False) == [
    ".strict-ext",
    ".non-strict-ext",
], "loose sees both"
print("add_type_strict_flag_routes_table OK")
"###);
    assert_output(&out, r###"add_type_strict_flag_routes_table OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/mimetypes/guess_all_extensions_list.py`.
#[test]
fn test_gen_behavior_std_libs_mimetypes_guess_all_extensions_list() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "mimetypes"
# dimension = "behavior"
# case = "guess_all_extensions_list"
# subject = "mimetypes.guess_all_extensions"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_mimetypes.py"
# status = "filled"
# ///
"""mimetypes.guess_all_extensions: guess_all_extensions returns the full list of registered extensions for a type: text/html yields .html and .htm"""
import mimetypes

exts = mimetypes.guess_all_extensions("text/html")
assert isinstance(exts, list), f"guess_all type = {type(exts)!r}"
assert len(exts) >= 1, f"text/html has at least one ext: {exts!r}"
assert ".html" in exts, f".html in {exts!r}"
assert ".htm" in exts, f".htm in {exts!r}"
print("guess_all_extensions_list OK")
"###);
    assert_output(&out, r###"guess_all_extensions_list OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/mimetypes/guess_all_extensions_returns_copy.py`.
#[test]
fn test_gen_behavior_std_libs_mimetypes_guess_all_extensions_returns_copy() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "mimetypes"
# dimension = "behavior"
# case = "guess_all_extensions_returns_copy"
# subject = "mimetypes.MimeTypes"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_mimetypes.py"
# status = "filled"
# ///
"""mimetypes.MimeTypes: guess_all_extensions returns a fresh list copy: mutating the result cannot corrupt the internal table"""
import mimetypes

db = mimetypes.MimeTypes()
db.add_type("test-type", ".strict-ext")

# The returned list is a fresh copy; mutating it cannot corrupt the table.
got = db.guess_all_extensions("test-type")
got.append(".no-such-ext")
assert ".no-such-ext" not in db.guess_all_extensions("test-type"), "copy isolated"
print("guess_all_extensions_returns_copy OK")
"###);
    assert_output(&out, r###"guess_all_extensions_returns_copy OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/mimetypes/guess_extension_dotted_string.py`.
#[test]
fn test_gen_behavior_std_libs_mimetypes_guess_extension_dotted_string() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "mimetypes"
# dimension = "behavior"
# case = "guess_extension_dotted_string"
# subject = "mimetypes.guess_extension"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_mimetypes.py"
# status = "filled"
# ///
"""mimetypes.guess_extension: guess_extension returns a single dotted extension string for a known type (image/png)"""
import mimetypes

ext = mimetypes.guess_extension("image/png")
assert ext is not None, "image/png has extension"
assert isinstance(ext, str), f"extension is str: {type(ext)!r}"
assert ext.startswith("."), f"starts with dot: {ext!r}"
print("guess_extension_dotted_string OK")
"###);
    assert_output(&out, r###"guess_extension_dotted_string OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/mimetypes/guess_type_case_insensitive.py`.
#[test]
fn test_gen_behavior_std_libs_mimetypes_guess_type_case_insensitive() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "mimetypes"
# dimension = "behavior"
# case = "guess_type_case_insensitive"
# subject = "mimetypes.guess_type"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_mimetypes.py"
# status = "filled"
# ///
"""mimetypes.guess_type: extension matching is case-insensitive: FILE.HTML and file.html both map to text/html"""
import mimetypes

ta, _ = mimetypes.guess_type("FILE.HTML")
tb, _ = mimetypes.guess_type("file.html")
assert ta == "text/html", f"uppercase ext: {ta!r}"
assert tb == "text/html", f"lowercase ext: {tb!r}"
assert ta == tb, "case-folded to the same type"
print("guess_type_case_insensitive OK")
"###);
    assert_output(&out, r###"guess_type_case_insensitive OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/mimetypes/guess_type_common_types.py`.
#[test]
fn test_gen_behavior_std_libs_mimetypes_guess_type_common_types() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "mimetypes"
# dimension = "behavior"
# case = "guess_type_common_types"
# subject = "mimetypes.guess_type"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_mimetypes.py"
# status = "filled"
# ///
"""mimetypes.guess_type: common filenames map to their IANA type with no encoding: .html/.css/.js/.png/.jpg/.json/.pdf"""
import mimetypes

cases = [
    ("index.html", "text/html", None),
    ("style.css", "text/css", None),
    ("script.js", "text/javascript", None),
    ("image.png", "image/png", None),
    ("image.jpg", "image/jpeg", None),
    ("data.json", "application/json", None),
    ("doc.pdf", "application/pdf", None),
]
for fname, etype, eenc in cases:
    t, e = mimetypes.guess_type(fname)
    assert t == etype, f"{fname!r}: type = {t!r}, expected {etype!r}"
    assert e == eenc, f"{fname!r}: encoding = {e!r}"
print("guess_type_common_types OK")
"###);
    assert_output(&out, r###"guess_type_common_types OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/mimetypes/guess_type_encoding_layer.py`.
#[test]
fn test_gen_behavior_std_libs_mimetypes_guess_type_encoding_layer() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "mimetypes"
# dimension = "behavior"
# case = "guess_type_encoding_layer"
# subject = "mimetypes.guess_type"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_mimetypes.py"
# status = "filled"
# ///
"""mimetypes.guess_type: a compression suffix yields the encoding: file.gz -> (None, 'gzip') and archive.tar.gz -> ('application/x-tar', 'gzip')"""
import mimetypes

# A bare .gz carries the encoding but no underlying type.
t1, e1 = mimetypes.guess_type("file.gz")
assert t1 is None, f"gz type = {t1!r}"
assert e1 == "gzip", f"gz encoding = {e1!r}"

# A compound .tar.gz resolves to the tar type plus the gzip encoding.
t2, e2 = mimetypes.guess_type("archive.tar.gz")
assert t2 == "application/x-tar", f"tar.gz type = {t2!r}"
assert e2 == "gzip", f"tar.gz encoding = {e2!r}"
print("guess_type_encoding_layer OK")
"###);
    assert_output(&out, r###"guess_type_encoding_layer OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/mimetypes/guess_type_pathlike_object.py`.
#[test]
fn test_gen_behavior_std_libs_mimetypes_guess_type_pathlike_object() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "mimetypes"
# dimension = "behavior"
# case = "guess_type_pathlike_object"
# subject = "mimetypes.MimeTypes"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_mimetypes.py"
# status = "filled"
# ///
"""mimetypes.MimeTypes: guess_type accepts any os.PathLike (__fspath__) and only the final extension matters: a FakePath('LICENSE.txt') guesses identically to the string, and a directory-only path returns (None, None)"""
import mimetypes


class FakePath:
    """Minimal os.PathLike wrapper."""

    def __init__(self, value):
        self._value = value

    def __fspath__(self):
        return self._value


db = mimetypes.MimeTypes()
expected = db.guess_type("LICENSE.txt")
assert expected == ("text/plain", None), f"baseline = {expected!r}"

# A path-like object guesses identically to the equivalent string.
assert db.guess_type(FakePath("LICENSE.txt")) == expected, "plain pathlike"
assert db.guess_type(FakePath("/dir/LICENSE.txt")) == expected, "abs-dir pathlike"
assert db.guess_type(FakePath("../dir/LICENSE.txt")) == expected, "rel-dir pathlike"

# A directory-only path has no extension -> (None, None).
assert db.guess_type(FakePath("./")) == (None, None), "dir-only pathlike"
print("guess_type_pathlike_object OK")
"###);
    assert_output(&out, r###"guess_type_pathlike_object OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/mimetypes/guess_type_suffix_map_tgz.py`.
#[test]
fn test_gen_behavior_std_libs_mimetypes_guess_type_suffix_map_tgz() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "mimetypes"
# dimension = "behavior"
# case = "guess_type_suffix_map_tgz"
# subject = "mimetypes.guess_type"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_mimetypes.py"
# status = "filled"
# ///
"""mimetypes.guess_type: suffix_map collapses .tgz/.svgz: backup.tgz and archive.svgz resolve through the suffix table to (type, encoding)"""
import mimetypes

# suffix_map expands a shorthand suffix into its long form before lookup.
assert mimetypes.suffix_map[".tgz"] == ".tar.gz", mimetypes.suffix_map.get(".tgz")
assert mimetypes.suffix_map[".svgz"] == ".svg.gz", mimetypes.suffix_map.get(".svgz")

assert mimetypes.guess_type("backup.tgz") == ("application/x-tar", "gzip"), \
    mimetypes.guess_type("backup.tgz")
assert mimetypes.guess_type("archive.svgz") == ("image/svg+xml", "gzip"), \
    mimetypes.guess_type("archive.svgz")
print("guess_type_suffix_map_tgz OK")
"###);
    assert_output(&out, r###"guess_type_suffix_map_tgz OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/mimetypes/guess_type_url_only_extension.py`.
#[test]
fn test_gen_behavior_std_libs_mimetypes_guess_type_url_only_extension() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "mimetypes"
# dimension = "behavior"
# case = "guess_type_url_only_extension"
# subject = "mimetypes.guess_type"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_mimetypes.py"
# status = "filled"
# ///
"""mimetypes.guess_type: URL-style inputs are matched by trailing extension only: http://example.com/path/file.json?q=1 -> application/json"""
import mimetypes

t, _ = mimetypes.guess_type("http://example.com/path/file.json?q=1")
assert t == "application/json", f"url json = {t!r}"
print("guess_type_url_only_extension OK")
"###);
    assert_output(&out, r###"guess_type_url_only_extension OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/mimetypes/init_fresh_dict_identity.py`.
#[test]
fn test_gen_behavior_std_libs_mimetypes_init_fresh_dict_identity() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "mimetypes"
# dimension = "behavior"
# case = "init_fresh_dict_identity"
# subject = "mimetypes.init"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""mimetypes.init: re-running init() yields fresh dict objects (new identity) with equal content for types_map/suffix_map/encodings_map/common_types"""
import mimetypes

mimetypes.init()
sm, em = mimetypes.suffix_map, mimetypes.encodings_map
tm, ct = mimetypes.types_map, mimetypes.common_types
mimetypes.init()
# Fresh identities ...
assert sm is not mimetypes.suffix_map, "suffix_map fresh object"
assert tm is not mimetypes.types_map, "types_map fresh object"
# ... but equal content.
assert sm == mimetypes.suffix_map, "suffix_map equal content"
assert em == mimetypes.encodings_map, "encodings_map equal content"
assert tm == mimetypes.types_map, "types_map equal content"
assert ct == mimetypes.common_types, "common_types equal content"
print("init_fresh_dict_identity OK")
"###);
    assert_output(&out, r###"init_fresh_dict_identity OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/mimetypes/init_resets_registry.py`.
#[test]
fn test_gen_behavior_std_libs_mimetypes_init_resets_registry() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "mimetypes"
# dimension = "behavior"
# case = "init_resets_registry"
# subject = "mimetypes.init"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_mimetypes.py"
# status = "filled"
# ///
"""mimetypes.init: init() rebuilds the global registry from defaults, discarding add_type edits: a registered foo/bar disappears after init()"""
import mimetypes

mimetypes.add_type("foo/bar", ".foobar")
assert mimetypes.guess_extension("foo/bar") == ".foobar", "add_type took effect"
mimetypes.init()
assert mimetypes.guess_extension("foo/bar") is None, "init() reset add_type"
print("init_resets_registry OK")
"###);
    assert_output(&out, r###"init_resets_registry OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/mimetypes/keyword_argument_names.py`.
#[test]
fn test_gen_behavior_std_libs_mimetypes_keyword_argument_names() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "mimetypes"
# dimension = "behavior"
# case = "keyword_argument_names"
# subject = "mimetypes.MimeTypes"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_mimetypes.py"
# status = "filled"
# ///
"""mimetypes.MimeTypes: the public keyword-argument names url=/type=/strict= are part of the API: guess_type(url=, strict=), guess_extension(type=, strict=), guess_all_extensions(type=, strict=)"""
import mimetypes

db = mimetypes.MimeTypes()

# Keyword-argument names: url=, type=, strict=.
assert db.guess_type(url="foo.html", strict=True) == ("text/html", None), "kw url"
assert db.guess_all_extensions(type="image/jpg", strict=True) == [], "kw type strict"
assert db.guess_extension(type="image/jpg", strict=False) == ".jpg", "kw type loose"
print("keyword_argument_names OK")
"###);
    assert_output(&out, r###"keyword_argument_names OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/mimetypes/mime_types_test_case__test_preferred_extension_uc2597b5.py`.
#[test]
fn test_gen_behavior_std_libs_mimetypes_mime_types_test_case__test_preferred_extension_uc2597b5() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "mimetypes"
# dimension = "behavior"
# case = "mime_types_test_case__test_preferred_extension_uc2597b5"
# subject = "cpython.test_mimetypes.MimeTypesTestCase.test_preferred_extension"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_mimetypes.py"
# status = "filled"
# ///
import io
import mimetypes
import os
import sys
from platform import win32_edition
self_db = mimetypes.MimeTypes()

def check_extensions():
    assert mimetypes.guess_extension('application/octet-stream') == '.bin'
    assert mimetypes.guess_extension('application/postscript') == '.ps'
    assert mimetypes.guess_extension('application/vnd.apple.mpegurl') == '.m3u'
    assert mimetypes.guess_extension('application/vnd.ms-excel') == '.xls'
    assert mimetypes.guess_extension('application/vnd.ms-powerpoint') == '.ppt'
    assert mimetypes.guess_extension('application/x-texinfo') == '.texi'
    assert mimetypes.guess_extension('application/x-troff') == '.roff'
    assert mimetypes.guess_extension('application/xml') == '.xsl'
    assert mimetypes.guess_extension('audio/mpeg') == '.mp3'
    assert mimetypes.guess_extension('image/avif') == '.avif'
    assert mimetypes.guess_extension('image/jpeg') == '.jpg'
    assert mimetypes.guess_extension('image/tiff') == '.tiff'
    assert mimetypes.guess_extension('message/rfc822') == '.eml'
    assert mimetypes.guess_extension('text/html') == '.html'
    assert mimetypes.guess_extension('text/plain') == '.txt'
    assert mimetypes.guess_extension('text/x-rst') == '.rst'
    assert mimetypes.guess_extension('video/mpeg') == '.mpeg'
    assert mimetypes.guess_extension('video/quicktime') == '.mov'
check_extensions()
mimetypes.init()
check_extensions()

print("MimeTypesTestCase::test_preferred_extension: ok")
"###);
    assert_output(&out, r###"MimeTypesTestCase::test_preferred_extension: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/mimetypes/read_mime_types_parses_table.py`.
#[test]
fn test_gen_behavior_std_libs_mimetypes_read_mime_types_parses_table() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "mimetypes"
# dimension = "behavior"
# case = "read_mime_types_parses_table"
# subject = "mimetypes.read_mime_types"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_mimetypes.py"
# status = "filled"
# ///
"""mimetypes.read_mime_types: read_mime_types parses a 'type ext ext' rules file into a {'.ext': 'type'} dict, mapping each extension to its type and skipping '# comment' lines"""
import mimetypes
import os
import tempfile

# read_mime_types parses 'type ext...' lines (and skips '# comments').
text = "application/x-foo  foo bar\n# comment line\napplication/x-baz  baz\n"
with tempfile.NamedTemporaryFile("w", suffix=".types", delete=False) as fh:
    fh.write(text)
    name = fh.name
try:
    table = mimetypes.read_mime_types(name)
finally:
    os.unlink(name)

assert table[".foo"] == "application/x-foo", f".foo = {table.get('.foo')!r}"
assert table[".bar"] == "application/x-foo", f".bar = {table.get('.bar')!r}"
assert table[".baz"] == "application/x-baz", f".baz = {table.get('.baz')!r}"
print("read_mime_types_parses_table OK")
"###);
    assert_output(&out, r###"read_mime_types_parses_table OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/mimetypes/strict_hides_common_types.py`.
#[test]
fn test_gen_behavior_std_libs_mimetypes_strict_hides_common_types() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "mimetypes"
# dimension = "behavior"
# case = "strict_hides_common_types"
# subject = "mimetypes.MimeTypes"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_mimetypes.py"
# status = "filled"
# ///
"""mimetypes.MimeTypes: the non-standard image/jpg alias is hidden under strict (default) but visible under strict=False for both guess_extension and guess_all_extensions"""
import mimetypes

db = mimetypes.MimeTypes()

# image/jpg is a non-standard alias: only visible when strict=False.
assert db.guess_all_extensions("image/jpg", strict=True) == [], "jpg strict"
assert db.guess_all_extensions("image/jpg", strict=False) == [".jpg"], "jpg loose"
assert db.guess_extension("image/jpg", strict=True) is None, "jpg ext strict"
assert db.guess_extension("image/jpg", strict=False) == ".jpg", "jpg ext loose"
print("strict_hides_common_types OK")
"###);
    assert_output(&out, r###"strict_hides_common_types OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/mimetypes/types_map_values_roundtrip.py`.
#[test]
fn test_gen_behavior_std_libs_mimetypes_types_map_values_roundtrip() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "mimetypes"
# dimension = "behavior"
# case = "types_map_values_roundtrip"
# subject = "mimetypes.guess_extension"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_mimetypes.py"
# status = "filled"
# ///
"""mimetypes.guess_extension: every value in the default types_map round-trips: each registered MIME type has at least one guess_extension result"""
import mimetypes

for mime in mimetypes.types_map.values():
    assert mimetypes.guess_extension(mime) is not None, f"no extension for {mime!r}"
print("types_map_values_roundtrip OK")
"###);
    assert_output(&out, r###"types_map_values_roundtrip OK
"###);
}
