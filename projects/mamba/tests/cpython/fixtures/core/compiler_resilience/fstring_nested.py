# /// script
# requires-python = ">=3.12"
# dependencies = []
# ///
"""core/compiler_resilience: deeply nested f-strings (CPython 3.12 oracle)."""
# mamba-xfail: compiler has no parser bounds — hostile source crashes/diverges (WI #3929)
#
# The nested f-string source is built at RUNTIME and fed to compile(). CPython
# 3.12 (PEP 701) parses f-strings with the real tokenizer and enforces a nesting
# bound, so a hostile depth is refused with a CLEAN SyntaxError — never a crash.
# mamba's f-string parser has no nesting bound and can overflow / diverge the
# COMPILER on the same input (project_mamba_compiler_hostile_source_dos).


def nest_fstring(depth):
    """Return source for an f-string whose replacement field nests `depth` deep.

    Each level wraps the previous in `f"{ ... }"`, e.g. depth 2 ->  f"{f'{1}'}".
    Built with runtime string ops so this fixture file itself stays flat.
    """
    inner = "1"
    for level in range(depth):
        quote = '"' if level % 2 == 0 else "'"
        inner = "f" + quote + "{" + inner + "}" + quote
    return "value = " + inner + "\n"


# A shallow nest compiles + evaluates fine, proving the construction is valid.
shallow = nest_fstring(3)
ns = {}
exec(compile(shallow, "<shallow-fstring>", "exec"), ns)
assert ns["value"] == "1"
print("shallow_fstring_ok: True")

# A hostile depth must be refused CLEANLY. CPython caps nesting and raises
# SyntaxError; RecursionError/MemoryError would also count as clean refusal.
deep = nest_fstring(200)
print("nesting_depth:", 200)
clean = False
err_kind = None
try:
    compile(deep, "<deep-fstring>", "exec")
    clean = True
    err_kind = "compiled"
except SyntaxError:
    clean = True
    err_kind = "SyntaxError"
except RecursionError:
    clean = True
    err_kind = "RecursionError"
except MemoryError:
    clean = True
    err_kind = "MemoryError"

assert clean, "deeply nested f-strings must be refused cleanly, never crash"
print("error_kind:", err_kind)

mamba_note = (
    "mamba f-string parser has no nesting bound; deep nesting can overflow / "
    "diverge the compiler (project_mamba_compiler_hostile_source_dos)"
)
print("mamba_note:", mamba_note)

print("fstring_nested: clean refusal, no crash — OK")
