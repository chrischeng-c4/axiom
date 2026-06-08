"""Hot-loop bench for `code.InteractiveInterpreter` /
`code.InteractiveConsole` / `code.interact` / `code.compile_command`
module-attribute reads (#1261).

End-user scenario: code-using REPL / sandbox code re-resolves
`InteractiveInterpreter` (eval engine), `InteractiveConsole`
(REPL frontend), `interact` (single-shot entry), and
`compile_command` (incremental parse) on every call site.
Per-call attribute resolution goes through the `code` module's
attribute table on each call site. That per-call module-attribute
quadruple-read is the workload measured here.

Tier: `runtime module-attr read` (target mamba/cpython <= 1.0x).

Workload: 20_000 paired reads of `InteractiveInterpreter`,
`InteractiveConsole`, `interact`, and `compile_command` per
iteration (ITERS scaled so 4 attrs x 20_000 = ~80k attr-reads
per run).

DoD: exits 0 under both CPython and mamba; the cross-runtime bench
marker on stderr) and reports the ratio. Floor is 1.0x per #1265
Goal 2.
"""

import code


_II_BASELINE = code.InteractiveInterpreter
_IC_BASELINE = code.InteractiveConsole
_I_BASELINE = code.interact
_CC_BASELINE = code.compile_command

ITERS = 20_000

acc = 0
for _ in range(ITERS):
    a = code.InteractiveInterpreter
    b = code.InteractiveConsole
    c = code.interact
    d = code.compile_command
    if (a is _II_BASELINE
            and b is _IC_BASELINE
            and c is _I_BASELINE
            and d is _CC_BASELINE):
        acc = acc + 1

assert acc - ITERS == 0, f"code module-attribute read acc drift: acc={acc} expected={ITERS}"
print("code_type_read_hot:", acc)
