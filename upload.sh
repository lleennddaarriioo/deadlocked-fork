#!/usr/bin/env bash

# Script to launch deadlocked, automatically step through all GUI tabs, capture full-system screenshots of each tab, stage, commit, and upload (push) to Git.

PREVIEWS_DIR="media/previews"
mkdir -p "$PREVIEWS_DIR"

echo "🖥️  Launching deadlocked GUI tab auto-capturer..."
cargo run --release --bin deadlocked -- --demo-screenshots

echo "📸 All GUI tab screenshots generated in ${PREVIEWS_DIR}/"

# Stage changes
git add .

# Default commit message if none provided as argument
COMMIT_MSG="${1:-"update: auto-captured GUI tab screenshots and codebase updates"}"

echo "🚀 Committing and uploading updates..."
git commit -m "$COMMIT_MSG"
git push
