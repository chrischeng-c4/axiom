# Custom exception subclass — #2793.
#
# Covers user-defined exception subclasses and the `except` matching
# rules that govern how they're caught:
#
#   - subclass(BaseClass)   `except BaseClass:` matches the subclass.
#   - BaseException.args    a tuple of the constructor arguments; the
#                           default __str__ formats it.
#   - except ordering       Python tries except clauses top-to-bottom;
#                           the FIRST matching clause wins, so the
#                           more-specific subclass MUST come before
#                           the parent class to be reached.
#   - tuple-of-types        `except (A, B):` matches an exception
#                           that's an instance of A OR B.
#
# Clauses:
#   1. Subclass `except` matching — catching with the parent type
#      catches a raised subclass instance.
#   2. .args storage — `raise MyErr(1, 2, 3)` -> args == (1, 2, 3),
#      and str() of single-arg exception is the arg repr-as-str.
#   3. except ordering — more-specific FIRST is reached; specific
#      AFTER parent is unreachable, parent clause runs.
#   4. except tuple — `except (A, B):` matches A-instance and
#      B-instance through the same clause.
#   5. isinstance walk — the runtime check `isinstance(exc, Parent)`
#      mirrors the except matching rule.
#   6. Custom __str__ overrides the default args-tuple formatting.
#
# Every print line tagged `[custom-exc]` so failure output names
# custom-exception semantics.


class AppError(Exception):
    """Base for the app's exceptions."""

    pass


class NotFoundError(AppError):
    pass


class PermissionDeniedError(AppError):
    pass


class FatalError(AppError):
    def __str__(self):
        # Override the default args-tuple formatting.
        return f"fatal[{','.join(str(a) for a in self.args)}]"


# Clause 1: subclass matched by parent except.
caught_via_parent = None
try:
    raise NotFoundError("missing-thing")
except AppError as exc:
    caught_via_parent = type(exc).__name__
print("[custom-exc] clause-1 caught-via-parent:", caught_via_parent)


# Clause 2: args storage + default str().
try:
    raise NotFoundError("widget", 42, "tenant-1")
except NotFoundError as exc:
    print("[custom-exc] clause-2 args:", exc.args)
    print("[custom-exc] clause-2 args-len:", len(exc.args))

try:
    raise NotFoundError("solo")
except NotFoundError as exc:
    # Default __str__ for single-arg is the arg itself as str.
    print("[custom-exc] clause-2 str-single:", str(exc))


# Clause 3: except ordering — specific BEFORE parent is reached.
order_trace = []
try:
    raise NotFoundError("first-clause-wins")
except NotFoundError:
    order_trace.append("notfound-first")
except AppError:
    order_trace.append("appbase-second")
print("[custom-exc] clause-3 specific-first:", order_trace)

# When specific comes AFTER parent it is unreachable — parent matches.
# Indirect the subclass through a name so pyright can't statically
# prove the second clause is unreachable.
from typing import Any

_subclass_ref: Any = NotFoundError
order_trace2 = []
try:
    raise NotFoundError("parent-wins-when-first")
except AppError:
    order_trace2.append("appbase-first")
except _subclass_ref:
    order_trace2.append("notfound-second-unreachable")
print("[custom-exc] clause-3 parent-first:", order_trace2)


# Clause 4: except tuple — matches A OR B.
def try_tuple(raised):
    try:
        raise raised
    except (NotFoundError, PermissionDeniedError) as exc:
        return type(exc).__name__


print("[custom-exc] clause-4 tuple-not-found:", try_tuple(NotFoundError("x")))
print(
    "[custom-exc] clause-4 tuple-permission:",
    try_tuple(PermissionDeniedError("y")),
)


# Clause 5: isinstance mirrors except matching.
nf = NotFoundError("for-isinstance")
print("[custom-exc] clause-5 isinstance-AppError:", isinstance(nf, AppError))
print("[custom-exc] clause-5 isinstance-Exception:", isinstance(nf, Exception))
print(
    "[custom-exc] clause-5 isinstance-PermissionDenied:",
    isinstance(nf, PermissionDeniedError),
)


# Clause 6: custom __str__ override.
try:
    raise FatalError("disk", "full")
except FatalError as exc:
    print("[custom-exc] clause-6 custom-str:", str(exc))
    # .args remains the raw tuple — __str__ is the only thing
    # overridden.
    print("[custom-exc] clause-6 args-untouched:", exc.args)
