#!/bin/bash

INPUT_FILES=$(ls inputs/*.csv)
failed_any=0

for infile in $INPUT_FILES; do
    outfile="$infile""_output"
    correct="$infile""_expected"
    cargo run -q -- $infile > $outfile
    if cmp $outfile $correct; then
        echo '---' $infile " PASS"
    else
        echo '---' $infile " FAIL"
    failed_any=1
    fi
    rm $outfile
done

if [ $failed_any -eq 0 ]; then
    echo "All tests PASS"
else
    echo "Some tests FAILED"
fi