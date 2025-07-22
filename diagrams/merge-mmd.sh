#!/usr/bin/env bash

parts=(core.mmd streams.mmd evaluation.mmd classifiers.mmd prequential.mmd)

out="class-diagram.mmd"

echo '---' > "$out"

{
  echo 'config:'
  echo '  theme: default'
  echo '  layout: elk'
  echo '---'
  echo 'classDiagram'
} >> "$out"

for f in "${parts[@]}"; do
  if [[ -f $f ]]; then
    tail -n +2 "$f" >> "$out"
    echo ""         >> "$out"
  else
    echo "File \"$f\" not found, ignoring..." >&2
  fi
done

cat class-diagram.mmd | pbcopy
echo ""
echo "Done!"
echo "'class-diagram.mmd' copied to clipboard!"
