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
  # Define input and output directories
  DIR="$BASE_DIR/$category"
  IN_DIR="$DIR/normalized"
  OUT_DIR="$BASE_DIR/$category"
  LABEL="${category//\//_}" # Replace slashes with underscores for labels

  # Create the output directory if it doesn't exist
  mkdir -p "$OUT_DIR"

  # Initialize counter
  i=1

  echo "Processing $category samples..."

  for file in "$IN_DIR"/*.wav; do
    if [[ -f "$file" ]]; then
      # Get file details
      base_name=$(basename "$file")
      ext="${base_name##*.}" # File extension

      # Generate new file name
      new_name="${LABEL}-${i}.${ext}"

      # Move the file to the output directory with the new name
      mv "$file" "$OUT_DIR/$new_name"

      ((i++))
    fi
  done

  rm -rf "$DIR/trimmed"
  rmdir "$DIR/normalized"
  rmdir "$DIR/trimmed"


  # Adjust the count for accurate reporting
  ((i--))
  echo "Completed processing $category. Updated $i files."
done

echo "All categories processed."
