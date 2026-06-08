---
change: score-remaining-p1p2
group: codegen-extensions
date: 2026-04-07
---

# Requirements

4) L3 codegen state-machine: stateDiagram-v2 → Python enum + transition function with match arms. 5) L2 codegen FlowchartPlus: nodes with fn signatures → function skeletons + @sdd:implement markers. 6) L2 codegen SequencePlus: actor messages → async call chain skeleton + @sdd:implement markers. All 3 implement SpecIRGenerator trait in sdd/generate/generators/.
