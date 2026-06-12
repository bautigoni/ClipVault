#!/bin/bash
for p in 8765 9123 9876 10999 11999 12999 13999; do
  if ! ss -tln 2>/dev/null | grep -q ":$p "; then
    echo "free: $p"
    exit 0
  fi
done
echo "no free port"
exit 1
