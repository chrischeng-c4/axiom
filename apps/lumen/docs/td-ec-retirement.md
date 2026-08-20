# TD/EC 退場戰役 — Lumen 依賴閉包

**這份文件是這場戰役的唯一狀態來源。** 任何新的 agent context 開場先讀這一份。

**它取代 `apps/lumen/docs/td-ec-campaign.md`（已刪，內容存檔於 session scratchpad
`td-ec-campaign.VOID.md`）。** 那份文件的目標是把 TD/EC 轉成 full-typed Python
DDD（epic #3364）；本文件的目標是把 TD/EC **整棵刪掉**。兩者互斥，以本文件為準。
舊文件作廢的三個理由：

1. `CLAUDE.md` 現行敘述已宣告 `tech-design/` 與 `external-contracts/` 不是 write
   root、不是 authoring surface，且「Python spec 模型與 `src/cases/*.py` verifier
   已退役 — 永不新增」。
2. `.github/workflows` 對 `td-ec-verify` / `tech-design` 是 **0 命中** —— 那 10 個
   gate 今天沒有任何自動化消費者，只有人手動跑。
3. 新工作已經只進 Rust：14 個 lib 的 `e2e/` 最後 commit 在 2026-08-13～08-18，
   而 `tech-design/` / `external-contracts/` 幾乎全部凍在 2026-08-06。

舊文件裡仍然為真、且與退場相關的量測，已併入本文件 §3（地雷）。其餘（例如
`topology/matrix.py` 的 import-time monkey-patch、`capability_id` 塌成 2 個值）
隨著那些檔案被刪而失去意義，不再追蹤。

---

## 0. 授權與範圍

**USER DECISION 2026-08-19**，三條凍結決策。改這三條之前先問人。

| # | 決策 | 內容 |
|---|---|---|
| **D1** | **整棵樹刪掉，含 Python** | in-scope 的 15 棵 `tech-design/` 與 15 棵 `external-contracts/` 全部刪除（md、`td.lock`、`ec.lock`、`*.py`、`pyproject.toml`、`uv.lock`、`evidence/`），指向它們的標頭全部清掉。**不是** markdown-only 的分階段版本。 |
| **D2** | **`apps/lumen/tests/` 搬進 `e2e/` 並鎖 `autotests`** | 161 個 `.rs` 搬到 `apps/lumen/e2e/`，`Cargo.toml` 加 `autotests = false` ＋ 每檔一條 `[[test]]`。搬完必須仍編得過、仍跑得起來。 |
| **D3** | **知識落在擁有它的 `.rs` 檔的 `//!` module doc** | 79 份真設計散文的內容併進它描述的那個 `src/**/*.rs` 的頂部 `//!` 區塊。**不**收攏成 project 層的 `docs/design-notes.md` —— 那會長回一份「專門文件」，與 D3 的意圖相反。 |

**in-scope = 15 個 project**：`apps/lumen` ＋ 它 `Cargo.toml` 引用的 14 個 lib
（`build-stamp`、`cli-std`、`metrics-prometheus`、`openapi-codegen`、`peer-tls`、
`raft-core`、`raft-runtime`、`service-auth`、`service-backup`、`service-http`、
`service-k8s`、`service-observability`、`storage-durable`、`transport-h2c`）。

**out of scope**：repo 另外 14 棵 `tech-design/` 與 15 棵 `external-contracts/`
（`apps/arena`、`apps/cap`、`apps/pgpool`、`.aw/` …）。它們不在 lumen 閉包內，
本戰役不碰。**這件事有代價**：`scripts/td-retire-*.py` 是全 repo 共用的工具，
擴充它會影響那些樹將來的退場，所以擴充只能加能力、不能改既有欄位語意。

---

## 1. 量測基線（2026-08-19，`scripts/td-retire-probe.py --census`）

### tech-design（15 棵）

| Project | md | 其中 `semantic/source` 鏡像 | 真散文 | `td.lock` | `*.py` | 指向它的標頭 |
|---|---:|---:|---:|---:|---:|---:|
| `apps/lumen` | 102 | 42 | **60** | 1 | 51 | 591 |
| `libs/openapi-codegen` | 25 | 23 | 2 | 1 | 36 | 92 |
| `libs/service-http` | 17 | 10 | **7** | 1 | 36 | 21 |
| `libs/raft-runtime` | 12 | 10 | 2 | 1 | 48 | 36 |
| `libs/service-backup` | 10 | 8 | 2 | 1 | 28 | 29 |
| `libs/service-k8s` | 10 | 7 | 3 | 1 | 32 | 51 |
| `libs/cli-std` | 9 | 7 | 2 | 1 | 40 | 78 |
| `libs/transport-h2c` | 9 | 8 | 1 | 1 | 22 | 33 |
| `libs/build-stamp` | 0 | 0 | 0 | 0 | 19 | 0 |
| `libs/metrics-prometheus` | 0 | 0 | 0 | 0 | 19 | 0 |
| `libs/peer-tls` | 0 | 0 | 0 | 0 | 21 | 0 |
| `libs/raft-core` | 0 | 0 | 0 | 0 | 25 | 0 |
| `libs/service-auth` | 0 | 0 | 0 | 0 | 25 | 0 |
| `libs/service-observability` | 0 | 0 | 0 | 0 | 25 | 0 |
| `libs/storage-durable` | 0 | 0 | 0 | 0 | 21 | 0 |
| **合計** | **194** | **115** | **79** | **8** | **448** | **931** |

7 棵已經是純 Python（無 md、無 `td.lock`）—— 那是 #3364 前半段的產物。對它們，
`scripts/td-retire-apply.py` 現況會**刪 0 個檔**（見 §3 地雷 2）。

### external-contracts（15 棵）

`*.py` 合計 **292**，非 Python（`pyproject.toml`、`uv.lock`、`ec.lock`、
`ec-author.json`、`ec-review.json`，不含 `evidence/`）合計 **45**，其中
`apps/lumen` 一棵佔 100 py ＋ 17 non-py。`evidence/` 是產物，另計。

### 目前的 `//!` 覆蓋（D3 的地板）

in-scope `src/**/*.rs` 共 **180** 個檔，**166 個已有 `//!` 區塊**，缺 **14 個**：

```
apps/lumen/src/bin/lumen-bench.rs
libs/raft-runtime/src/group.rs
libs/service-backup/src/{policy,destination,sink,runner,source,s3}.rs
libs/service-observability/src/{process,filesystem}.rs
libs/storage-durable/src/{fsync,atomic,framed_log,snapshot_store}.rs
```

這是好消息：D3 不是從零開始寫 180 份設計註解，而是「補 14 個缺口 ＋ 把 79 份散文
的差集併進既有 `//!`」。`apps/lumen/src/operator/reshard_driver.rs` 已經帶 175 行
`//!`，是這個形狀已經行得通的證據。

---

## 2. 階段

每一階段自己有 gate，且**後一階段不得在前一階段落地前開始**。

| # | 階段 | 寫入 | 完成判準 |
|---|---|---|---|
| **S0** | 擴充儀器 | `scripts/td-retire-*.py` | probe 看得見 `.py` 目標與 EC 標頭；gate 能判「整棵消失」；新增 knowledge row |
| **S1** | 知識地板 | 14 個缺 `//!` 的 `.rs` | 14/14 有非空 `//!`；`cargo build` 全綠 |
| **S2** | 知識差集 | in-scope `src/**/*.rs` 的 `//!` | 79 份散文逐份判定：併入 / 判為衍生可棄，兩者都要留判定紀錄 |
| **S3** | `tests/` → `e2e/` | `apps/lumen/{e2e,tests,Cargo.toml}` | `tests/` 空；`autotests = false`；每檔一條 `[[test]]`；`cargo test -p lumen --no-run` 綠 |
| **S4** | 刪樹 ＋ 清標頭 | 30 棵樹 ＋ 帶標頭的檔 | 逐 project bottom-up，每個 project 一輪 gate |
| **S5** | 規則 ＋ ratchet | `.claude/rules/`、`plugins/aw/verification/` | 「source 自己承載設計」成為有消費者會退件的規則 |

**S2 的兩個合法答案都要留紀錄。** 一份散文被判為「衍生、可棄」時，判定理由要寫在
它描述的那個 `.rs` 的 `//!` 裡或本文件 §6，否則下一個 context 無法分辨
「看過並判定可棄」與「沒看就刪了」。

---

## 3. 地雷（全部實測，不是推測）

