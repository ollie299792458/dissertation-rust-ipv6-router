#!/usr/bin/env bash
# run using "sudo test.sh"
case $1 in
    -c)
        mn -c
        fuser -k 6633/tcp
        ;;
    -6ex)
        python -E test_python/research_ipv6_example.py
        ;;
    -v)
        python --version
        mn --version
        ;;
    *)
        python -m test_python.test_example
        ;;
esac