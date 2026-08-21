# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "compileall"
# dimension = "behavior"
# case = "optimize_list_builds_one_pyc_per_level"
# subject = "compileall.compile_file"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_compileall.py"
# status = "filled"
# ///
"""compileall.compile_file: optimize=[0, 2] builds one tagged .pyc per requested level in a single call (level 0 and level 2 present, level 1 absent), and the optimization tag is embedded in the filename"""
import compileall
import importlib.util
import os
import tempfile

# The cache filenames are queried so the interpreter version tag is never
# hardcoded.
with tempfile.TemporaryDirectory() as d:
    script = os.path.join(d, "opt.py")
    with open(script, "w") as f:
        f.write("a = 0\n")
    pyc = {0: importlib.util.cache_from_source(script, optimization=""),
           1: importlib.util.cache_from_source(script, optimization=1),
           2: importlib.util.cache_from_source(script, optimization=2)}
    ok = compileall.compile_file(script, quiet=2, optimize=[0, 2])
    assert ok, "optimize list compile succeeds"
    assert os.path.isfile(pyc[0]), "level 0 .pyc produced"
    assert os.path.isfile(pyc[2]), "level 2 .pyc produced"
    assert not os.path.isfile(pyc[1]), "level 1 .pyc not requested"
    # The optimization tag is embedded in the filename.
    assert ".opt-2." in os.path.basename(pyc[2]), os.path.basename(pyc[2])
print("optimize_list_builds_one_pyc_per_level OK")
