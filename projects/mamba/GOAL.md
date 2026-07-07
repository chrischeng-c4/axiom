# GOAL

## Project completion condition (the long-term end state)

mamba 同時滿足以下三軸才算完成：

1. **功能對標 Python 3.12** — CPython `Lib/test/` conformance suite 100% pass on unmodified source。
2. **效能超越 CPython** — perf-primary 套組（int_sum_loop、int_mul_loop、fib_recursive、factorial_recursive、range_sum_loop、generator_sum、list_sort_builtin、string_concat 等 `src/bench/` 內建項）mamba ≥ CPython 3.12，latency 與 memory 兩面。
3. **mambalibs 端到端支援** — cclab 經由 mambalibs 拉回 mamba export，downstream（agkit / score / qc）可直接呼叫 mamba-built Python，無需重 build mamba。

任一軸沒過，goal 沒達成。

## Session-end condition (evaluator-checkable protocol)

Session 結束需要 transcript 同時包含這三段，且第三段嚴格優於第一段：

1. `BASELINE READING [axis=<functional|perf|mambalibs>]` — session 早期（前 3 turn）必須 paste 進對話的一段 fenced code block，內容為下列其中一條命令的完整輸出：
   - **functional**: `cargo test -p mamba --test conformance 2>&1 | tail -5`
   - **perf**: `cargo bench -p mamba -- --quick string_concat 2>&1 | tail -20`（或同等 perf-primary 名稱）
   - **mambalibs**: `ls ../mambalibs/src 2>&1 | wc -l`（或同等 mambalibs export count 指標 — 暫無正式指令時，以 mambalibs repo 內任一可重現的 count 為準）
2. `WORK SUMMARY` — 描述本 session 對選定軸做的具體改動（PR 編號、檔案、commit）。
3. `CURRENT READING [axis=<same>]` — session 結束前必須 paste 的同條命令完整輸出，數值嚴格優於 baseline（functional：通過 test 數更高；perf：對應 benchmark 時間更短；mambalibs：count 更高）。

Evaluator 判定：transcript 同時看到三段 + current > baseline → goal 達成。

**Turn bound**: 若 20 個 turn 內 transcript 無法產生 baseline+current 的 strictly-better 對比（或 30 個 turn 內仍無 work summary），session 自行結束並 surface "STOP: no measurable progress in N turns" 一行。Evaluator 看到該行視同 goal 終結（但不算達成）。

## 不算 goal-satisfier 的工作

- 純文件 PR（CLAUDE.md、GOAL.md、README、COVERAGE-*.md）。
- 只 close 過時 issue、沒推進三軸的工作。
- 只加 test 但 test target 跟三軸無關。
- mamba 端改動但 mambalibs 沒接到、downstream 看不到的 export。
- 不在 `src/bench/` perf-primary 列表上的 micro-benchmark。

## Operating rules

- Session 開頭先決定要推哪一軸，立刻 paste baseline 進對話。
- 一次只推一軸 / 一張票，避免半成品平行展開。
- autocompact 處理 context，不要因 context 提前收尾或跳過驗證。
- 若該軸的 measurement 命令本身不存在，session 的有效工作就是把該命令建出來、產出第一個 baseline reading（這本身屬於該軸的 prerequisite，視為推進）。
- mambalibs 是 downstream sink — mamba export shape 改變時，宣告 issue close 前先確認 mambalibs 是否需要同步接住。