| # | 地雷 | 觀測 | 處置 |
|---|---|---|---|
| 1 | **編譯期硬相依** | `apps/lumen/tests/capability_stateful_workload_linkage.rs:5` 用 `include_str!` 把 `tech-design/validate/link-stateful-service-workload-claim-to-primary-td-verification.md` 編進 crate，並對它的內容下 5 條 `assert!`。那份 md 是**產品輸入不是文件**，直接刪 → 編譯失敗。 | S3 先處理。該測試驗的是 TD 自己的 bookkeeping（TD 退場後沒有主體），`primary_td_linkage_is_bound` 應隨樹一起退場；同檔的 `active_and_historical_provenance_are_distinct` 有一半在驗 `README.md` 的 `## Capabilities` 潔淨度，那一半留得下來。 |
| 2 | **probe 的 `REF` regex 只認 `tech-design/**.md`** | `scripts/td-retire-probe.py` 的 `REF = r"((?:[\w./\-]*?)tech-design)/[\w./\-]+\.md"`。所以 **(a)** 指向 `external-contracts/` 的標頭 **124 條** 完全不進 `hdr`，也不進 `other`；**(b)** 指向 tree 內 `.py` 的參照同樣看不見。census 對 `apps/lumen/external-contracts` 印 `hdr=0 files=0` 就是這個結構性盲點，不是真的沒有。 | S0 擴充。加 `--include-ec` 之類的能力，**不要改** `hdr` 既有語意（out-of-scope 的樹還要用它）。 |
| 3 | **probe 的 `py` 欄被設計成釘死** | 註解原文：「Held FIXED by every gate … the loudest way to fail this campaign is to delete a tech-design directory wholesale」。D1 正是要 wholesale 刪。 | S0 加一個明確的 whole-tree 模式，讓 `py` 可動；同時要補一條讓「整棵砍掉」**不會變成無鑑別力的綠**的 row（見 §4）。 |
| 4 | **106 個 `tests/*.rs` 反指 EC** | `apps/lumen/tests/` 161 檔中 106 檔含 `external-contracts` 參照 —— 那批 EC claim 直翻的 Rust port（`behavior_*` 76、`stability_*` 13、`security_*` 9、`efficiency_*` 5、`capability_*` 2、`benchmark_*` 1）。 | S3 搬檔時一併清；清掉的是標頭不是斷言。 |
| 5 | **`apps/lumen/Cargo.toml` 沒有 `autotests`** | grep 0 命中 → 161 個 `tests/*.rs` 仍被 `cargo test -p lumen` 自動收。`e2e/` 的 6 個檔是**並存**不是取代。 | S3。 |
| 6 | **37/41 個 `apps/lumen/src/*.rs` 帶 `@spec` 指向源碼鏡像** | 例：`apps/lumen/src/api.rs:1` → `tech-design/semantic/source/apps-lumen-src-api-rs.md#rust-source-unit`。鏡像是 `.rs` 的抄本，**零知識**。 | S4 直接清；不需要 S2 的判定。這也是 931 條標頭裡絕大多數的形狀。 |
| 7 | **`git` 會無限卡死** | 此 checkout 開了 `core.fsmonitor`；daemon 一啞，任何讀 index 的 git 指令永久阻塞。 | 一律 `git -c core.fsmonitor=false`。 |
| 8 | **`python3` 是 3.9 的機器存在** | TD/EC 腳本讀 TOML 需要 `tomllib`（3.11+）。此 checkout 的 `python3` 是 3.12.11，可直接用；跨機器則走 `uv run --python 3.13 --no-project`。 | 兩種啟動字串不要混用在同一個 gate。 |
| 9 | **`cargo build -p lumen` 是空包彈** | `service-k8s` 是 `operator` feature 下的 optional dep，裸 build 不編它、0.5 秒就綠。 | baseline 與驗證一律帶 CI 的 feature 集：`--features "otel operator raft-wal self-update issue delegated-auth"`。 |
| 10 | **`cargo test --lib` 藏得住 build break** | `cfg(test)` 開著時 production 引用 `#[cfg(test)]` 符號會全綠，`cargo build` 才 E0599。 | 每輪都要獨立跑一次 `cargo build`，不能只看 test 綠。 |
| 11 | **`counts` 的預設期望值不可滿足** | `row_counts` 原本用 `ALL_ZEROS` 把 probe 的全部 16 欄釘成 0，但 `ecdirs` / `ecscan` 是「這趟走訪到幾個目錄／幾個檔」的報告欄，樹刪光之後仍然是 141 / 106 → probe 退回這個期望，`counts` 對一輪本該全綠的 round 判紅。 | 已修。`row_counts` 改成解析 count 行，只要求該模式的 `required` 欄歸零（whole-tree 13 欄、markdown 4 欄），`--expect` 降級為選用的精確釘點。修後同一輪 `counts` 從 FAIL 變 `PASS … 13 required columns at zero`。 |
| 12 | **一輪必須從乾淨樹量** | 第一次演練用 `--base HEAD` 在髒樹上跑，`additions` 報 `added_lines=476`、`deletions` 報 `offending_deletions=144 e.g. .claude/settings.json:4` —— 量到的是我自己未 commit 的儀器編輯，對那一輪零斷言。 | 每輪要嘛從乾淨樹跑，要嘛把 `--base` 指到 `git stash create` 產出的 commit（演練腳本走後者，樹不動）。這兩個 row 讀的是**整份 diff**，不是只讀 prefix 底下。 |
| 13 | **whole-tree apply 會刪掉 git 救不回來的東西** | `td-retire-apply.py` 的 `delete_corpus_files` 不套 `probe.SKIP`，所以 `.venv`、`__pycache__`、產生出來的 `evidence/` 一起刪 —— `residue` 要歸零本來就需要這樣。演練已永久失去：`libs/build-stamp` 43 檔、`libs/transport-h2c` 67 檔（tracked 檔與四支儀器都逐 byte 還原）。 | 不修：這些都在 D1 的刪除意圖內且可重新產生。但演練一定要把「不可還原」印出來，只報 `status identical` 會把它蓋掉。 |
| 14 | **非標頭參照沒有任何 row 看得見** | `probe.HEADER` 要求行首是註解記號，所以 `apps/lumen/k8s/operator/crd.yaml` 那 13 條指向退場樹的 pointer 落在 `other` 欄，而 `other` **刻意**不在 `WHOLE_TREE_ZEROS` 內 —— 本文件自己就要指名它退了什麼，否則過不了自己的 gate。 | 新增 `references` row（whole-tree 限定），只掃 project 自己的 source root（`src e2e tests benches examples k8s` ＋ `Cargo.toml`）：在那裡活著的 pointer 是斷指標，不是紀錄。 |
| 15 | **CRD 只能重算不能逐行改** | schemars 把 `///` 折成單一 `description` 字串，13 條裡有 **4 條整條 description 就是 `@spec …`** —— 拔掉 doc comment 不是把那行變短，是讓 `description:` 這個 key 消失。其中 `crd.yaml:424` 指向 `libs/service-k8s/tech-design/`。 | 用它的 renderer 重新產生。因為那是「寫入」，必須與 S4 的刪除放**不同 commit**，否則 `additions` 紅。 |
| 16 | ~~`apps/lumen/scripts/dx-contract-gate.sh` 今天就是紅的~~ **已刪除** | `:89-91` 釘死 7 欄舊值 `want='md=102 lock=1 py=51 hdr=589 files=105 other=19 embed=1'`，整份腳本的量測對象是被 S4 刪掉的樹。 | S4/L4d `6c0c64f3b0` 依 #3708 的處置把它刪了，連同 `apps/lumen/scripts/source_mirrors.py`（它同步的 `tech-design/semantic/source/` 在 L4a 消失）。刪前全 repo grep 兩個檔名，除了它們自己與本文件之外沒有任何呼叫者。 |
| 17 | **#3708 的 `## Never` 與 D1 衝突** | 單子上寫「Never delete a `.py` file under a `tech-design/` tree」，而 D1 就是要刪整棵含 `.py`。 | 以 D1 為準，但動 #3708 之前先改單；留著舊 `## Never` 等於給 reviewer 一條合法的退件理由。 |
| 18 | **儀器 docstring 引用的 fixture 已不在此 checkout** | `td-retire-probe.py` 的 docstring 拿 `apps/agentic-workflow` 當字面事實，而那個 crate 在 `main` 上整個被刪（此 worktree 仍帶著）。 | 讀 docstring 的數字當歷史，不當現況；要現況就重跑 `--census`。 |
| 19 | **`CODEGEN-BEGIN` / `HANDWRITE-BEGIN` marker 一輪過後原封不動** | 它們的 producer 已不存在，但七個 row 沒有一個判它們。 | 不在本戰役範圍（D1 只講 TD/EC 兩棵樹）；留給 S5 的 ratchet。 |
| 20 | **私有 module 的 `//!` 連結，預設沒有被檢查** | `service-backup` 的 module 全是 `mod x;`（私有）＋ root re-export，所以 `cargo doc` 不產生它們的頁面，裡面的 intra-doc link 壞掉也不報。第一次跑 `-D warnings` 是綠的，加上 `--document-private-items` 之後才冒出 9 條 error。 | 驗 `//!` 一律帶 `--document-private-items`。S2 直接受影響：散文搬進 `//!` 之後，連結有沒有效取決於一個平常沒開的旗標。 |
| 21 | **簡寫連結與 `crate::` 兩個方向都會紅** | 同 crate 內 `` [`X`] `` 只在 `X` 於該檔 in scope 時解析，不在 scope 就是 `unresolved link`；反過來 `X` 就定義在本檔時寫 `` [`X`](crate::X) `` 會被判 `redundant explicit link target`。實測：機械地把 12 條都改成簡寫 → 3 條轉成 `unresolved`。 | 不能用「一律加」或「一律不加」的規則，逐條看該檔的 `use`。 |
| 22 | **S1 有一行不是新增** | `libs/raft-runtime/src/state_machine.rs:36` 原本就有一條壞掉的 `` [`applied_index`] ``（同檔第 28 行早就是合格形式），`-D warnings` 下整個 crate 的 rustdoc 因此 abort，我補在 `group.rs` 的連結**那一輪根本沒被檢查**。 | 改成與第 28 行相同的 `` [`applied_index`](RaftStateMachine::applied_index) `` 後轉綠。所以 S1 的 commit 含一條修改行；這不影響 S4 的 `additions`／`deletions`（那兩列量的是 S4 那一輪相對 stash base 的 diff），但 §4「S1/S2 只加行」要照這條讀。 |
| 23 | **標頭本身就帶著散文，而 S4 會把它刪掉** | `// HANDWRITE-BEGIN` 與 `// SPEC-MANAGED:` 上有 `reason="…"` 屬性，那是散文不是標記：15 個 in-scope project 共 **97 條**（86 個檔），其中 **76 條 ≥15 字**，中位數 15–30 字。有些是那個事實在整棵樹裡唯一的敘述，例如 `libs/service-auth/src/k8s/loopback_proxy.rs:1` 的「contract 是 credential **不在**哪裡 —— 不在子行程環境、不在 argv、不在它的位址空間 ——而且失敗模式必須是拒絕而不是過期的 token」。 | §4 的 `knowledge` 列問的是「掉了標頭的檔還有沒有 `//!`」，**從來沒問標頭自己的散文有沒有留下**，所以這是一批沒有任何 consumer 擋得住的知識刪除。處置：S2 逐條判定並寫進 §6 的 ledger，S4 加一列 `reasons` 讀那份 ledger（見 §4）。 |
| 24 | **有三條註解散文指向退場樹，其中兩條今天就已經是錯的** | doc comment 裡帶著退場樹路徑的行有 741 條，其中 739 條是 `/// @spec <path>#<anchor>` 這種純參照形狀（`references` 列抓得到，因為它逐行比對前綴、不看註解形狀）。剩下 **3 條是散文**：`apps/lumen/src/dx.rs:7`、`apps/lumen/tests/llm_command_template_flags_are_live.rs:5`、`apps/lumen/tests/cli_credential_paths_retired.rs:59`。前兩條都說 dx 契約在 `tech-design/interfaces/dx/lumen-dx-contract.md`，**而真正被 `include_str!` 編進 binary 的是 `apps/lumen/src/dx-contract.yaml`**；md 裡那段 fence 與該檔 diff 是**逐位元組相同**，它是副本不是來源。 | 已修：兩處 `//!` 都改成指 `src/dx-contract.yaml`（`cargo build -p lumen --lib` exit 0）。第三條（`external-contracts/` 被刻意排除在 residue 掃描外）等 S3 搬檔時一起改。教訓不是「路徑會過期」，是**一個事實有兩個家的時候，兩個家會分別壞掉**：同一個錯誤在兩個檔各犯一次。 |
| 25 | **14 個測試檔一行 `//!` 都沒有，唯一的散文就在 S4 要刪的那行標頭上** | `apps/lumen/tests/` 9 個（`capability_shared_ownership`、`capability_stateful_workload_linkage`、`cli_convention`、`ec_claim_closure_consistency`、`jieba_bigram_fallback_e2e`、`operator_backup_kubernetes_wiring`、`rig_stateful_adapter`、`shared_stateful_foundations`、`structured_stdout_traceparent`）加 `e2e/` 5 個（`raft-runtime/peer_mtls`、`service-http/{body_limit,otlp_tracing,request_trace_context}`、`service-observability/service_log_jsonl`）。這 14 個檔 `grep -c '//!'` 都是 **0**。 | 剩下的 62 個檔都有 `//!` 承接，這 14 個沒有。D3 的字面要求（知識落在擁有它的 `.rs` 的 `//!`）在這 14 個檔上是**未履行**，不是「已履行但要搬」。S3 搬檔到 `e2e/` 時補上，ledger 記 `deferred:S3`，並在 S3 那一輪加一條機械斷言：這 14 個檔每一個 `//!` 行數 ≥3。 |
| 26 | **只有 3 個 `.rs` 在程式碼層面真的相依退場樹，三種都不一樣** | 逐檔判斷字面值（不是看檔名）：**編譯期 1 個** —— `apps/lumen/tests/capability_stateful_workload_linkage.rs:6` `include_str!` 一份 TD md，刪了就編不過；**執行期 1 個** —— `apps/lumen/tests/ec_claim_closure_consistency.rs:9` 的 `CLAIM_DOCUMENT` 常數，編得過但**整個測試的主題就是那棵樹**；**主動拒絕 1 個** —— `apps/lumen/tests/retired_credential_surface.rs:88,:134` 的兩筆 `Allowance`，而該測試斷言「排除項若已對不到東西就失敗」（`:421-441`）。 | 第三個是**這整輪唯一一個天生就會擋住刪除的 consumer**，不是地雷 —— S4 必須在同一輪把那兩筆 `Allowance`（約 12 行真程式碼）一起刪掉，而那 12 行要**事先宣告**，否則 `deletions` 列會把它報成「多刪了真程式碼」。第二個是 subject-loss：那個測試跟樹一起刪，硬留下來只會留一個對空氣斷言的測試。 |
| 27 | **`pub mod` 上的 `///` 會讓那個檔的 `//!` 在「父模組」的 scope 解析連結** | `libs/service-k8s/src/render.rs` 四個子模組都同時有兩份 doc：宣告行上的 `///` 加各自檔案裡的 `//!`。rustdoc 把合併後的 doc 放在**外層那段所在的模組**解析，所以 `render/deployment.rs` 的 `//!` 寫 `` [`service_deployment`] ``（就在同檔 `:36`）會報 `no item named … in scope`，`` [`super::common`] `` 則報 `no item named 'common' in module 'service_k8s'` —— 那正是從 `render` 看出去的 `super`。逐段拆掉外層 `///` 之後：unresolved link **10 → 8 → 1**。 | 所以「S1 那 8 條 pre-existing 壞連結」有 **7 條根本不是壞連結**（`rbac` 6 條、`projected_token` 1 條），是同一個陷阱；真的寫錯路徑的只有 `certificate/issuer.rs:6` 的 `` [`EphemeralIssuer`] ``（它在 `certificate::ephemeral`，`certificate.rs:49` re-export，已改 `` [`super::EphemeralIssuer`] ``）。處置：知識搬進 `//!` 時，若父模組在宣告行上有 `///`，**必須把外層拆掉**，否則連結是在錯的模組解析 —— 而它只在 `--document-private-items` 下才報（地雷 20）。拆掉外層時要檢查它有沒有帶只存在於那一行的事實：`projected_token` 的 `#2877` 與 `rbac` 的 `#2876,#2889` 只在該行與 S4 要刪的 HANDWRITE 行上出現過，已補進兩檔的 `//!`。`RUSTDOCFLAGS="-D warnings" cargo doc -p service-k8s -p service-observability --no-deps --document-private-items` exit 0。S2 因此也含刪除行（`render.rs` 5 行外層 doc）；對 `additions`／`deletions` 的影響與地雷 22 同一個理由 —— 那兩列量的是 S4 那一輪相對 stash base 的 diff。 |
| 28 | **每一段的 rustdoc gate 只跑自己那幾個 crate，七個 crate 從沒一起跑過** | S1 記的是 `RUSTDOCFLAGS="-D warnings" cargo doc --no-deps`（四個 crate），S2 記的是同一條加 `-p service-k8s -p service-observability`。兩條都真的綠過，但它們的聯集不等於 S1／S2 動過的七個 crate。到 commit 前把六個 crate 一次跑完才發現兩件事：**(a)** `libs/storage-durable/src/snapshot_store.rs:19` 我自己新加的 `` [`atomic_write`](crate::atomic_write) `` 是 `rustdoc::redundant-explicit-links`，從 S1 起就在樹上沒被任何一條 gate 讀到；**(b)** `libs/cli-std/src/lib.rs:26` 連到 `` [`connect`] ``，而 `:41` 的 `pub mod connect;` 被 `#[cfg(feature = "k8s")]` 擋著（2026-06-25 起就是這樣），不開 feature 就報一條**假紅** —— 那是 gate 指令的產物，不是樹的缺陷。 | 全樹一條指令，兩個旗標都是承重的：`RUSTDOCFLAGS="-D warnings" cargo doc --no-deps --document-private-items -p cli-std -p raft-runtime -p service-backup -p service-k8s -p service-observability -p storage-durable --features cli-std/k8s`，exit 0、六個 crate 全 documented、零 warning。`--document-private-items` 不能省，因為 `service-backup` 的 module 全是私有的，不加的話 rustdoc **根本不會讀**那些 `//!` 裡的連結 —— gate 會因為沒看而綠（地雷 20 同源）。分段量測的教訓與 mutation gate 同一條：一條只覆蓋宣告範圍子集的 gate，對剩下那一部分等於沒跑過。 |
| 29 | **`references` 列只看得見被刪的那一個 project** | `row_references` 的掃描範圍是 `roots = sorted({os.path.dirname(p.rstrip("/")) for p in prefixes})`（`scripts/td-retire-gate.py:419` 起），所以一輪只走那個 project 自己的 source root。L3 從 `libs/service-k8s/src/service.rs` 拔掉 14 條 `@spec` / `SPEC-MANAGED` 之後，`defer`／`keep`／`lumen`／`relay`／`tape` 五個 app 的 `k8s/operator/crd.yaml` 各自留下指向 `libs/service-k8s/tech-design/semantic/source/libs-service-k8s-src-service-rs.md` 的 pointer（共 **7 條**），而那一輪 `references` 報 `dangling=0`。 | 那個綠在 scope 內是真的，但它不是「repo 沒有斷指標」。L3 的 commit message 因此把宣告寫成 across the four projects 而不是全 repo。lumen 那一條在 L4b 隨 CRD 重算修掉；另外四個 app 的 **6 條**在 L4c（`8f662e398b`）一併重算清掉 —— USER DECISION 2026-08-20 選了「現在修」而不是「開單延後」。重算前先確認 `libs/service-k8s/src/service.rs` 已無指向已刪樹的行（唯一存活的 `:37` `@spec` 指向 `apps/pgpool/tech-design/`，那棵樹不在本戰役的 15 個 project 內、檔案仍在，而且它掛在 `ReconcilePlan` 上、不是 schemars 型別，不會被折進任何 CRD）。 |
| 30 | **`references` 是字面前綴比對，相對路徑與 `aw.toml`／`llms.txt` 全部漏掉** | 判定式是 `if any(pref in line for pref in prefixes)`，prefix 是 `apps/lumen/tech-design` 這種絕對前綴，所以 `include_str!("../tech-design/validate/…")` 一次都不會命中；而掃描面 `SOURCE_DIRS`／`SOURCE_FILES`（`:370-371`）只有 `src e2e tests benches examples k8s` ＋ `Cargo.toml`，`aw.toml` 與 `llms.txt` 不在裡面。L4a 那一輪 `references` 報 `dangling=15`；我另寫一份同時吃絕對與相對、並加掃那兩個檔的獨立掃描，量到的是 **128 行 / 6 個檔**。 | 用獨立掃描的答案，並在 L4a 的 commit message 裡指名第 6 列低報。L4b 從 `aw.toml` 刪掉的殘留有 **108 行**（111 條 `td_ref` 全數清空 ＋ 3 條 stanza）、從 `llms.txt` 刪掉 1 個整檔的舊 `CODEGEN` 區塊 —— 這 109 行第 6 列一行都沒看見。與地雷 2、14 同一個形狀：**一個掃描面比宣告面窄的 gate，對窄掉的那一塊等於沒跑過。** |
| 31 | **`build` 列不編 test target，`e2e/` 的 `include_str!` 斷掉照樣綠** | L4a 那一輪第 8 列 `build` exit 0 過關，同一棵樹的 `cargo test -p lumen --no-run` 卻編不過：`apps/lumen/e2e/capability_stateful_workload_linkage.rs:16` 的 `include_str!` 指向那一輪剛刪掉的 md。`cargo build` 從不編 `[[test]]` target，所以這種斷裂對第 8 列**結構性隱形**。抓到它的是 `counts` 的 `embed` 欄（probe 會把 `include_str!` 的參數解析成相對於引用檔的路徑），不是 `build`。 | 有 `e2e/` 樹的 project，`build` 列要換成 `cargo test --no-run`（或兩條都跑）。本輪走人工補：L4b 先 `cargo test -p lumen --no-run`，再跑完整 `cargo test -p lumen`。S5 的 ratchet 要把這條寫死，否則下一次刪樹會用同一個方式假綠 —— 這是地雷 10 的同族：**一條不編某個 target 的 gate，對那個 target 說不出話。** |
| 32 | **`reasons` 列在 S4 是結構性空包彈** | `probe.selects` 只選「同一行同時帶退場樹路徑**且**帶 `reason=`」的標頭行。實測：`apps/lumen` ＋ `libs` 底下 `.rs` 裡帶 `reason="` 的 **101 行全部**是 `HANDWRITE-BEGIN` / `<HANDWRITE` 標記，而那些行不含樹路徑 → 交集恆為 **0**。S4 每一輪都報 `deleted_reasons=0 covered=0/0`，四條拒絕路徑一次都沒執行；連帶 S2 寫的 76 列 ledger 在 S4 沒有 consumer，§4 寫的 target `76/76 covered` 不可達。 | **USER DECISION 2026-08-20：不管。** 原話「之後不用管 SPEC-MANAGED HANDWRITE-BEGIN 之類的 畢竟 EC/TD 都退場了」。那些 marker 的 producer（`aw` CLI）已經整個刪掉，為一個沒有東西可保護的 row 補鑑別力，是在維護退役機制。ledger 留在樹上當 S2 的判定紀錄，不當 gate 輸入；地雷 19 一併以此收束。 |

