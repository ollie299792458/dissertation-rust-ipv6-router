#!/usr/bin/env bash
# run using "sudo test.sh"
case $1 in
    -c) #clean up
        mn -c
        fuser -k 6633/tcp
        ;;
    -6ex) #try the example
        python -E python/test/research_ipv6_example.py
        ;;
    -v) #version
        python --version
        mn --version
        ;;
    *) #placeholder - break out tests
        python -m python.test.test_example
        ;;
esac