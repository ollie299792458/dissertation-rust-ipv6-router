from mininet.log import info, setLogLevel
from .test_framework import TestFramework


def run():
    test = TestFramework()

    left = test.addHost('h1', mac='00:00:00:00:01:00')
    right = test.addHost('h2', mac='00:00:00:00:02:00')
    router = test.addRouter('r3', mac='00:00:00:00:03:01')

    test.addLink(left, router, addr2='00:00:00:00:03:01')
    test.addLink(right, router, addr2='00:00:00:00:03:02')

    test.start()
    info('Example test starting\n')

    router.cmdPrint("ifconfig")
    right.cmdPrint("netstat -rn -A inet6")
    right.cmdPrint("ifconfig")

    test.ping6()

    router_process = test.runRouter(router)

    test.ping6()

    info('Example test completed\n')
    test.killRouter()
    test.stop()


if __name__ == '__main__':
    setLogLevel('info')
    run()