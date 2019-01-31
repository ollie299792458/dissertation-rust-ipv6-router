from mininet.cli import CLI
from mininet.log import info, setLogLevel
from mininet.net import Mininet
from .test_framework import TestFramework


def run():
    test = TestFramework()

    left_host = test.addHost('h1')
    right_host = test.addHost('h2')
    switch = test.addSwitch('s3')

    test.addLink(left_host, switch)
    test.addLink(switch, right_host)

    test.start()
    info('Example test starting\n')

    node = test.get('h1')
    node2 = test.get('h2')

    node.cmdPrint("ifconfig")
    node.cmdPrint("ping6 -w 2 -I " + str(node) + "-eth0 " + self.address(str(node2)))
    test.ping()
    node.cmdPrint("route -6 -n")
    node.cmdPrint("ip -6 neighbor show")
    info('Example test completed\n')
    net.stop()


if __name__ == '__main__':
    setLogLevel('info')
    run()