---

## 4. Gate

### S4 每一輪（一個 project 一輪）

```bash
python3 scripts/td-retire-gate.py --whole-tree \
    --prefix <project>/tech-design --prefix <project>/external-contracts \
    --base <sha> --package <crate> [--features "<feature set>"]
```

七個 row，全部已實作並在真資料上跑過：

| # | row | 判什麼 | 紅的條件 |
|---|---|---|---|
| 1 | `counts` | 樹與指向它的標頭都不在了 | 該模式的 `required` 欄有任何一欄不是 0（whole-tree 13 欄；markdown 模式 4 欄 `md/lock/hdr/files`） |
| 2 | `residue` | whole-tree：prefix 底下什麼都不剩；markdown 模式：只剩 `.py` | 還有檔案 |
| 3 | `additions` | 這一輪一行都不加 | diff 有任何 `+` 行 |
| 4 | `deletions` | tree 外被刪的每一行都是規則選中的標頭 | 刪到別的東西 |
| 5 | `knowledge` | 這一輪失去標頭的每個 `src/**/*.rs` 都有非空 `//!` | 有檔案沒有 |
| 6 | `references` | project 自己的 source root 裡不再有任何一條指向退場樹（whole-tree 限定） | 還找得到 |
| 7 | `build` | 具名 crate 仍編得過（可帶 `--features`） | exit 非 0 |

