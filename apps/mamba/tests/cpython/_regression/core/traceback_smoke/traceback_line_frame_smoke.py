# Traceback line/frame smoke — #2791.
#
# Covers the minimum traceback information that Python attaches to an
# exception:
#
#   exc.__traceback__   the linked list of frame records. Each node
#                       carries tb_frame (the runtime frame), tb_lineno
#                       (line where the exception passed through that
#                       frame), and tb_next (the next frame down).
#   tb_frame.f_code     the code object — gives co_name (function name)
#                       and co_filename (source path).
#
# Out of scope: exact CPython traceback formatting (the prose produced
# by traceback.format_exc()). We only assert that the structured frame
# data is present and consistent.
#
# Clauses:
#   1. Raising from a known helper produces a traceback whose
#      INNER-MOST frame names that helper.
#   2. The traceback chain contains at LEAST two frames (caller +
#      helper) — proves callers stay on the chain.
#   3. The function-name sequence reads outer-to-inner (caller-first)
#      and matches the call stack.
#   4. Line numbers along the chain are positive and the inner frame's
#      lineno equals the source line of the `raise`.
#   5. tb_next of the deepest frame is None — chain terminates cleanly.
#   6. co_filename of every frame ends with this fixture's basename —
#      avoids brittle absolute-path assertions while still proving the
#      frames came from this file.
#
# Every print line tagged `[traceback]` so failure output names
# traceback semantics.


import os
import sys


def inner_raise():
    # The exact line of this `raise` is captured below as RAISE_LINE.
    raise ValueError("from-inner")


# Source line of the `raise` above. Update if you move the raise.
RAISE_LINE = 41


def middle_call():
    inner_raise()


def outer_call():
    middle_call()


def walk_frames(tb):
    """Walk tb_next chain and return (name, lineno, filename) tuples."""
    out = []
    cur = tb
    while cur is not None:
        code = cur.tb_frame.f_code
        out.append((code.co_name, cur.tb_lineno, code.co_filename))
        cur = cur.tb_next
    return out


# Trigger and capture.
try:
    outer_call()
    frames = []
    exc_for_clause5 = None
except ValueError as exc:
    frames = walk_frames(exc.__traceback__)
    exc_for_clause5 = exc


# Clause 1: inner-most frame names `inner_raise`.
inner_name = frames[-1][0] if frames else None
print("[traceback] clause-1 innermost-name:", inner_name)


# Clause 2: at least two frames on the chain.
print("[traceback] clause-2 frame-count-ge-2:", len(frames) >= 2)


# Clause 3: function-name sequence is outer-to-inner.
names = [f[0] for f in frames]
print("[traceback] clause-3 name-sequence:", names)
# The top frame is module-level (`<module>`); the deepest is
# `inner_raise`. We assert the suffix is the user-call chain.
expected_suffix = ["outer_call", "middle_call", "inner_raise"]
print(
    "[traceback] clause-3 has-call-chain:",
    names[-len(expected_suffix) :] == expected_suffix,
)


# Clause 4: line numbers are positive and the inner frame's lineno
# equals RAISE_LINE.
linenos = [f[1] for f in frames]
print("[traceback] clause-4 linenos-positive:", all(n > 0 for n in linenos))
print("[traceback] clause-4 inner-lineno-matches:", frames[-1][1] == RAISE_LINE)


# Clause 5: tb_next of the deepest frame is None.
tb_cursor = exc_for_clause5.__traceback__ if exc_for_clause5 is not None else None
while tb_cursor is not None and tb_cursor.tb_next is not None:
    tb_cursor = tb_cursor.tb_next
print(
    "[traceback] clause-5 deepest-tb-next-none:",
    tb_cursor is not None and tb_cursor.tb_next is None,
)


# Clause 6: every frame's filename ends with this file's basename.
this_basename = os.path.basename(__file__)
all_match = all(os.path.basename(f[2]) == this_basename for f in frames)
print("[traceback] clause-6 every-frame-from-this-file:", all_match)
print("[traceback] clause-6 basename:", this_basename)

# Reference sys to keep import meaningful in case future clauses add
# sys.exc_info() checks; harmless no-op for now.
_ = sys.version_info[:2]
