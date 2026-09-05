#!/usr/bin/env bash

sift_remove_ephemeral_evidence_secrets() {
  [[ "$#" == "1" ]] || return 1
  local evidence_dir="$1"
  local kubernetes_dir token_file

  [[ "$evidence_dir" == /* \
    && -d "$evidence_dir" \
    && ! -L "$evidence_dir" ]] || return 1
  kubernetes_dir="$evidence_dir/kubernetes"
  if [[ ! -e "$kubernetes_dir" && ! -L "$kubernetes_dir" ]]; then
    return 0
  fi
  [[ -d "$kubernetes_dir" && ! -L "$kubernetes_dir" ]] || return 1

  token_file="$kubernetes_dir/sift-rig.token"
  if [[ -d "$token_file" && ! -L "$token_file" ]]; then
    return 1
  fi
  rm -f -- "$token_file" || return 1
  [[ ! -e "$token_file" && ! -L "$token_file" ]]
}
