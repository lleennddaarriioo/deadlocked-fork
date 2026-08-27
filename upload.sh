#!/usr/bin/env bash

# Script to take a screenshot of the GUI layout, stage changes, commit, and upload (push) to Git.

PREVIEWS_DIR="media/previews"
mkdir -p "$PREVIEWS_DIR"

TIMESTAMP=$(date +"%Y%m%d_%H%M%S")
SHOT_PATH="${PREVIEWS_DIR}/gui_${TIMESTAMP}.png"

echo "📸 Taking screenshot of current GUI layout..."

if command -v grim &> /dev/null; then
    grim "$SHOT_PATH"
elif command -v hyprshot &> /dev/null; then
    hyprshot -m output -o "$PREVIEWS_DIR" -f "gui_${TIMESTAMP}.png"
elif command -v spectacle &> /dev/null; then
    spectacle -b -n -o "$SHOT_PATH"
elif command -v import &> /dev/null; then
    import -window root "$SHOT_PATH"
else
    echo "⚠️ No screenshot utility (grim, hyprshot, spectacle, import) found."
fi

if [ -f "$SHOT_PATH" ]; then
    echo "✅ Screenshot saved to ${SHOT_PATH}"
fi

# Stage changes
git add .

# Default commit message if none provided as argument
COMMIT_MSG="${1:-"update: update gui screenshot and codebase"}"

echo "🚀 Committing and uploading updates..."
git commit -m "$COMMIT_MSG"
git push
