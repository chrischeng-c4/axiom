#!/usr/bin/env bash
# ~/.claude/statusline.sh — Claude Code status line
# Displays: model | cwd | git branch | context tokens / 200k (pct%)

input=$(cat)

# Model
model=$(echo "$input" | jq -r '.model.display_name // .model.id // "?"')

# CWD basename
cwd=$(echo "$input" | jq -r '.workspace.current_dir // .cwd // ""')
cwd_base=$(basename "$cwd")

# Git branch (fast, no lock)
branch=$(GIT_DIR="$cwd/.git" git --no-optional-locks -C "$cwd" rev-parse --abbrev-ref HEAD 2>/dev/null)
[ -z "$branch" ] && branch="no-git"

# Context window
ctx_size=$(echo "$input" | jq -r '.context_window.context_window_size // 200000')
used_pct=$(echo "$input" | jq -r '.context_window.used_percentage // empty')

# Token count from current_usage (most recent API call context)
tokens_used=$(echo "$input" | jq -r '
  .context_window.current_usage
  | if . == null then empty
    else (.input_tokens // 0)
       + (.cache_read_input_tokens // 0)
       + (.cache_creation_input_tokens // 0)
       + (.output_tokens // 0)
    end
' 2>/dev/null)

# Format token count
if [ -n "$tokens_used" ] && [ -n "$used_pct" ]; then
  # Round pct to integer
  pct_int=$(printf '%.0f' "$used_pct")
  ctx_part="${tokens_used}/${ctx_size} (${pct_int}%)"
elif [ -n "$used_pct" ]; then
  pct_int=$(printf '%.0f' "$used_pct")
  ctx_part="${pct_int}% of ${ctx_size}"
else
  ctx_part="no msgs yet"
fi

# Rate-limit quotas (remaining %, reset time)
rl_5h=$(echo "$input" | jq -r '.rate_limits.five_hour.used_percentage // empty')
rl_5h_reset=$(echo "$input" | jq -r '.rate_limits.five_hour.resets_at // empty')
rl_7d=$(echo "$input" | jq -r '.rate_limits.seven_day.used_percentage // empty')
rl_7d_reset=$(echo "$input" | jq -r '.rate_limits.seven_day.resets_at // empty')

fmt_quota() {
  local used="$1" reset_ts="$2" label="$3"
  [ -z "$used" ] && return
  local remain
  remain=$(awk -v u="$used" 'BEGIN{printf "%.0f", 100 - u}')
  local reset_fmt=""
  if [ -n "$reset_ts" ]; then
    reset_fmt=$(date -r "$reset_ts" "+%H:%M" 2>/dev/null \
      || date -d "@$reset_ts" "+%H:%M" 2>/dev/null)
  fi
  if [ -n "$reset_fmt" ]; then
    printf '%s:%s%%→%s' "$label" "$remain" "$reset_fmt"
  else
    printf '%s:%s%%' "$label" "$remain"
  fi
}

q5h=$(fmt_quota "$rl_5h" "$rl_5h_reset" "5h")
q7d=$(fmt_quota "$rl_7d" "$rl_7d_reset" "7d")

out="$model | $cwd_base | $branch | ctx: $ctx_part"
[ -n "$q5h" ] && out="$out | $q5h"
[ -n "$q7d" ] && out="$out | $q7d"
printf '%s' "$out"
