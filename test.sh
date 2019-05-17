#!/usr/bin/env bash
# run using "sudo test.sh"
# make sure to build router first
export RUST_BACKTRACE=1
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
        python -E python/luxing/research_ipv6_example.py
        ;;
    1112)
        python -m python.luxing.test_1112
        ;;
    11211)
        python -m python.luxing.test_11211
        ;;
    11212)
        python -m python.luxing.test_11212
        ;;
    1211)
        python -m python.luxing.test_1211
        ;;
    1212)
        python -m python.luxing.test_1212
        ;;
    1214)
        python -m python.luxing.test_1214
        ;;
    1215)
        python -m python.luxing.test_1215
        ;;
    1216)
        python -m python.luxing.test_1216
        ;;
    1217)
        python -m python.luxing.test_1217
        ;;
    web)
        python -m python.luxing.test_web_server
        ;;
    ex) #example test of ipv6 wrapper
        python -m python.luxing.test_example
        ;;
    *) #if in doubt, run the playground test
        python -m python.luxing.test_playground
        ;;
esac