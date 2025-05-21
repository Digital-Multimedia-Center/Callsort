#!/bin/bash

# Input SVG
INPUT_SVG="michigan-state-university.svg"

# Desired square sizes (from your file list)
SIZES=(30 32 44 71 89 107 128 142 150 284 310)

for SIZE in "${SIZES[@]}"; do
  TMP_FILE="tmp-${SIZE}.png"
  OUT_FILE="Square${SIZE}x${SIZE}Logo.png"

  # Step 1: Export SVG to PNG with height = SIZE (preserving aspect ratio)
  rsvg-convert -h $SIZE "$INPUT_SVG" -o "$TMP_FILE"

  # Step 2: Create square canvas with white background and center the image
  convert "$TMP_FILE" -gravity center -background white -extent ${SIZE}x${SIZE} "$OUT_FILE"

  # Step 3: Clean up temp file
  rm "$TMP_FILE"

  echo "Created $OUT_FILE"
done
