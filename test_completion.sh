#!/bin/bash
source ./crabster.bash

# Mock COMP_WORDS and COMP_CWORD for "crabster stop "
COMP_WORDS=(crabster stop "")
COMP_CWORD=2
cur=""
prev="stop"

# Run the completion function
_crabster crabster "$cur" "$prev"

echo "Suggestions for 'crabster stop ':"
echo "${COMPREPLY[@]}"

# Mock COMP_WORDS and COMP_CWORD for "crabster "
COMP_WORDS=(crabster "")
COMP_CWORD=1
cur=""
prev="crabster"
_crabster crabster "$cur" "$prev"
echo "Suggestions for 'crabster ':"
echo "${COMPREPLY[@]}"
