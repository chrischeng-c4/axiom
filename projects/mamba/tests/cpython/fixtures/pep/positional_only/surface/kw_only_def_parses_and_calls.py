# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "pep"
# lib = "positional_only"
# dimension = "surface"
# case = "kw_only_def_parses_and_calls"
# subject = "*"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""*: a keyword-only signature (def f(*, host, port)) parses and is callable when every argument is passed by keyword"""

# Every parameter after a bare `*` is keyword-only.
def _connect(*, host: str, port: int) -> str:
    return f"{host}:{port}"

assert _connect(host="localhost", port=8080) == "localhost:8080", _connect(host="localhost", port=8080)

print("kw_only_def_parses_and_calls OK")
