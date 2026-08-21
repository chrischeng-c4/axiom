# /// script
# requires-python = ">=3.12"
# dependencies = []
# ///
# Each call binds a FRESH *args tuple and **kwargs dict; mutating the
# kwargs dict in one call must not leak into the next call.


def collect(**kwargs):
    kwargs["seen"] = True       # mutate the per-call dict
    return sorted(kwargs.items())


# Second call starts clean: the "seen" key from call 1 does not persist.
print(collect(a=1))
print(collect(b=2))


def grow(*args):
    return len(args)


# *args length reflects only the current call's positionals.
print(grow())
print(grow(1, 2, 3))
print(grow())


# **kwargs is a plain dict supporting normal dict ops.
def as_dict(**kw):
    out = dict(kw)
    out["extra"] = len(kw)
    return sorted(out.items())


print(as_dict(x=1, y=2))


# Building a call from a dict does not alias that dict into **kwargs.
def keys(**kw):
    kw.clear()
    return "cleared"


src = {"p": 1, "q": 2}
print(keys(**src))
print(sorted(src.items()))      # src untouched by the callee's clear()
