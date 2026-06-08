---
change: mermaid-plus
date: 2026-01-29
---

# Clarifications

## Q1: Target Crate
- **Question**: Mermaid+ 功能應該實作在哪個 crate？
- **Answer**: cclab-aurora，prism 和 genesis 引用 aurora
- **Rationale**: aurora 是基礎圖表生成 crate，Mermaid+ 作為擴展的 state diagram 功能應該在此實作，上層 crates 透過依賴引用

## Q2: 技術方向
- **Question**: 應該整合 XState 還是擴展 Mermaid？
- **Answer**: 擴展 Mermaid，不使用 XState
- **Rationale**: 保持簡單，專注於擴展 Mermaid state diagram 的語義化能力，而非引入外部 state machine 規範

## Q3: 功能範圍
- **Question**: 需要支援哪些 Mermaid+ 功能？
- **Answer**: 語義化 actions/guards、YAML frontmatter 輸出、嵌套狀態增強、驗證機制
- **Rationale**: 這四個功能構成完整的 Mermaid+ 擴展：結構化定義（actions/guards）、機器可讀輸出（YAML）、複雜狀態支援（嵌套）、品質保證（驗證）

## Q4: 現有程式碼
- **Question**: 如何處理 cclab-prism 中現有的 Mermaid+ 實作？
- **Answer**: 遷移核心邏輯到 aurora，prism 改為引用 aurora
- **Rationale**: 避免重複實作，將 prism/spec/statemachine/ 中的核心邏輯（不含 XState 部分）遷移到 aurora/diagrams/

