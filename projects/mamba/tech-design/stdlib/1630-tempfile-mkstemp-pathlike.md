# #1630 — tempfile.mkstemp fd validity + os PathLike signatures

Status: OPEN (p2). Design for implementation.

## Mechanism

Two distinct defects bundled:
1. fd from `tempfile.mkstemp()` fails subsequent os-level operations with
   `Errno 9 (bad file descriptor)` — either mkstemp returns a fake/closed fd,
   or the fd→file-object bridge invalidates it. Runtime bug in tempfile_mod
   (note the module just gained the `__enter__` retain fix `b789a7900` —
   unrelated path, but re-run its fixtures as guards).
2. `os.rmdir`/related reject `PathLike` arguments at COMPILE time — checker
   signature says `str` where CPython accepts `os.PathLike` — type-system
   false positive; fix the signature (path-accepting os functions take
   `str | bytes | os.PathLike`), per `1595-ingress-overwalling-shapes.md`.

## Invariant

An fd returned by mkstemp is open and usable (write/close). Path-accepting
os APIs accept `__fspath__`-bearing objects everywhere str is accepted.

## Verification contract

mkstemp round-trip probe (mkstemp → os.write → os.close → read back content)
byte-identical vs oracle; PathLike probe (`os.rmdir(Path(...))`-shape) passes
compile and runs; #1627's tempfile fixtures stay green 5/5; tempfile+os dir
sweeps no regressions; gate no worse.
