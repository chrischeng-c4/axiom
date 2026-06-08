# /// script
# requires-python = ">=3.12"
# dependencies = []
# ///
"""core/compiler_resilience: deeply nested parentheses (CPython 3.12 oracle)."""
# mamba-xfail: compiler has no parser bounds — hostile source crashes/diverges (WI #3929)
#
# The hostile source is CONSTRUCTED at runtime (never written literally) so that
# CPython's own compiler does not choke while parsing THIS fixture. We then feed
# the string to compile() and assert CPython rejects it CLEANLY (a catchable
# exception), never a SIGSEGV. mamba's parser has no recursion bound, so the same
# input overflows the native stack / diverges (project_mamba_compiler_hostile_source_dos).
import sys

# Nest far beyond the interpreter recursion limit so the recursive-descent
# parser must bottom out into a guard rather than blow the C stack.
depth = sys.getrecursionlimit() * 4
src = "(" * depth + "1" + ")" * depth
print("recursion_limit:", sys.getrecursionlimit())
print("nesting_depth:", depth)

# CPython 3.12 reports one of a small, well-defined set of CLEAN errors here:
#   - RecursionError                       (parser recursion guard)
#   - SyntaxError "too many nested parentheses"  (tokenizer paren-depth cap)
#   - MemoryError                          (allocation refusal)
# Any of these is a PASS: the point is "clean refusal, no crash".
clean = False
err_kind = None
try:
    compile(src, "<hostile>", "eval")
    # If it somehow compiled, that is still a non-crash; record it.
    clean = True
    err_kind = "compiled"
except RecursionError:
    clean = True
    err_kind = "RecursionError"
except MemoryError:
    clean = True
    err_kind = "MemoryError"
except SyntaxError as exc:
    clean = True
    err_kind = "SyntaxError"
    # When it is a SyntaxError, CPython's message is about paren nesting.
    msg = str(exc).lower()
    assert "nested" in msg or "parenthes" in msg or "many" in msg, msg

assert clean, "deep paren nesting must be refused cleanly, never crash"
assert err_kind is not None
print("error_kind:", err_kind)

# A modestly nested expression of the SAME shape still compiles + evaluates,
# proving the construction itself is valid Python (only the depth is hostile).
small = "(" * 20 + "7" + ")" * 20
assert eval(compile(small, "<ok>", "eval")) == 7
print("small_nesting_ok: True")

print("deep_nesting: clean refusal, no crash — OK")
