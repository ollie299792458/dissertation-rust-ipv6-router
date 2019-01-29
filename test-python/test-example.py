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
        switch = self.addSwitch('s3')

        # Add links
        self.addLink(left_host, switch)
        self.addLink(switch, right_host)


def setup_ipv6(net, node_ids, switch_ids):
    addresses = {}
    for switch_id in switch_ids :
        switch = net.get(switch_id)
        switch.cmd("sysctl net.ipv6.conf.all.disable_ipv6=0")

    count = 1
    for node_id in node_ids :
        node = net.get(node_id)
        address = "fc00::"+str(count)
        node.cmd("ifconfig "+node_id+"-eth0 inet6 add "+address+"/64")
        addresses[node_id] = address
        count = count + 1

    return addresses


def run():
    topo = ExampleTopo()
    net = Mininet(topo=topo)
    net.start()
    ipv6_addresses = setup_ipv6(net, {'h1','h2'}, {'s3'})
    info('Example test starting\n')
    node = net.get('h1')
    node2 = net.get('h2')
    print(ipv6_addresses[node2.name])
    node2.cmdPrint("ifconfig")
    node.cmdPrint("ping6 -w 2 "+ipv6_addresses[node2.name])
    info('Example test completed\n')
    net.stop()


if __name__ == '__main__':
    setLogLevel('info')
    run()
