#!/usr/bin/env bash
# run using "sudo test.sh"
case $1 in
    -c) #clean up
        mn -c
        fuser -k 6633/tcp
        ;;
    -v) #version
        python --version
        mn --version
        ;;
    tw_ex) #try the taiwanese example
        python -E python/test/research_ipv6_example.py
        ;;
    ex) #placeholder - break out tests
        python -m python.test.test_example
        ;;
esac