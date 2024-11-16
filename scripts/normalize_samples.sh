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
  IN_DIR="$DIR/trimmed"
  LABEL="$category" # Replace dashes with underscores for filenames

  OUTPUT_DIR="$DIR/normalized"
  mkdir -p "$OUTPUT_DIR"

  # Initialize counter
  i=1

  echo "Processing $category samples..."

  for file in "$IN_DIR"/*; do 
    if [[ -f "$file" ]]; then
      base_name=$(basename "$file")
      ext="${base_name##*.}" # File extension
      name="${base_name%.*}" # File name without extension

      # Generate new file name
      new_name="${name}-${i}.${ext}"
      ffmpeg -i "$file" -af "loudnorm=I=$LUFS:TP=$TP:LRA=$LRA" -y "$OUTPUT_DIR/$new_name"
      ((i++))
    fi
  done

  # Adjust the count for accurate reporting
  ((i--))
  echo "Completed processing $category. Updated $i files."
done

echo "All categories processed."
