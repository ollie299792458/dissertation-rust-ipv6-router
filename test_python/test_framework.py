from mininet.net import Mininet
from mininet.topo import Topo


# To use: extend and implement build(self) method
class TestFramework(Topo):

    def __init__(self, *args, **params):
        self.__net = None
        self.__host_to_address = None
        self.__address_to_host = None
        self.__host_names = []
        self.__switch_names = []
        self.__count = 0
        super(TestFramework, self).__init__(*args, **params)

    def __incCount(self):
        self.__count += 1
        return self.__count

    def __add_ipv6_address(self, node, name):
        count = self.__incCount()
        # maybe get address from node rather than allocating a new one - but parsing and compatibility
        address = "fc00::" + str(count)
        # add address to interface
        node.cmd("ifconfig " + name + "-eth0 inet6 add " + address + "/64")
        # add addresses to lookup maps
        self.__host_to_address[name] = address
        self.__address_to_host[address] = name
        return address

    def address(self, name):
        return self.__host_to_address[name]

    def name(self, address):
        return self.__address_to_host[address]

    def addHost(self, name, **opts):
        self.__host_names.append(name)
        node = super(TestFramework, self).addHost(name, **opts)
        return node

    def addSwitch(self, name, **opts):
        self.__switch_names.append(name)
        switch = super(TestFramework, self).addSwitch('s3', **opts)
        return switch

    def addLink(self, left, right, **opts):
        return super(TestFramework, self).addLink(left, right, **opts)

    def start(self, topo):
        self.__net = Mininet(topo=topo)
        self.__net.start()
        for name in self.__host_names :
            node = self.__net.getNodeByName(name)
            self.__add_ipv6_address(node, name)
        #do we actually need this?
        for name in self.__switch_names :
            switch = self.__net.getNodeByName(name)
            switch.cmd("sysctl net.ipv6.conf.all.disable_ipv6=0")
        return self.__net

    def stop(self):
        return self.__net.stop()

    # check link is up between nodes
    def pingLink(self, left_node, right_node):
        left_node.cmdPrint("ping6 -W 3 -I " + left_node.name + "-eth0 " + self.address(right_node.name))
        right_node.cmdPrint("ping6 -W 3 -I " + right_node.name + "-eth0 " + self.address(left_node.name))
