import time

from mininet.log import info, setLogLevel
from .test_framework import TestFramework


def run():
    test = TestFramework()

    host1 = test.addHost('h1', mac='00:00:00:00:01:00')
    host2 = test.addHost('h2', mac='00:00:00:00:02:00')
    router = test.addHost('r3', mac='00:00:00:00:03:01')

    test.addLink(host1, router, addr2='00:00:00:00:03:01')
    test.addLink(host2, router, addr2='00:00:00:00:03:02')

    test.start()
    info('Example test starting\n')

    router.cmdPrint("ifconfig")

    test.ping6()

    test.runRouter(router)

    test.ping6()

    info('Example test completed\n')
    test.stop()


if __name__ == '__main__':
    setLogLevel('info')
    run()