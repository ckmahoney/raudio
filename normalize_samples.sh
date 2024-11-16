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
  LABEL="category" # Replace dashes with underscores for filenames

  # Create processed directory
  PROCESSED_DIR="$DIR/processed"
  mkdir -p "$PROCESSED_DIR"

  # Initialize counter
  i=1

  echo "Processing $category samples..."

  for file in "$DIR"/*; do
    if [[ -f "$file" ]]; then
      ext="${file##*.}"
      new_name="${LABEL}_${i}.${ext}"
      ffmpeg -i "$file" -af "loudnorm=I=$LUFS:TP=$TP:LRA=$LRA" -y "$PROCESSED_DIR/$new_name"
      ((i++))
    fi
  done

  # Adjust the count for accurate reporting
  ((i--))
  echo "Completed processing $category. Updated $i files."
done

echo "All categories processed."
