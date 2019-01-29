#!/usr/bin/env bash
# run using "sudo test.sh"
case $1 in
    -c)
        mn -c
        fuser -k 6633/tcp
        ;;
    -6ex)
        python -E test-python/research-ipv6-example.py
        ;;
    *)
        python -E test-python/test-example.py
        ;;
esac