**兩個模式互相退件。** 預設是 markdown-only（#3694 的 42 棵樹，`.py` 留在原地）；
`--whole-tree` 是本戰役的 15 棵（D1）。`--expect` 是選用的精確釘點，不是必填 ——
它曾經是必填而且**不可滿足**（§3 地雷 11）。

`additions` / `deletions` / `knowledge` 讀的是對 `--base` 的**整份 diff**，
不是只讀 prefix 底下。所以一輪必須從乾淨樹量（§3 地雷 12）。

### 第 8 列 `reasons`（地雷 23 的 consumer）

S4 每刪一條帶 `reason=` 的標頭，那條 `reason` 就消失。前七列沒有一列會紅。

所以加第八列，判準機械：**這一輪要刪的標頭裡，每一條 `reason=` 長度 ≥15 字的，
都必須在 ledger（`apps/lumen/docs/td-ec-reason-ledger.tsv`）裡有一列**，值是
`merged`（附 `//!` 的行號區間）、`disposable`（附理由）、或 `deferred:S3`（只有地雷 25 那 14 個檔可以用，且該列額外斷言那個檔**現在**的 `//!` 行數是 0 ——所以「已經補好了卻還掛著 deferred」會紅，而不是靜靜留著）。ledger 缺一條就 exit 非零。

| 列 | 量什麼 | current → target | 為什麼不會意外綠 |
|---|---|---|---|
| `reasons` | 本輪被刪掉的 `reason=` ≥15 字者，在 ledger 中的覆蓋率 | `0/76 covered` → `76/76 covered`（ledger 現況 **76 列全有值**：`disposable` 51 ＋ `merged` 11 ＋ `deferred:S3` 14，後者就是地雷 25 那 14 個檔，S3 補完要一起翻成 `merged`） | ledger 是人寫的，`merged` 那列還要對得上真的 `//!` 行號；把 ledger 刪掉會讓覆蓋率歸零而不是變成無事可查 |

已實作：`scripts/td-retire-gate.py` 的 `row_reasons()`，`--whole-tree` 那一輪自動掛上，
ledger 路徑可用 `--ledger` 覆蓋。它有四條拒絕路徑：本輪刪掉的 `reason=` 在 ledger 裡
沒有列（`covered`）、verdict 不在封閉集合裡（`bad_verdict`）、`disposable`／`merged`
宣稱的證據散文已經不在了（`bad_evidence`）、以及 `deferred:S3` 的檔**現在有** `//!`
（`stale_deferred`）。

證據的錨是**散文的 sha 而不是行號**（ledger 多一欄 `evidence_sha`）。理由是機械的：
S4 在同一批檔案裡刪標頭，每一個位於被刪標頭下方的 `//!` 區間都會往上位移，所以純行號
的錨會在**這一輪**把整份 ledger 判紅，而那和知識有沒有留下無關。行號降級成提示，
sha 仍然擋得住「證據被改寫」「證據被刪掉」「證據根本不是註解」三種。

### 為什麼「整棵刪掉」不是無鑑別力的綠

單看 `counts` 與 `residue`，「`rm -rf` 整棵樹」必然全綠 —— 那是 §3 地雷 3 引用的
那句註解在防的事。撐住鑑別力的是另外三個 row，它們對「刪過頭」與「刪不夠」都會紅：

- `deletions` — tree 外被刪掉的每一行都必須是規則選中的標頭。手滑刪掉一行真程式
  碼 → 紅。
- `build` — 具名 crate 必須仍編得過。§3 地雷 1 那種編譯期相依 → 紅。
- `knowledge` — 失去標頭的檔沒有 `//!` → 紅。「把指標和它指的知識一起刪掉」→ 紅。
- `references` — project 自己的 source root 裡還留著一條指向已刪樹的參照 → 紅。這是唯一看得見**非標頭**斷指標的一列（`crd.yaml` 那 12 條就是這一類）。

**S1 / S2 與 S4 必須是不同的 commit。** S1/S2 只加行不刪行，S4 只刪行不加行。
混在一輪會讓 `additions` 與 `deletions` 兩個 row 同時失去鑑別力。

### 負控制帳（一列一個 row，沒付過的要標出來）

一個 row 第一次要求就是綠的，那個綠可能是「乾淨」也可能是「打不到」。以下每一條
突變都在演練取 `git stash create` base **之前**種下，所以它屬於基線、那一輪自己的
diff 不變 —— base 之後才種會變成一條新增行，`additions` 會跟著紅，控制組就同時指向
兩個 row 而不是一個。

每一條突變還要宣告**它該讓哪一個欄位動**，並核對真的動了那一個。`reasons` M4 第一版
是改 ledger 的 `file` 欄，row 確實紅了 —— 但紅在 `covered`，因為 `file` 是 key 的一半，
改掉它等於讓那一列對不上任何被刪的 reason，`stale_deferred` 那條分支**一次都沒執行**。
只看紅綠會把它記成已付。所以 M4 改成動原始碼。

