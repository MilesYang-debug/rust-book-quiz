#!/bin/sh
# One-click build artifact cleanup (mirrors the artifact list in .gitignore).
# Linux/macOS counterpart of clean.cmd.
# Usage:
#   ./clean.sh        remove compile caches and intermediate outputs
#   ./clean.sh all    also remove final deliverables (RustBookQuiz.exe / .apk)
cd "$(dirname "$0")" || exit 1

echo "Cleaning build artifacts..."

for d in \
  app/target \
  app/dist \
  app/src-tauri/target \
  app/src-tauri/gen/schemas \
  app/src-tauri/gen/android/.gradle \
  app/src-tauri/gen/android/.kotlin \
  app/src-tauri/gen/android/build \
  app/src-tauri/gen/android/app/build \
  app/src-tauri/gen/android/app/.cxx \
  app/src-tauri/gen/android/buildSrc/build \
  app/src-tauri/gen/android/buildSrc/.gradle \
  app/src-tauri/gen/android/buildSrc/.kotlin
do
  if [ -d "$d" ]; then
    echo "  removing $d"
    rm -rf "$d"
  fi
done

if [ "$1" = "all" ]; then
  for f in RustBookQuiz.exe RustBookQuiz.apk RustBookQuiz.apk.idsig; do
    if [ -f "$f" ]; then
      echo "  removing $f"
      rm -f "$f"
    fi
  done
fi

echo "Done."
