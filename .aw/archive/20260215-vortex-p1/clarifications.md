---
change: vortex-p1
date: 2026-02-15
---

# Clarifications

## Q1: Git Workflow
- **Question**: 這 8 個 P1 issues 要用哪種 git workflow？
- **Answer**: in_place — 直接在目前的 dev/vortex branch 上開發
- **Rationale**: 已在 vortex dev branch 上，無需額外建 branch，減少 context switch

## Q2: Event Bus Async
- **Question**: Event bus (#345) 要支援 async 嗎？還是純同步 queue 就好？
- **Answer**: Sync + Async — 同時支援同步 dispatch 和 async listener
- **Rationale**: 未來擴展性較好，可同時滿足 game loop 內同步使用和外部 async 整合需求

## Q3: Game State Machine
- **Question**: Game state machine (#371) 要支援哪些 state transitions？
- **Answer**: Extended — 加入 Loading、LevelSelect、Victory 等額外 states
- **Rationale**: 完整的 state machine 更貼近實際遊戲需求，包含 Loading、LevelSelect、Victory 等狀態

## Q4: Tower AI Pattern
- **Question**: Agent tower AI (#334) 的 BT/FSM 偏好？
- **Answer**: Both — 提供兩種抽象，讓不同塔類型選用適合的 AI 模式
- **Rationale**: BT 適合複雜可組合行為，FSM 適合固定模式，兩者並存提供最大靈活性