| row | 突變 | 觀測 | 還原 |
|---|---|---|---|
| `knowledge` | 刪掉 `libs/transport-h2c/src/error.rs` 唯一的 `//!` 行 | row 5 `FAIL … without_doc=1 e.g. libs/transport-h2c/src/error.rs`，rows 1–4、6 仍 PASS | sha256 `5896faef780dde0987ded3e256765e2d5a17c6613e5c8000d557adf92d863c9f` 前後相同 |
| `references` | 在同一個檔頭插一行**非標頭**註解 `// see libs/transport-h2c/tech-design/semantic/h2c-manager.md …`（刻意不含 `SPEC-REF:`／`@spec`，所以只有這一列看得見） | row 6 `FAIL … dangling=1 e.g. libs/transport-h2c/src/error.rs:1`，rows 1–5、7 仍 PASS | 同一個 sha256 `5896faef…3c9f`，逐 byte 相同 |
| `reasons` M1 未判定 | 從 ledger 拿掉一列 `merged` | `FAIL … covered=75/76 … e.g. unadjudicated libs/cli-std/src/issue.rs` | ledger 是 scratchpad 的副本，正本沒動 |
| `reasons` M2 verdict 越界 | 把一列 `merged` 改成 `documented-elsewhere` | `FAIL … bad_verdict=1` | 同上 |
| `reasons` M3 證據不是註解 | 把 `merged` 的 evidence 指到該檔第一行 `use`／`fn` | `FAIL … bad_evidence=1 … no run matching 71275bcf9ee1` | 同上 |
| `reasons` M4 已補好卻還掛 deferred | **改原始碼**：給 `apps/lumen/tests/capability_shared_ownership.rs` 加一行 `//!` | `FAIL … stale_deferred=1 e.g. deferred-but-now-documented apps/lumen/tests/capability_shared_ownership.rs` | sha256 `fee1a312a0d3` 還原後逐位元組相同 |
| `reasons` M5 證據散文被改寫 | 保留行號，只把 `evidence_sha` 換掉 | `FAIL … bad_evidence=1 … no run matching 1b2454077412 near libs/cli-std/src/issue.rs:3-30://!` | ledger 副本 |
| `reasons` 位移對照（必須**綠**） | 在 `libs/cli-std/src/issue.rs` 證據上方插一行 | `PASS … covered=76/76`，證明 sha 錨受得住 S4 的行號位移 | sha256 `60ef09289484` 還原後逐位元組相同 |
| `counts` | **未付**。要種的是「只留一份 `.md` 不刪」，預期 `counts` 紅而 `residue` 也紅。 | — | — |
| `deletions` | **未付**。要種的是「多刪一行真程式碼」。 | — | — |
| `build` | 由 §3 地雷 1 天然承擔（`include_str!` 那條相依刪了就編不過），但尚未實測。 | — | — |

### L4 前置盤點：header 規則打不到的 16 條

`apps/lumen` 的 source root 一共 702 條指向退場樹的參照，其中 686 條是標頭形狀、
`hdr` 這一欄看得見；剩下 **16 條**要逐條處置，否則 `references` 在 L4 必紅：

| 位置 | 條數 | 處置 |
|---|---|---|
| `k8s/operator/crd.yaml` | 12 | 用 renderer 重新產生，放在與刪除**不同**的 commit（§3 地雷 15） |
| `tests/retired_credential_surface.rs:88,134` | 2 | 字串字面值，隨 S3 搬檔一起清 |
| `tests/llm_command_template_flags_are_live.rs:5` | 1 | `//!` 行；#3708 已指定改指 `apps/lumen/src/dx-contract.yaml` |
| `tests/ec_claim_closure_consistency.rs:9` | 1 | `const CLAIM_DOCUMENT`，它驗的主體隨樹退場 → 整條退場 |

---

## 5. 順序 — bottom-up

一個 project 不會依賴尚未收斂的 project。同層之間可平行（寫入所有權互不重疊），
跨層不可。

| 層 | Project | 為什麼在這層 |
|---|---|---|
| **L0** | `build-stamp`、`metrics-prometheus`、`peer-tls` | md=0、標頭=0 —— 純 Python 樹，最乾淨的第一刀 |
| **L1** | `raft-core`、`service-auth`、`service-observability`、`storage-durable` | 同樣 md=0、標頭=0 |
| **L2** | `transport-h2c`、`cli-std`、`openapi-codegen` | 開始有 md 與標頭（33 / 78 / 92 條） |
| **L3** | `raft-runtime`、`service-backup`、`service-http`、`service-k8s` | 有 md、標頭，且被 lumen 直接組合 |
| **L4** | `apps/lumen` | 591 條標頭、102 個 md、地雷 1/4/5/6 全在這裡 |

L0/L1 共 7 個 project 是純 Python 樹、零標頭，所以它們同時是 **S0 儀器的對照組**：
擴充後的 gate 如果對 `build-stamp` 都判不出「py 從 19 變 0」，儀器就是壞的。

---

## 6. 進度

| 階段 | 狀態 | 證據 |
|---|---|---|
| S0 擴充儀器 | **完成** | `libs/transport-h2c` 一輪 whole-tree 演練七列全綠：`counts` 13 欄歸零、`residue` 0、`additions` 0、`deletions` 0（9 個檔 33 條標頭）、`knowledge` `lost_pointers=8 owed_doc=6 without_doc=0 exempt_non_src=2`、`references` `scanned_files=10 dangling=0`、`build` `exit=0`。`knowledge` 與 `references` 兩列都已付負控制（見 §4）。 |
| S1 知識地板（14 檔） | **完成** | 14 個檔全部補上 `//!`，共 316 行；`src/**/*.rs` 缺 `//!` 的數量 14 → **0**（15 個 in-scope root 全掃）。四個 crate build exit 0，且 `RUSTDOCFLAGS="-D warnings" cargo doc --no-deps` 綠（`service-backup` 另加 `--document-private-items`，因為它的 module 全是私有的，不加就不會檢查那些 `//!` 裡的連結）。**這個綠是分段的、不是全樹的** —— 它沒有涵蓋 `storage-durable` 與 `cli-std`，而 `libs/storage-durable/src/snapshot_store.rs:19` 的一條 `redundant-explicit-links` 就是從這裡漏過去的，見地雷 28。無 code fence、`lumen-bench` 塊零方括號。 |
| S2 知識差集（79 份） | **判定完成，併入 12 檔** | 79 份 md 的可判定面逐條判完（見下三個小節）；76 條 `reason=` 全部落 ledger（判定當下 `disposable` 51 ／ `merged` 11 ／ `deferred:S3` 14；S3 把其中 8 條的擁有者搬進 `e2e/` 之後重蓋成 `disposable` 51 ／ `merged` 19 ／ `deferred:S3` 6，`apps/lumen/docs/td-ec-reason-ledger.tsv`）。併入的 `//!`：`tokenize.rs`、`cli-std/{lib,issue}.rs`、`service-k8s/render.rs`、`render/deployment.rs`（1 → 17 行）、`service-observability/jsonl.rs`（1 → 30 行）。`RUSTDOCFLAGS="-D warnings" cargo doc -p service-k8s -p service-observability --no-deps --document-private-items` exit 0（含地雷 27 的四處外層 doc 拆除與 `issuer.rs` 一條真的壞連結）；`cargo test -p service-k8s -p service-observability` exit 0，205 passed / 0 failed。第 8 列 `reasons` 已實作並付清負控制（M1–M5 全紅 ＋ 位移對照綠）。剩：`interfaces/dx` 的 TLS 段與 `logic/prove-traceparent-*` 兩段（後者轉 S3）。 |
| S3 `tests/` → `e2e/`（161 檔） | **完成** | 176 個 rename 全部是 `apps/lumen/tests/` → `apps/lumen/e2e/`：161 個 `.rs` ＋ 15 個它們讀的附帶檔（`ec/pg/schema.sql`、`perf-baseline.json`、一個 proptest-regressions、9 個 rig `.toml`、2 個 rig `.py`）。`apps/lumen/Cargo.toml` 加上 `autotests = false` 與 167 條 `[[test]]`（161 搬進來 ＋ 6 條原本就在 `e2e/`），檔案 ↔ 宣告雙向零差異、`name` 全等於檔名、依 name 排序。11 個 feature-gated 檔**不需要** `required-features`：它們用檔頭 `#![cfg(...)]` inner attribute，feature 關掉時編成空 crate——這正是原本那 6 條所依賴的。`--no-run` census：170 個 Executable，其中 167 個路徑在 `e2e/` 底下（另外 3 個是 lib 與兩個 bin 的 unittests），0 行 error，cargo exit 0。`include_str!` 深度不變（`tests/` 與 `e2e/` 同為 `apps/lumen/` 的直接子目錄），唯二的同目錄 include（`perf-baseline.json`、`rig_stateful_adapter.rs`）跟著讀它們的檔一起搬。S2 轉過來的 8 條知識落在擁有它的 `.rs` 的 `//!`，共 90 行。兩條負控制見下。 |
| S4 刪樹（30 棵 / 15 project） | **完成** | 六顆 commit，bottom-up：`3cafed580d` L0（116 檔 / −9,739）、`5109627194` L1（184 / −22,419）、`e81ffb94bd` L2（262 / −42,081）、`1ba60e9a89` L3（383 / −54,378）、`4a30ca3097` L4a（492 / −95,981）—— 刪除段共 **1,437 檔 / −224,598 行 / +0 行** —— 再加 `5d65a659e5` L4b（lumen 的 consumer 修補，見下）。L0–L3 每一輪八列全綠；L4a 六綠兩紅，兩條紅的成因逐條寫在它自己的 commit message 裡（地雷 30、31）。`knowledge` 列到 L2 才第一次有牙：L2 **40** 個 `src/**.rs` 掉了標頭、37 個欠 module doc、**0 個沒有**；L3 **34** 個掉標頭、34 個都已經有 `//!`。累計 **74** 條指標消失、**0** 行 `//!` 需要現場補 —— S1 的地板把 S4 移走的東西整個接住了，這是這場戰役唯一一個事前預測、事後兌現的數字。`reasons` 列每一輪都是 `deleted_reasons=0 covered=0/0`（地雷 32）。 |
| S4 / L4b lumen 的 consumer 修補 | **完成** | 與刪除分開的一顆 commit，因為它含新增行（地雷 15）。**十個檔、+44 / −824**：`e2e/ec_claim_closure_consistency.rs`（−375，整個測試的主體就是被刪掉的 claim-closure 樹，subject-loss）與它的 3 個 `#[ignore]` EC wrapper 一起刪，`Cargo.toml` 少 4 條 `[[test]]`（167 → 163）；`e2e/capability_stateful_workload_linkage.rs` 拆掉 `primary_td_linkage_is_bound`（77 → 59 行），`//!` 記下「讀的那份文件 S4 刪了，沒有可改綁的存活物」；`e2e/retired_credential_surface.rs` 拔掉兩筆已經指不到東西的 `Allowance`（該檔 `:418-441` 自己要求這樣做，是收緊不是放寬）；`aw.toml` 清 111 條 `td_ref` ＋ 3 條 stanza（TOML 重新 parse 過）；`llms.txt` 整份重寫（32 → 33 行，舊的是已刪 `aw` CLI 的 `CODEGEN` 區塊，指著四個不存在的指令）；`k8s/operator/crd.yaml` 用它自己的 renderer 重算，diff 恰好 **13 刪 / 9 增** —— 與 gate docstring 事先寫下的預測逐條相符（4 條整條 `description:` 消失）。事後獨立掃描（絕對＋相對路徑，加掃 `aw.toml`／`llms.txt`／`README.md`／`CONTRIBUTING.md`）：`apps/lumen` 殘留 **0 檔 / 0 行**。 |
| S4 / L4c 四個 app 的 CRD 重算 | **完成** | `8f662e398b`。地雷 29 指名的跨 project 殘留：`apps/{defer,keep,relay,tape}/k8s/operator/crd.yaml` 共 **6 行**，全部指向同一個被 L3 刪掉的檔 `libs/service-k8s/tech-design/semantic/source/libs-service-k8s-src-service-rs.md`。與 lumen 同一個做法 —— 用各 app 自己的 renderer 重算，不逐行改（地雷 15）：`cargo run -q -p <app> --features operator --bin <app> -- k8s crd render --out apps/<app>/k8s/operator/crd.yaml`，四支 exit 0。重算後全 repo 掃描：`apps/*/k8s/operator/crd.yaml` 指向退場樹的行 **6 → 0**。 |
| S4 / L4d 殘留清乾淨 | **完成** | `6c0c64f3b0`，**16 檔 / +215 / −457**。全 repo（不是 gate 第 6 列那種 per-project）掃 15 棵退場樹：**31 檔 / 40 行**。其中 22 行是戰役自己的儀器與本文件（必須指名它退場了什麼）、1 行在 pgpool 自己的活樹裡（不在 15 個範圍內、指得到），剩下 **16 檔 / 18 行**才是指著空氣的活物件。清法：13 份 `libs/<crate>/llms.txt` 整份重寫、root `aw.toml` 拔掉 lumen stanza 的 `td_path`、刪掉兩支死腳本。`llms.txt` 舊的是已刪 `aw` CLI 的 `CODEGEN-BEGIN/END` 區塊，開頭一律寫 “TD-first map for agents. Start from tech design”，連結不存在的 `tech-design/`，並指名四個不存在的指令 —— 掃描抓到的那一條路徑是它最小的問題。重寫時每個 crate 的 Behaviour 段是從它自己的 `Cargo.toml` 量出來的：7 個有 `[[test]]` 的寫實際條數（openapi-codegen 1、transport-h2c 2、peer-tls 4、service-k8s 6、service-http 8、raft-core 11、raft-runtime 22），6 個沒有 `e2e/` 的直說沒有，而不是連一個不存在的目錄。`libs/service-observability` 根本沒有 `llms.txt`，所以是 13 份不是 14 份。`aw.toml` 24 條 `td_path` 只有 lumen 這條指不到東西，其餘 23 條的樹都還在；`plugins/` 與 `scripts/` 底下沒有任何東西讀 `td_path`。事後同一支掃描：**15 檔 / 22 行**，全部歸得了戶（儀器 12、本文件 7、pgpool 1、`acceptance/gcp/scripts/verify-lumen-auth.sh:726` 的 `SPEC-MANAGED:` 標記 1，最後這條依 2026-08-20 的 USER DECISION 不管）。活物件殘留 **0**。`e2e/cli_credential_paths_retired.rs` 會走 `apps/lumen/scripts/`，直接跑 5 passed / 0 failed；它沒有排除清單，所以刪檔只會讓它掃的集合變小。 |
| S5 規則 ＋ ratchet | 未開始 | 已知進場條件：地雷 29／30／31 各是一條 gate 的鑑別力缺口，ratchet 要一次補齊（跨 project 掃描、相對路徑、`cargo test --no-run`）。地雷 32 依 USER DECISION 不補。 |

