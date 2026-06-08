---
change: agent-pyo3
group: agent-pyo3-bindings
date: 2026-03-18
status: answered
---

# Pre-Clarifications

### Q1: General
- **Question**: Are all dependent crates ready?
- **Answer**: Yes. All 4 SDD agents (RestructureIssueAgent, ReferenceContextAgent, ChangeSpecAgent, CodeAgent), ReviewAgent, CRRCycle, and integration extensions are implemented with 173 tests passing.

### Q2: General
- **Question**: Tool executor callable API?
- **Answer**: Accept async callable: async def handler(args: dict) -> str. The tool name and description are registered separately. Signature: register_python_tool(name: str, description: str, parameters: dict, handler: Callable). Handler receives parsed args dict, returns string result.

### Q3: General
- **Question**: complete_structured return type?
- **Answer**: Return Python dict (deserialized JSON). Schema passed as dict (JSON Schema). This matches Python conventions — users work with dicts, not Rust types.

### Q4: General
- **Question**: Crate structure?
- **Answer**: Follow the standard pyo3_bindings/ layout from CLAUDE.md: mod.rs (module entry), types.rs (Py* wrappers), methods.rs (method impls). Split by domain if needed: providers.rs, agents.rs, tools.rs.

### Q5: General
- **Question**: Stub generation timing?
- **Answer**: Manual step after maturin develop. Stubs checked into git so consumers get type hints without building from source. Run: cclab lens gen-stub.

