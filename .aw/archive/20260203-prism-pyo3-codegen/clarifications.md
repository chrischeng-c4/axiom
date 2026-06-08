---
change: prism-pyo3-codegen
date: 2026-01-31
---

# Clarifications

## Q1: 標記方案
- **Question**: 選擇哪種方案來標記要暴露的 API？
- **Answer**: Proc Macro - 使用 #[pyexport] attribute
- **Rationale**: Rust-native、IDE 友好、零配置，與 Rust 生態系統最佳整合

## Q2: 輸出位置
- **Question**: 生成的 binding 要放在哪裡？
- **Answer**: python/cclab/ 目錄，按 crate 分子目錄 (如 python/cclab/ion/)
- **Rationale**: 保持與現有 Python 套件結構一致，每個 crate 對應一個子模組

## Q3: 整合方式
- **Question**: 這個 codegen 要整合到哪裡？
- **Answer**: cclab-prism gen 子命令 (cclab prism gen pyo3)
- **Rationale**: 與現有 prism 工具鏈整合，統一的 CLI 體驗

