---
change: multi-agent-fallback
date: 2026-01-29
---

# Clarifications

## Q1: Current Gap
- **Question**: 目前配置缺少什麼？
- **Answer**: 目前 config.toml 只定義了各 agent (gemini, codex, claude) 的 models，但沒有設定 workflow 階段應該使用哪個 agent
- **Rationale**: 需要新增 workflow stage 到 agent 的對應配置

## Q2: Workflow Stages
- **Question**: Workflow 階段需要哪些 agent 配置？
- **Answer**: All Stages - proposal, challenge, implementation, review 都需要配置
- **Rationale**: 完整覆蓋所有 workflow 階段，提供最大彈性

## Q3: Config Format
- **Question**: Fallback 配置格式偏好？
- **Answer**: Agent Array per Stage - 每個階段用 array 配置 agents，如 [workflow.proposal] agents = ["gemini", "codex"]
- **Rationale**: 直覺的配置方式，清楚表達每個階段的 agent 優先順序

## Q4: Quota Detection
- **Question**: Quota 用完的判斷方式是什麼？
- **Answer**: Error Message Parsing - 從 CLI 回傳的錯誤訊息判斷 quota exceeded
- **Rationale**: 最直接的方式，不需要額外 API 呼叫，且各 provider 的 rate limit 錯誤訊息都有明確模式可匹配