### S2 量測：可判定面比「79 份文件」小得多

先量再判。把每一份 md 的**圍籬外**內容取出，扣掉五種零知識形狀 ——
H1（檔名的重述）、`# Reviews` / `### Review N` / `**Verdict:**`、reviewer 自己的
`- [logic]` / `- [unit-test]` 條目、以及 `<!-- type: logic lang: mermaid -->`
這種機器標記 —— 剩下的才可能帶著 code 沒有的事實。

| 類別 | 份數 | 依據 |
|---|---|---|
| 目錄鏡像（generated inventory） | **27** | front-matter 是 `Semantic coverage for "<dir>"` 或 `Lossless source-unit coverage`，內容是 `path:` 鍵的清單。與 `semantic/source/` 同一類，只是粗一級。 |
| 圍籬外只剩樣板 | **38** | 扣掉五種形狀後一行不剩 |
| 圍籬外真的有散文段 | **3** | 見下表 |

所以 S2 的判定面是 **3 個散文段 ＋ 68 份 front-matter `summary:` ＋ 76 條 `reason=`**，
不是 79 份文件。最後那一項不在 md 裡 —— 它在 `.rs` 的標頭上，是判定 J 的時候才浮出來的
（地雷 23）。
`## Logic` 的 mermaid 是演算法流程圖（現在 `.rs` 本身就是），`## Unit Test` 的
requirementDiagram 是測試理由（歸 S3 的 `e2e/`），`## Changes` 的 yaml 是「哪個檔
擁有它」的索引 —— 這三種都是**投影**，不是來源。

三個散文段：

| 散文段 | 落點 | 判定 |
|---|---|---|
| `logic/cjk-bigram-fallback-*.md` 的 `Contract (approved, final)` | `apps/lumen/src/tokenize.rs` | **併入（並改正）** |
| `interfaces/dx/lumen-dx-contract.md` 的 TLS Secret 所有權段 | `apps/lumen/src/dx.rs` | 待判 |
| `logic/prove-traceparent-*.md` 的兩段 | test 側 | 待判（測試理由，歸 S3） |

### S2 shortlist：front-matter `summary:` 裡真的帶事實的

判準是可否證的，不是品味：**md 說了一件事，而它 (a) 讀那個 `.rs` 推不出來、
(b) 也沒寫在任何測試裡**。合格的只有三種 —— 被**否決的替代方案**、
**跨檔／維運的注意事項**、以及某個常數或保守上限的**為什麼**。其餘（做了什麼、
改了哪些檔、哪一號 WI）都是 code 與 git 已經記著的。

| # | md | 事實 | 落點 |
|---|---|---|---|
| A | `interfaces/cli/cli-connect-query-k8s-agent-workflow.md` | #2873 把 credential 那半（旗標＋環境變數＋Secret 查詢）整個移除，所以 wrapped command 只拿到 `LUMEN_URL`、`lumen query` 不送 `Authorization`；身分改用 `TokenRequest` 鑄的 audience-bound SA token，只留在 CLI 自己的記憶體裡（#2878），不進子行程環境 | `libs/cli-std/src/connect.rs`、`apps/lumen/src/bin/lumen.rs` |
| B | `logic/expose-ssd-as-a-simple-toggle-*.md` | **per-cloud-provider 的 StorageClass 對照表被明確否決**（維護與正確性負債）；沒有 provider 偵測邏輯 | `apps/lumen/src/operator/crd.rs` 的 module `//!` |
| C | `logic/raftstorage-pvc-has-no-auto-expansion-*.md` | 改 `spec.serving.raftStorage` **本身不會** resize 既有的 per-pod PVC —— StatefulSet 的 `volumeClaimTemplates` 建立後不可變，`replicasPerShard` 取任何值都一樣（#812 之後） | `apps/lumen/src/operator/resize.rs` |
| D | `logic/render-serving-as-a-statefulset-unconditionally-*.md` | 無條件 StatefulSet＋PVC，`replicasPerShard:1` 也一樣；**「按 replica 數換 workload kind」是被換掉的舊行為** | `apps/lumen/src/operator/render.rs` |
| E | `logic/allow-has-child-to-combine-with-sort-*.md` | `has_child` 無法驅動 per-doc keyset planner，所以帶 `has_child` 的 sorted query 走 materialized sort 路徑 | `apps/lumen/src/storage.rs` |
| F | `logic/raise-multi-key-sort-cap-beyond-2-keys.md` | 上限 2→4 的理由：plan 與 cursor 早就帶完整 `Vec<SortValue>` 並逐鍵比較，**2 是保守值不是限制** | `apps/lumen/src/types.rs` |
| G | `logic/offset-cursor-sort-silently-ignores-sort-*.md` | 那個 400 存在的理由：先前 `offset` cursor 配 `sort` 是**靜默忽略** sort 退回 score 排序 | `apps/lumen/src/api.rs` |
| H | `logic/sort-missing-value-handling-*.md` | `exclude`（預設）留在快的 keyset planner；任一鍵是 `first`/`last` 就換 materialized 路徑並把缺值列計入精確 total | `apps/lumen/src/types.rs` |
| I | `interfaces/dx/lumen-dx-contract.md` | serving 與 peer TLS Secret 由部署方／外部平台提供，operator 只消費，**不擁有 issuer 或憑證生命週期** | `apps/lumen/src/dx.rs` |
| J | `interfaces/cli/courier-proxy-mode-client-*.md` | 沒設 courier URL 時走原本的 direct-GitHub 路徑，**byte-identical fallback** | `libs/cli-std/src/issue.rs` |
| K | `interfaces/rest/relay-wal.md` | 顯式 broker-log WAL 模式已退場，改成 raft-runtime 的 primary/replica durability；`wal_relay` 產物已刪，不得復原 | 待定（`apps/lumen/src/lib.rs` 或 WAL 選模處） |

