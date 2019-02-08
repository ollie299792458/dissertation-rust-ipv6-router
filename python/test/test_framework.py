from mininet.link import Intf, Link
from mininet.log import info, error, output
from mininet.net import Mininet
from mininet.node import OVSKernelSwitch, Host, DefaultController
from mininet.topo import Topo


# To use: extend and implement build(self) method
class TestFramework(Mininet):

    def __init__(self, topo=None, switch=OVSKernelSwitch, host=Host,
                 controller=DefaultController, link=Link, intf=Intf,
                 build=False, xterms=False, cleanup=False, ipBase='10.0.0.0/8',
                 inNamespace=False,
                 autoSetMacs=False, autoStaticArp=False, autoPinCpus=False,
                 listenPort=None, waitConnected=False):
        self.__host_to_address = {}
        self.__address_to_host = {}
        self.__count = 0
        super(TestFramework, self).__init__(topo, switch, host,
                                            controller, link, intf,
                                            build, xterms, cleanup, ipBase,
                                            inNamespace,
                                            autoSetMacs, autoStaticArp, autoPinCpus,
                                            listenPort, waitConnected)

    def __incCount(self):
        self.__count += 1
        return self.__count

    def __add_ipv6_address(self, node):
        count = self.__incCount()
        # maybe get address from node rather than allocating a new one; done for parsing, visual debug, & compatibility
        address = "fc00::" + str(count)
        name = node.name
        # add address to interface
        node.cmd("ifconfig " + name + "-eth0 inet6 add " + address + "/64")
        # add addresses to lookup maps
        self.__host_to_address[name] = address
        self.__address_to_host[address] = name
        return address

    # may return stale addresses if hosts are removed TODO
    def address(self, name):
        return self.__host_to_address[name]

    def name(self, address):
        return self.__address_to_host[address]

    def start(self):
        info("Thank you for using Oliver's Mininet Test Framework\n")
        for host in self.hosts :
            address = self.__add_ipv6_address(host)
            info("Given IPv6 address: "+address+" to node: "+host.name+"\n")

        for switch in self.switches :
            switch.cmd("sysctl net.ipv6.conf.all.disable_ipv6=0")

        return super(TestFramework, self).start()

    def ping6( self, hosts=None, timeout=None ):
        """Ping between all specified hosts.
           hosts: list of hosts
           timeout: time to wait for a response, as string
           returns: ploss packet loss percentage"""
        # should we check if running?
        packets = 0
        lost = 0
        ploss = None
        if not hosts:
            hosts = self.hosts
            output( '*** Ping6: testing ping reachability\n' )
        for node in hosts:
            output( '%s -> ' % node.name )
            for dest in hosts:
                if node != dest:
                    opts = ''
                    if timeout:
                        opts = '-W %s' % timeout
                    if dest.intfs:
                        result = node.cmd( 'ping6 -c1 %s %s' %
                                           (opts, self.address(str(node))) )
                        sent, received = self._parsePing( result )
                    else:
                        sent, received = 0, 0
                    packets += sent
                    if received > sent:
                        error( '*** Error: received too many packets' )
                        error( '%s' % result )
                        node.cmdPrint( 'route' )
                        exit( 1 )
                    lost += sent - received
                    output( ( '%s ' % dest.name ) if received else 'X ' )
            output( '\n' )
        if packets > 0:
            ploss = 100.0 * lost / packets
            received = packets - lost
            output( "*** Results: %i%% dropped (%d/%d received)\n" %
                    ( ploss, received, packets ) )
        else:
            ploss = 0
            output( "*** Warning: No packets sent\n" )
        return ploss