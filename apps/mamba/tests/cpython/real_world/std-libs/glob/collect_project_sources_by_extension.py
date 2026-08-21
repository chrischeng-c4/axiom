# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "glob"
# dimension = "real_world"
# case = "collect_project_sources_by_extension"
# subject = "glob.glob"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""glob.glob: a realistic build-tool flow: lay out a small project tree in a temp dir, then use recursive glob('**/*.py') to collect every Python source while skipping data/doc files, mirroring how a packager gathers inputs"""
import glob
import os
import tempfile

# A packager scans a project tree and collects only the Python sources.
tree = [
    "setup.py",
    "README.md",
    os.path.join("pkg", "__init__.py"),
    os.path.join("pkg", "core.py"),
    os.path.join("pkg", "data.json"),
    os.path.join("pkg", "sub", "helper.py"),
    os.path.join("tests", "test_core.py"),
    os.path.join("docs", "guide.rst"),
]

with tempfile.TemporaryDirectory() as root:
    for rel in tree:
        full = os.path.join(root, rel)
        os.makedirs(os.path.dirname(full), exist_ok=True)
        with open(full, "w") as fh:
            fh.write("")

    matches = glob.glob(os.path.join(root, "**", "*.py"), recursive=True)
    sources = sorted(os.path.relpath(p, root) for p in matches)

    expected = sorted([
        "setup.py",
        os.path.join("pkg", "__init__.py"),
        os.path.join("pkg", "core.py"),
        os.path.join("pkg", "sub", "helper.py"),
        os.path.join("tests", "test_core.py"),
    ])
    assert sources == expected, f"collected sources = {sources!r}"

    # Non-.py files (json, md, rst) are skipped.
    assert all(name.endswith(".py") for name in sources), "only .py collected"

print("collect_project_sources_by_extension OK")
