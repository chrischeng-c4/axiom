---
change: mamba-p1-bugfix
group: try-except-dict-set-parse
date: 2026-03-28
---

# Requirements

Fix try/except block with dict/set literal triggering parse error. The parser likely confuses { as block start vs dict/set literal in except clause body. Fix parser to correctly handle dict/set literals inside try/except blocks.
