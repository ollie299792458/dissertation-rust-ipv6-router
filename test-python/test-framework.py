from mininet.net import Mininet
from mininet.topo import Topo


# To use: extend and implement build(self) method
class MyMininetTestingFramework(Topo):

    def __init__(self, *args, **params):
        super().__init__(*args, **params)
        self.net = None
        self.node_to_address = None
        self.address_to_node = None
        self.count = 0

    def __add_ipv6_address(self, node, name):
        self.count += 1
        # maybe get address from node rather than allocating a new one - but parsing and compatibility
        address = "fc00::" + str(self.count)
        # add address to interface
        node.cmd("ifconfig " + name + "-eth0 inet6 add " + address + "/64")
        # add addresses to lookup maps
        self.node_to_address[name] = address
        self.address_to_node[address] = name
        return address

    def address(self, name):
        return self.node_to_address[name]

    def name(self, address):
        return self.address_to_node[address]

    def addNode(self, name, **kwargs):
        node = self.addHost(name, **kwargs)
        address = self.__add_ipv6_address(node, name)
        return node, address

    def addSwitch(self, name, **kwargs):
        switch = self.addSwitch('s3', **kwargs)
        # enable ipv6
        switch.cmd("sysctl net.ipv6.conf.all.disable_ipv6=0")
        return switch

    def addLink(self, left, right, **kwargs):
        return self.addLink(left, right, **kwargs)

    def start(self, topo):
        self.net = Mininet(topo=topo)
        self.net.start()
        return self.net

    def stop(self):
        return self.net.stop()

    # check link is up between nodes
    def pingLink(self, left_node, right_node):
        left_node.cmd("ping6 -C 1 -I " + left_node.name + "-eth0 " + self.address(right_node.name))
        right_node.cmdPrint("ping6 -C 1 -I " + right_node.name + "-eth0 " + self.address(left_node.name))
