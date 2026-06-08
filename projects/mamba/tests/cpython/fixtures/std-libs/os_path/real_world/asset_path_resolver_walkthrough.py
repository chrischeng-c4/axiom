# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "os_path"
# dimension = "real_world"
# case = "asset_path_resolver_walkthrough"
# subject = "os.path"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_posixpath.py"
# status = "filled"
# ///
"""os.path: a static-asset/build-tool resolver drives join/normpath/isabs/splitext/basename/dirname/relpath/commonpath together, then materializes files in a TemporaryDirectory and confirms exists/isfile/isdir/getsize/samefile against the synthesized layout"""
import os
import os.path
import tempfile

# A static-asset resolver, as found in build tools and serving stacks:
# given a project root and a request path, it normalizes, classifies by
# extension, and resolves to an on-disk file. We exercise the pure-string
# path algebra first, then materialize the layout and probe the filesystem.

with tempfile.TemporaryDirectory() as root:
    # 1. Synthesize a static-asset tree: <root>/static/{css,js}/*.
    static = os.path.join(root, "static")
    css_dir = os.path.join(static, "css")
    js_dir = os.path.join(static, "js")
    os.makedirs(css_dir)
    os.makedirs(js_dir)

    assets = {
        os.path.join(css_dir, "site.css"): b"body{margin:0}",
        os.path.join(js_dir, "app.js"): b"console.log(1)",
        os.path.join(js_dir, "vendor.min.js"): b"/*min*/",
    }
    for path, payload in assets.items():
        with open(path, "wb") as fh:
            fh.write(payload)

    # 2. Resolve a request path the way a server would: join under static/,
    #    collapse any '..' / '.' traversal, and confirm it stays absolute.
    request = "css/../css/./site.css"
    resolved = os.path.normpath(os.path.join(static, request))
    assert os.path.isabs(resolved), f"resolved must be absolute = {resolved!r}"
    assert resolved == os.path.join(css_dir, "site.css"), f"normpath resolve = {resolved!r}"

    # 3. Classify the resolved asset by extension and components.
    name, ext = os.path.splitext(resolved)
    assert ext == ".css", f"ext = {ext!r}"
    assert os.path.basename(resolved) == "site.css", "basename"
    assert os.path.dirname(resolved) == css_dir, "dirname"

    # 4. A multi-dot minified asset only sheds its final extension.
    minified = os.path.join(js_dir, "vendor.min.js")
    mname, mext = os.path.splitext(minified)
    assert mext == ".js" and mname.endswith("vendor.min"), f"min splitext = {(mname, mext)!r}"

    # 5. relpath gives the public URL path of an asset under the doc root.
    url_path = os.path.relpath(resolved, static)
    assert url_path == "css/site.css", f"relpath url = {url_path!r}"

    # 6. commonpath confirms two assets share the static/ root.
    common = os.path.commonpath([resolved, minified])
    assert common == static, f"commonpath = {common!r}"

    # 7. Filesystem probes against the materialized layout.
    assert os.path.exists(resolved), "resolved asset exists"
    assert os.path.isfile(resolved), "resolved asset is a file"
    assert os.path.isdir(css_dir), "css dir is a directory"
    assert not os.path.isfile(css_dir), "css dir is not a file"
    assert os.path.getsize(resolved) == len(b"body{margin:0}"), "getsize matches payload"
    assert os.path.samefile(resolved, os.path.abspath(resolved)), "samefile via abspath"
    assert not os.path.exists(os.path.join(static, "css", "missing.css")), "missing asset"

print("asset_path_resolver_walkthrough OK")