**每一條都要先對照現在的 code 才准寫進 `//!`。** md 的說法可能已經過期，把過期的
說法抄進 source 就是反向的 tokenize.rs 事故。

### S2 判定：A–K 逐條核對（已做完）

11 條逐一打開 code 對照過。**10 條已經在 source 裡，其中好幾條是逐字，
有兩條比 md 寫得更完整。** 只有 J 是真缺口。

| # | 判定 | 已由誰承載 | 對照後的觀察 |
|---|---|---|---|
| A | 衍生可棄 | `apps/lumen/tests/cli_credential_paths_retired.rs:2-30`；`apps/lumen/src/bin/lumen.rs:564-588`、`:623-640`、`:1474-1476` | 測試檔的 `//!` 比 md **寫得更好** —— 它列出三條被移除的路徑、寫明「故意斷言兩次」、並劃出 `spec_cli.rs`／`operator_render.rs` 兩個姊妹檔的界線。**S3 搬這個檔時 `//!` 必須整段帶走。** |
| B | 衍生可棄 | `apps/lumen/src/spec.rs:1397-1408`；`apps/lumen/src/operator/crd.rs:558-566` | `spec.rs` 有一整段 `### Non-goals: no serving.ssd toggle, no provider-detection`，比 md 的 summary 完整。crd.rs 的欄位 doc 另外寫明那個 storageClass 欄位是「informational reference only, not a value validated or defaulted by this field」。 |
| C | 衍生可棄 | `apps/lumen/src/operator/resize.rs:3-12` | 逐字：`volumeClaimTemplates` 建立後不可變，所以改 `spec.serving.raftStorage` 之後 operator 對那個欄位的 `apply` 是 silent no-op。 |
| D | 衍生可棄 | `apps/lumen/src/operator/crd.rs` 的 `//!`；`apps/lumen/src/operator/render.rs` 的 `//!` | 兩邊都寫了，且都把 `replicasPerShard` 與 durability 的關係講清楚（「`replicasPerShard` only gates raft consensus, never persistence」）。 |
| E | 衍生可棄 | `apps/lumen/src/types.rs:343-355` | `SearchRequest.sort` 的 `///` 同時承載 E、G、H 三條。**md 的落點欄寫的是 `storage.rs`，錯的。** |
| F | 衍生可棄 | `apps/lumen/src/storage.rs:57-62` | `MAX_SORT_KEYS` 的 doc 逐字寫著「a guard against pathological requests, not a structural limit」。**md 的落點欄寫 `types.rs`，錯的。** |
| G | 衍生可棄 | `apps/lumen/src/storage.rs:3908-3913`；`types.rs:343-355` | 逐字，連「would silently fall through to the score-ranked path and IGNORE `sort`」都在。**md 的落點欄寫 `api.rs`，錯的。** |
| H | 衍生可棄 | `apps/lumen/src/types.rs:343-355` | 同 E。 |
| I | 衍生可棄 | `apps/lumen/src/spec.rs:222`；`operator/crd.rs:76-114` | 逐字。**md 的落點欄寫 `dx.rs`，錯的。** |
| J | **真缺口 → 走 ledger** | 無 `//!` 承載 | `libs/cli-std/src/issue.rs:1-17` 的 `//!` **完全沒提 courier**。courier 路由與 byte-identical fallback 只寫在 `:214`、`:320`、`:393`、`:494`、`:564`、`:747`、`:883` 的 `reason=` 屬性和 `:216-228` 的 item doc 裡 —— 也就是 S4 要刪的那一面。這一條不在這裡解決，歸地雷 23 的 ledger。 |
| K | 衍生可棄 | 無殘留 | `apps/lumen/src/` 裡沒有 `wal_relay` 殘留可講。「已退場、不得復原」的 consumer 是「那個符號不存在」，不是一段散文。 |

**這張表推翻了一個工作假設。** 我原本預期 79 份 md 裡有一批只存在於文件的設計事實；
實測是走過 aw lifecycle、帶 per-change TD 的 change，**當年就把契約散文抄進 code
comment 了**。`tokenize.rs` 是唯一的例外，而它剛好是 md 自己的 `## Changes` 有要求
更新 doc、那件事卻沒做的那一份。

**同時它給了 D1 一個新的理由。** E、F、G、I 四條的 md「落點」欄位指向錯的 `.rs`——
事實有被承載，但不是 md 說的那個檔。一份對自己主題的索引都已經過期的文件，
留著只會誤導下一個讀它的人。

判為可棄的其餘 md，理由歸這三類：目錄鏡像（27）、把「做了什麼／改了哪些檔／
哪一號 WI」重述一遍（code 與 git 已經記著）、以及 `summary: (fill)` 從來沒填。

### S2 判定：76 條 `reason=` 的機械分群

判定不靠讀 76 次的印象，靠一個機械性質：**標頭底下三行內有沒有 `//!`**。

| 群 | 條數 | 為什麼這個性質能預測判定 |
|---|---|---|
| head（底下就是 `//!`） | **49** | 同一個作者在同一個 commit 寫了兩份：`reason=` 是給機器看的一行摘要，`//!` 是給人看的散文。所以 `//!` 幾乎一定更長更完整。 |
| deep（底下沒有 `//!`） | **27** | 沒有鄰居可以承接。無主知識只會在這裡。 |

head 群另外用詞彙覆蓋率排序後**逐條讀了 23 條**，涵蓋 11 個 project 中的 8 個，
並且**把覆蓋率最低的 9 條全部讀完**（含 3 個 0.00 的）。結果一致：`//!` 每一次都說得更多。
最能說明的是 `libs/service-auth/src/k8s/loopback_proxy.rs` —— `reason=` 說
「contract 是 credential 不在哪裡」，`//!` 則畫了 ASCII 流程圖、寫明環境變數會被
`/proc/<pid>/environ` 讀到與被 crash reporter 抄走、並說明為什麼 refresh 失敗必須是 `503`
而不是繼續轉發手上那顆 token。判定：**46 條 `disposable`，證據是它們自己下面那段 `//!`**。

剩下 3 條是這個機械性質**預測失敗**的地方，而失敗有規律：`//!` 只有一行、是個**標題**，比它本來要涵蓋的那條 ≥15 字 `reason=` 還短。三條逐一重判：

| 檔 | 一行 `//!` | 重判 |
|---|---|---|
| `libs/raft-runtime/src/peer_transport.rs:1` | `//! Peer transport …` | **還是可棄**。generation 與 last-known-good 的語意由同檔 `///` `:26-28`、`:52-53` 承載，peer-tls 的 `reload.rs` module doc 又把 build-then-swap 與「It does not fail open」寫得更完整。 |
| `libs/service-k8s/src/render/deployment.rs:1` | `//! Stateless Deployment workload rendering.` | **已併入**（1 → 17 行）。`reason=` 的真內容是一條**否定契約**（emitting no stateful or sticky-session contract），而否定契約是程式碼結構上說不出口的 —— 它講的是**不存在**的欄位。併入的是缺席清單本身（`serviceName`／`volumeClaimTemplates`／`podManagementPolicy`／`SHARD_COUNT`／`REPLICAS_PER_SHARD`／`VOTER_COUNT`／`sessionAffinity`）、要那些欄位的 caller 該去用 StatefulSet helpers，以及**這條排除是有 consumer 擋著的**：同檔的 `deployment_has_no_stateful_or_sticky_session_contract` 與 `render/common.rs` 的 `ordinary_children_are_cluster_ip_and_non_sticky`。 |
| `libs/service-observability/src/jsonl.rs:1` | `//! Versioned collector-compatible structured stdout.` | **已併入**（1 → 30 行）。`reason=` 看起來只是清單，但它點到的每一件都在實作一條沒寫下來的政策：schema 常數是 collector 的鍵（所以改語意要換常數不是改常數）、超長 key/value 在 UTF-8 邊界**截斷**而 attribute 超過上限則**丟棄**且因為輸入是 `BTreeMap` 所以丟的是字典序在後面的（決定性，不是任意子集）、reserved key 是**丟棄不是改名**（caller 偽造不了 framing）、sensitive key 是**丟棄不是遮罩**（所以測試斷言的是缺席而不是遮罩字串，而 `baggage`／`tracestate` 被當成帶 credential 而不是 trace metadata）。 |

教訓寫進 ledger 產生器（`MIN_EVIDENCE_LINES = 2`）：證據段落**短於 2 行就不算證據** —— 一行 `//!` 是標題不是敘述，要往上找 item doc、再往下找 `///`，都沒有更長的就記 `homeless:unadjudicated` 讓它**紅**而不是記成已判定。ledger 已重跑：76 列，`disposable` 51 ／ `merged` 11 ／ `deferred:S3` 14，短證據 0 列、無主 0 列。三條重判各自落在：`peer_transport.rs:1 → :26-28:///`、`render/deployment.rs:1 → :2-18://!`、`jsonl.rs:1 → :2-31://!`。

deep 群 27 條的處置：

