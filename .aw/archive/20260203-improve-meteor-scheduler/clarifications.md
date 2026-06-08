# Clarifications: improve-meteor-scheduler

## Q1: Schedule Storage

**Topic**: Storage Strategy

**Question**: Scheduler 要如何儲存 periodic task 的定義？

**Answer**: Config-based (like Celery Beat)

**Rationale**: Celery Beat 使用 Python config (CELERYBEAT_SCHEDULE dict) 定義 schedules，這是最常見且易於版本控制的方式。支援：
- Python decorator (`@app.periodic_task`)
- Config dict in code
- 可選的 runtime override via Ion (動態修改)

## Q2: Schedule Types

**Topic**: Scheduling Methods

**Question**: 需要支援哪些排程方式？

**Answer**: Crontab + Interval

**Rationale**:
- **Crontab**: 標準 cron 語法 (`*/5 * * * *`)，支援 minute/hour/day/month/weekday
- **Interval**: 固定間隔 (`every 30s`, `every 5m`)
- 這兩種覆蓋 95% 的使用場景

## Q3: Distributed Locking

**Topic**: Execution Guarantee

**Question**: 需要 distributed locking 確保單一執行嗎？

**Answer**: Yes, use Ion locks

**Rationale**: 在多 instance 部署時，使用 cclab.ion 的分散式鎖確保同一個 periodic task 只會被一個 scheduler 執行。這和 celery-redbeat 的做法類似。

## Q4: Management Interface

**Topic**: Task Management

**Question**: 需要 Web UI 管理介面嗎？

**Answer**: CLI + API only

**Rationale**: 先實作 CLI 和 REST API 進行 task 管理（list, pause, resume, trigger），Web UI 可以之後再加。這樣可以更快交付核心功能。
