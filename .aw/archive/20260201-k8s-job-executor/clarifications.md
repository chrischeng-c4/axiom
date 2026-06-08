---
change: k8s-job-executor
date: 2026-01-31
---

# Clarifications

## Q1: 整合模式
- **Question**: K8s Job Executor 應該如何與現有 meteor worker 整合？
- **Answer**: Worker 統一處理，根據 task 的 executor 標記決定 in-process 或 spawn K8s Job。Worker 不需要等待 Job 完成，spawn 後立即返回繼續處理下一個 task。
- **Rationale**: 保持架構簡單，不需要獨立的 Job Controller。Worker 非阻塞 spawn Job，避免佔用 worker 資源。

## Q2: Image 策略
- **Question**: Task container image 應該如何管理？
- **Answer**: Mixed 模式 - 預設使用 base runner image，可 override 為專屬 image。
- **Rationale**: 提供靈活性，簡單任務用通用 image，特殊需求可指定專屬 image。

## Q3: Result 傳遞
- **Question**: Job 執行結果如何回傳？
- **Answer**: Job 直接寫入 result backend (Ion/Redis)，與現有 normal task 一致。
- **Rationale**: 統一結果儲存方式，簡化架構。Chain 觸發由 Job 自己負責發送下一個 task 到 broker。

## Q4: 資源類型
- **Question**: 需要支援哪些特殊資源類型？
- **Answer**: 完整支援所有 K8s 資源類型：GPU (nvidia.com/gpu)、TPU (google.com/tpu)、自定義擴展資源 (FPGA, NPU 等)，以及 nodeSelector、nodeAffinity、tolerations。
- **Rationale**: K8s 原生支援這些資源調度機制，直接暴露給使用者即可。

## Q5: Queue 策略
- **Question**: Normal task 和 K8s Job task 是否需要分離 queue？
- **Answer**: 預設全部進同一個 queue，使用者可自行配置分離 queue。
- **Rationale**: 保持預設簡單，進階使用者可根據需求自行規劃 queue topology。HPA 配置和 queue 規劃是使用者的責任。

## Q6: Chain 觸發
- **Question**: K8s Job 完成後如何觸發 Chain 的下一個 task？
- **Answer**: Job container 執行完畢後，直接發送下一個 task 到 broker (NATS)。Job container 需要包含 NATS client。
- **Rationale**: Job 自己負責 chain continuation，不需要額外的 coordinator 或 operator。

