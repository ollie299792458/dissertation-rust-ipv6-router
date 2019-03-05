import time

from mininet.log import info, setLogLevel
from .test_framework import TestFramework


def run():
    test = TestFramework()

    default = test.addDefault('d0', mac='ff:00:00:00:00:00')
    left = test.addHost('h1', mac='00:00:00:00:01:00') #todo allow setting ipv6 address
    right = test.addHost('h2', mac='00:00:00:00:02:00')
    router = test.addRouter('r3', mac='00:00:00:00:03:00')

    test.addLink(default, router, addr2='00:00:00:00:03:00')
    test.addLink(left, router, addr2='00:00:00:00:03:01')
    test.addLink(right, router, addr2='00:00:00:00:03:02')

    test.start()
    info('Example test starting\n')

    default.cmdPrint("ifconfig")
    left.cmdPrint("ifconfig")
    right.cmdPrint("ifconfig")
    router.cmdPrint("ifconfig")

    router_process = test.runRouter(router)
    time.sleep(1)
    test.ping6()

    info('Example test completed\n')
    test.stop()


if __name__ == '__main__':
    setLogLevel('info')
    run()