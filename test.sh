#!/usr/bin/env bash
# run using "sudo test.sh"
case $1 in
    -c) #clean up
        mn -c
        fuser -k 6633/tcp
        ;;
    -6ex) #try the example
        python -E test_python/research_ipv6_example.py
        ;;
    -v) #version
        python --version
        mn --version
        ;;
    *) #placeholder - break out tests
        python -m test_python.test_example
        ;;
esac