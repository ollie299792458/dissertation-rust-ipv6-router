#!/usr/bin/env bash
# run using "sudo test.sh"
case $1 in
    -c)
        mn -c
        ;;
    *)
        python -E test-python/test-example.py
        ;;
esac