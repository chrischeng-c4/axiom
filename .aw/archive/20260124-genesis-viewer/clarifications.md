---
change: genesis-viewer
date: 2026-01-24
---

# Clarifications

## Q1: URL Pattern
- **Question**: Viewer 的 URL 結構應該怎麼設計？
- **Answer**: /{project}/genesis
- **Rationale**: 簡潔且明確表達這是 genesis 專用的 viewer 路徑

## Q2: Open Method
- **Question**: Skill 觸發後應該怎麼開啟 viewer？
- **Answer**: 自動開啟瀏覽器
- **Rationale**: 使用 open/xdg-open 命令開啟預設瀏覽器，提供最佳使用體驗

## Q3: Navigation
- **Question**: Viewer 的資料夾結構導航應該支援哪些功能？
- **Answer**: Tree view + 內容預覽
- **Rationale**: 左側樹狀結構導航，右側顯示選中檔案內容，標準的文件瀏覽器體驗

## Q4: Markdown
- **Question**: Markdown 渲染應該支援哪些進階功能？
- **Answer**: 全功能 (Mermaid + 高亮 + LaTeX + 表格排序等)
- **Rationale**: genesis 文件包含大量 Mermaid 圖表和程式碼區塊，需要完整的渲染支援

