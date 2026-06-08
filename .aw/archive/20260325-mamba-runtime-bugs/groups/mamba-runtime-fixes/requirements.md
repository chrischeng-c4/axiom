---
change: mamba-runtime-bugs
group: mamba-runtime-fixes
date: 2026-03-24
---

# Requirements

Fix 5 runtime bugs:
1. Semicolons as statement separator — parser rejects `;` between statements
2. ZeroDivisionError — integer `//` floor division by zero doesn't raise exception
3. @decorator return value — decorated function returns None instead of result
4. Nested f-string — inner f-string expression value lost
5. json.dumps returns None — module function call convention incomplete
