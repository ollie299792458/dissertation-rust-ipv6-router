from mininet.cli import CLI
from mininet.log import info, setLogLevel
from mininet.net import Mininet
from .test_framework import TestFramework


class ExampleTopo(TestFramework):

    def __init__(self,*args, **params):
        super(ExampleTopo, self).__init__(*args, **params)
        # Create custom topo.

        # Add hosts and switches
        left_host = self.addHost('h1')
        right_host = self.addHost('h2')
        switch = self.addSwitch('s3')

        self.addLink(left_host, switch)
        self.addLink(switch, right_host)

    def run(self):
        net = Mininet(topo=self)
        info('Example test starting\n')

        node = net.get('h1')
        node2 = net.get('h2')

        net.start()

        node.cmdPrint("ifconfig")
        node.cmdPrint("ping6 -W 2 -I " + node.name + "-eth0 " + self.address(node2.name))
        self.pingLink(node2, node)
        node.cmdPrint("route -6 -n")
        node.cmdPrint("ip -6 neighbor show")
        info('Example test completed\n')
        net.stop()


def run():
    test = ExampleTopo()
    test.run()


if __name__ == '__main__':
    setLogLevel('info')
    run()
