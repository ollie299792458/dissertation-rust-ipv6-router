from mininet.cli import CLI
from mininet.log import info, setLogLevel
from mininet.net import Mininet
from mininet.topo import Topo


class ExampleTopo(Topo):
    "Simple topology example."

    def build(self):
        # Create custom topo.

        # Add hosts and switches
        left_host = self.addHost('h1')
        right_host = self.addHost('h2')
        left_switch = self.addSwitch('s3')
        right_switch = self.addSwitch('s4')

        # Add links
        self.addLink(left_host, left_switch)
        self.addLink(left_switch, right_switch)
        self.addLink(right_switch, right_host)


def run():
    topo = ExampleTopo()
    net = Mininet(topo=topo)
    net.start()
    info('Example test starting\n')
    node = net.get('h1')
    node2 = net.get('h2')
    node.cmdPrint("ping -w 2 "+node2.IP())
    info('Example test completed\n')
    net.stop()


if __name__ == '__main__':
    setLogLevel('info')
    run()
