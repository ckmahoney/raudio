#!/bin/bash

# Base directory containing all sample categories
BASE_DIR="/home/naltroc/apps/raudio/audio-samples"

# Category-specific settings
declare -A SETTINGS=(
  ["kick"]="-12:-2:6"
  ["perc"]="-13:-2.5:8"
  ["hats/short"]="-14:-2.5:8"
  ["hats/long"]="-12:-2:12"
)

# Process each category
for category in "${!SETTINGS[@]}"; do
  # Parse loudness settings
  IFS=':' read -r LUFS TP LRA <<< "${SETTINGS[$category]}"

  # Define the directory and label
  DIR="$BASE_DIR/$category"
  LABEL="$category" # Replace dashes with underscores for filenames

  # Create trimmed directory
  trimmed_DIR="$DIR/trimmed"
  mkdir -p "$trimmed_DIR"

  # Initialize counter
  i=1

  echo "Processing $category samples..."

  for file in "$DIR"/*; do
    if [[ -f "$file" ]]; then
      base=$(basename "$file")
      ext="${file##*.}"

      ffmpeg -i "$file" -af "silenceremove=stop_periods=-1:stop_threshold=-60dB" "$OUTPUT_DIR/trimmed/$base.$ext"

      ((i++))
    fi
  done

  # Adjust the count for accurate reporting
  ((i--))
  echo "Completed trimming $category. Updated $i files."
done

echo "All categories trimmed."