| 條數 | 檔 | 處置 |
|---|---|---|
| 8 | `libs/cli-std/src/{issue,lib}.rs` | **已併入**。整個 `libs/cli-std/src` 的 `//!` 裡 `courier` 出現 **0 次** —— courier proxy mode（#1320）從 module doc 看不見。併入三件 `///` 沒說的事：courier 模式存在本身、未設定時的 fallback 是**契約上逐位元組相同**（這才是 URL 建構被抽成純函式的原因）、以及本 crate 沒有 HTTP-mock dev-dependency 所以路由只能靠斷言 request shape 驗。`RUSTDOCFLAGS="-D warnings" cargo doc -p cli-std --no-deps --all-features --document-private-items` exit 0。 |
| 1 | `libs/service-k8s/src/render.rs:31` | **已併入**。子句「preserving the monolithic root compatibility surface … in this first landing」沒有任何地方承載：root 仍直接定義 `service_account`／`headless_service`／`pdb`／`cron_job`／`service_statefulset`／`sharded_statefulset`，而 `common`／`deployment` 是子模組。這個不對稱是刻意的（#1849），不是沒做完的重構 —— 不寫下來，下一個人會去「收尾」而順手打爛所有 adopter。 |
| 4 | `apps/lumen/src/operator/fleet.rs:418`、`libs/service-k8s/src/controller.rs:29`、`:247`、`libs/service-k8s/src/service.rs:232` | **衍生可棄**。緊接著的 `///` 每一條都寫得更完整：fleet 的 poll-loop 理由（人手編輯的 cluster-scoped 物件、30s 收斂、自己的 Lease）、leader gate 放在 `reconcile_entry` 的度量論證、`Condition` 手寫的 `JsonSchema` 理由、以及 clock-free fact/projection 切分（`service.rs:292-297`、`:442-443`）。 |
| 14 | 見地雷 25 | **`deferred:S3`**。這 14 個檔一行 `//!` 都沒有。 |

### S2 判定紀錄（一份散文一列）

| 散文 | 判定 | 落點 / 理由 |
|---|---|---|
| `logic/cjk-bigram-fallback-for-analyzer-jieba-when-jieba-feature-is-off.md`（#1975） | **併入** | 落 `apps/lumen/src/tokenize.rs` 的 `//!`，2 行換 15 行。**原本的 `//!` 是錯的**，不只是不完整：它說 jieba feature 關掉時 fallback 是 `whitespace_lower`，而 `tokenize.rs:94-139` 是 CJK-bigram tokenizer。md 自己的 `## Changes` 就要求更新這段 module doc，那件事當年沒做。搬進去的是 CJK run 規則、N-1 bigram（對齊 Lucene `CJKBigramFilter`）、單字元 unigram、`lumen 搜尋引擎` 保留 `lumen`、掃描順序，以及**只存在於 md 的 reindex 注意事項**。驗證：`cargo build -p lumen --lib` exit 0；`cargo test -p lumen --lib tokenize` 9 passed。 |
| shortlist A–K 的 10 條（A–I、K） | **衍生可棄** | 逐條打開 code 對照過，每一條都附了承載它的 `file:line`，見 §6「S2 判定：A–K 逐條核對」。不是抽樣，是 11 取 11。 |
| shortlist J（courier fallback） | **未決 → 轉 ledger** | `libs/cli-std/src/issue.rs` 的 `//!` 沒提 courier，事實在 `reason=` 屬性上，屬地雷 23。判定寫進 `apps/lumen/docs/td-ec-reason-ledger.tsv`，不寫在這裡。 |
| 27 份目錄鏡像（`semantic/*.md`，見上表） | **衍生可棄** | 量測而非推測：front-matter 是 `Semantic coverage for "<dir>"` 或 `Lossless source-unit coverage`，本體是 `path:` 鍵的清單，圍籬外只有 H1。`ls` 加 `Cargo.toml` 就是同一份清單，且是活的。 |

---

### S3 的兩條負控制（manifest ↔ 守衛）

`autotests = false` 之下，一個 `.rs` 沒有 `[[test]]` 就不會被編譯，也就不會被跑。
所以這一階段真正要證的不是「檔案搬到了」，而是「manifest 與磁碟不一致時會有東西喊」。

| # | 突變 | 預期 | 實測 |
|---|---|---|---|
| NC1 | 刪掉 `access_log_subject` 那一條 `[[test]]`（檔案留著） | 紅：census 少一顆，且宣告守衛點名該檔 | census 從 170 掉到 **169**；守衛 `all_e2e_test_files_are_declared_in_cargo_toml` 在 `:207` panic（3 passed／**1 failed**）並在輸出點名 `access_log_subject.rs`；還原後 sha256 `7cceeb22…836fa8` 逐位元組相同 |
| NC2 | 把 `api_e2e` 與 `perf_gate` 兩條的 `path` 對調，`name` 不動 | **綠** | 守衛 **4 passed／0 failed**，全綠；還原後 sha256 `7cceeb22…836fa8` 逐位元組相同 |

NC2 是故意設計成過不了的。`feature_gated_targets_are_registered` 的第四個測試
（`e2e/feature_gated_targets_are_registered.rs:200` 組字面值、`:207` 斷言）只比對
`path = "e2e/<basename>"` 這個字面值，從不讀 `name`，所以把兩條的 path 對調對它是隱形的——雖然這之後
`cargo test --test api_e2e` 會跑到 `perf_gate.rs`，而 `apps/lumen/aw.toml` 的
`test_path` 正是用檔案路徑指的。

**這一列量的是盲點，不是鑑別力。** 記在這裡是為了讓 S5 寫 ratchet 的時候知道要補
`name` ↔ `path` basename 的一致性斷言；現在那個一致性只由「我逐條檢查過」擔保，
沒有任何 gate 守著它。

### S3 交棒給 S4 的六件事

| # | 事實 | 為什麼 S4 需要它 |
|---|---|---|
| 1 | ~~S3 必須先 commit，S4 的 diff 才量得準~~ **已滿足** | gate 的 `additions`／`deletions` 是對 base 量整份 diff；base 不含 S3 的話，176 個 rename 會被算進 S4 的帳上。S3 已落在 `aa0da939b1` 與 `e005374a7f`，所以 S4 的 base 就是 §7 記的那顆 SHA，不需要再扣除 rename |
| 2 | `apps/lumen/Cargo.toml` 現在是測試清單本身 | S4 刪 `tech-design/`／`external-contracts/` 不該動到它；動到就是刪錯東西 |
| 3 | ledger 還有 6 條 `deferred:S3` 沒收 | 兩個不同的理由。5 條的擁有者在 `libs/**`，不在 S3 的範圍；第 6 條 `apps/lumen/e2e/ec_claim_closure_consistency.rs` 這一輪**有搬**，但它沒有知識要留——它的 `CLAIM_DOCUMENT`（`:9`）指向 `apps/lumen/external-contracts/claim-closure/production-claims.md`，主體隨樹退場，case 也跟著退場。六個檔都還是 0 行 `//!`，這是它們不觸發 gate `stale_deferred` 的原因 |
| 4 | `retired_credential_surface` 是既存紅 | 它點名 `apps/lumen/README.md:368`。S4 刪 `tech-design/` 會讓該檔 `:88` 與 `:134` 兩條 allowance 失效，要一起處理 |
| 5 | `e2e/feature_gated_targets_are_registered.rs` 的 11 條 REGISTRY 路徑已改成 `apps/lumen/e2e/…` | S4 不需要再碰它 |
| 6 | ~~`apps/lumen/e2e/ec_claim_closure_consistency.rs` 必須維持 0 行 `//!`~~ **已作廢** | L4b 把那個檔整個刪了（它的主體就是被刪掉的樹），所以既沒有 `//!` 要維持，也沒有 `reasons` 列會讀它 —— 地雷 32 說明那一列本來就沒在讀任何東西。ledger 的第 6 條 `deferred:S3` 隨之收束為「擁有者已刪除」。 |

---

## 7. 恢復程序（context 掉了怎麼接）

**S0–S3 落在哪。**「已完成」在 §6 是一句話，在樹上是四顆 commit，`app/lumen`
上連續：

| SHA | 主旨 | 帶了什麼 |
|---|---|---|
| `d27508f64c` | `td-ec(S0-S2)` | 儀器（`scripts/td-retire-*.py`）＋知識地板與差集：29 檔，1155 insertions／143 deletions |
| `aa0da939b1` | `lumen(S3): move tests/ to e2e/` | 176 個 `apps/lumen/{tests => e2e}/` rename、`autotests = false` ＋ 167 個 `[[test]]`、16 個 consumer 重新指向：192 檔，818／171 |
| `e005374a7f` | `lumen(S3): land the eight deferred knowledge runs and the reason ledger` | 8 條 `deferred:S3` 的 `//!` ＋ `docs/td-ec-reason-ledger.tsv`（76 列）＋本文件：10 檔，620 insertions |
| `ae40071a1e` | `docs(lumen)` | 地雷 28：分段 rustdoc gate 的覆蓋缺口 |

S4 的 diff base 是這四顆的最後一顆，不是 `main`。`apps/lumen/Cargo.toml`
在 `e005374a7f` 之後的 sha256 是
`7cceeb2285f4074bbe27987f6116c1645b3b6e23334edebb785a5528bb836fa8` —— S4 不該
動到它，動到了就是碰了 D2 的產物。

進 S4 之前，樹上有一條**既存的紅**：`apps/lumen/e2e/retired_credential_surface.rs:411`
指著 `apps/lumen/README.md:368`。它在 S3 之前就紅，不是 S3 造成的，也不要在 S4
把它當成新壞掉的東西去追。

1. 讀 §0（三個凍結決策）、§3（地雷）、§4（gate）。
2. 讀 §6 找第一個未完成的階段。**不要相信 §1 的表**，它是 2026-08-19 的快照。
3. 重量一次：`python3 scripts/td-retire-probe.py --census`（全 repo 走一遍，約
   2 分鐘，會超過預設 Bash timeout，用背景執行）。
4. 照 §5 的層序做下一個 project，一個 project 一輪 gate。
5. 更新 §6 的表。
