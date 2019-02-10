from mininet.log import info, setLogLevel
from .test_framework import TestFramework


def run():
    test = TestFramework()

    left_host = test.addHost('h1')
    right_host = test.addHost('h2')

    test.addLink(left_host, right_host)

    test.start()
    info('Example test starting\n')

    test.ping6()
    # ran twice to allow correct discovery
    test.ping6()
    info('Example test completed\n')
    test.stop()


if __name__ == '__main__':
    setLogLevel('info')
    run()
