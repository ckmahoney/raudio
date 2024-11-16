#!/bin/bash

# Base directory containing all sample categories
BASE_DIR="/home/naltroc/apps/raudio/audio-samples"

# Category-specific settings
declare -A SETTINGS=(
  ["kick"]="stop_threshold=-36dB"
  ["perc"]="stop_threshold=-50dB"
  ["hats/short"]="stop_threshold=-32dB"
  ["hats/long"]="stop_threshold=-60dB"
)

# Process each category
for category in "${!SETTINGS[@]}"; do
  
  # Define the directory and label
  DIR="$BASE_DIR/$category"
  LABEL="$category" # Replace dashes with underscores for filenames

  # Create trimmed directory
  OUTPUT_DIR="$DIR/trimmed"
  mkdir -p "$OUTPUT_DIR"

  # Initialize counter
  i=1

  echo "Processing $category samples..."

  for file in "$DIR"/*; do
    if [[ -f "$file" ]]; then
      base=$(basename "$file")
      ext="${file##*.}"

      ffmpeg -i "$file" -af "silenceremove=stop_periods=-1:$THRESHOLD" -y "$OUTPUT_DIR/$base-$i.$ext"

      ((i++))
    fi
  done

  # Adjust the count for accurate reporting
  ((i--))
  echo "Completed trimming $category. Updated $i files."
done

echo "All categories trimmed."
