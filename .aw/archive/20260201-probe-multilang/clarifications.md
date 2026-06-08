---
change: probe-multilang
date: 2026-02-01
---

# Clarifications

## Q1: Runner Strategy
- **Question**: Rust/TS 測試要如何執行？整合現有工具還是自建 runner？
- **Answer**: Rust 用 cargo test（整合現有），TypeScript 和 Python 一樣自建 runner
- **Rationale**: Rust 生態系的 cargo test 已經很成熟且整合 Rust 工具鏈，整合它可以獲得最佳相容性。TS 生態系分散（jest/vitest/mocha），自建 runner 可提供統一體驗，與 Python 一致。

## Q2: Report Format
- **Question**: 三個語言的測試結果要如何呈現？
- **Answer**: 統一報告 + 各語言詳細報告
- **Rationale**: 跨語言比較需要統一格式，但各語言有專屬指標（如 Rust 的 compile time、TS 的 bundle size）需保留詳細報告。

## Q3: Profiling Depth
- **Question**: 效能測試的 profiling 深度？
- **Answer**: 通用指標 + 語言專屬深度分析
- **Rationale**: 通用指標（latency percentiles, throughput, memory）支援跨語言比較；語言專屬（Rust flamegraph, TS V8 profiler, Python cProfile/GIL）提供深度診斷能力。

## Q4: Security Scope
- **Question**: 安全測試要支援哪些類型？
- **Answer**: 完整套件：Fuzzing + Injection + SAST + Dependency scanning
- **Rationale**: 企業級 QC 需要完整安全測試覆蓋。各語言有對應工具：Rust(cargo-audit, cargo-fuzz), TS(npm audit, eslint-security), Python(bandit, safety)。

## Q5: Language Priority
- **Question**: 優先支援哪個語言？
- **Answer**: Rust 優先
- **Rationale**: cclab 核心是 Rust，probe 本身也是 Rust 寫的，優先支援 Rust 可以最大化 dogfooding 價值並驗證架構。

