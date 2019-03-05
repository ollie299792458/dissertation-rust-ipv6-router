import os
import subprocess
import sys
from time import sleep

from mininet.link import Intf, Link
from mininet.log import info, error, output
from mininet.net import Mininet
from mininet.node import OVSKernelSwitch, Host, DefaultController
from mininet.topo import Topo


class TestFramework(Mininet):

    #todo allow specifying ipv6 addresses on creation

    def __init__(self, topo=None, switch=OVSKernelSwitch, host=Host,
                 controller=DefaultController, link=Link, intf=Intf,
                 build=False, xterms=False, cleanup=False, ipBase='10.0.0.0/8',
                 inNamespace=False,
                 autoSetMacs=False, autoStaticArp=False, autoPinCpus=False,
                 listenPort=None, waitConnected=False):
        self.__host_to_address = {}
        self.__address_to_host = {}
        self.__count = 0
        self.__buildRouter()
        self.__router_process = None
        self.__router = None
        self.__default = None
        super(TestFramework, self).__init__(topo, switch, host,
                                            controller, link, intf,
                                            build, xterms, cleanup, ipBase,
                                            inNamespace,
                                            autoSetMacs, autoStaticArp, autoPinCpus,
                                            listenPort, waitConnected)

    def __buildRouter(self):
        info("Building router from source\n")
        # todo fix this
        # subprocess.call(['cargo build'], cwd='rust/router', shell=True)
        info("Router built\n")
        return

    def __incCount(self):
        result = self.__count
        self.__count += 1
        return result

    def __add_ipv6_address(self, node):
        count = self.__incCount()
        # maybe get address from node rather than allocating a new one; done for parsing, visual debug, & compatibility
        address = "fc00::" + str(count)
        name = node.name
        # add address to interface
        for port, intf in node.intfs.iteritems():
            node.cmd("ifconfig " + intf.name + " inet6 add " + address + "/64")
        # add addresses to lookup maps
        self.__host_to_address[name] = address
        self.__address_to_host[address] = name
        info("Given IPv6 address: " + address + " to node: " + node.name + "\n")
        return address

    #must be called after __add_ipv6_address
    def __add_default_route(self, node, gateway_address):
        while "tentative" in node.cmd("ip -6 addr"):
            sleep(0.1)
        node.cmd("ip -6 route add default via " + gateway_address +" src "+self.address(node.name))
        info("Set default route: "+gateway_address+", for: "+node.name+"\n")


    # may return stale addresses if hosts are removed TODO
    def address(self, name):
        return self.__host_to_address[name]

    def name(self, address):
        return self.__address_to_host[address]

    def start(self):
        info("Thank you for using Oliver's Mininet Test Framework\n")

        result = super(TestFramework, self).start()

        for switch in self.switches:
            switch.cmd("sysctl net.ipv6.conf.all.disable_ipv6=0")

        self.__add_ipv6_address(self.__default)

        for host in self.hosts:
            if (not host == self.__router) and (not host == self.__default):
                self.__add_ipv6_address(host)

        gateway_address = self.__add_ipv6_address(self.__router)

        self.__add_default_route(self.__default, gateway_address)

        for host in self.hosts:
            # pings from router and default are slightly meaningless
            if (not host == self.__router) and (not host == self.__default):
                self.__add_default_route(host, gateway_address)

        # may want to change
        self.__add_default_route(self.__router, self.address(self.__default.name))

        return result

    def stop(self):
        if self.__router_process:
            info("Stopping router")
            self.__router_process.kill()
        return super(TestFramework, self).stop()

    def addLink(self, node1, node2, port1=None, port2=None,
                cls=None, **params):

        return super(TestFramework, self).addLink(node1, node2, port1, port2,
                                                  cls, **params)

    # todo only allow a single router & default to be added
    def addRouter(self, name, cls=None, **params):
        self.__router = self.addHost(name, cls, **params)
        return self.__router

    def addDefault(self, name, cls=None, **params):
        self.__default = self.addHost(name,cls, **params)
        return self.__default

    def ping6(self, hosts=None, timeout=None):
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
            output('*** Ping6: testing ping reachability\n')
        for node in hosts:
            # don't ping from router
            if self.__router == node:
                continue
            output('%s -> ' % node.name)
            for dest in hosts:
                if node != dest:
                    opts = ''
                    if timeout:
                        opts = '-W %s' % timeout
                    if dest.intfs:
                        intfcount = 0
                        sent, received = (0, 0)
                        for _ in node.intfs:
                            result = node.cmd('ping6 -I %s-eth%s -c1 %s %s' %
                                              (str(node), str(intfcount), opts, self.address(str(dest))))
                            intfcount += 1
                            justsent, justreceived = self._parsePing(result)
                            sent, received = (sent + justsent, received + justreceived)
                    else:
                        sent, received = 0, 0
                    packets += sent
                    if received > sent:
                        error('*** Error: received too many packets')
                        error('%s' % result)
                        node.cmdPrint('route')
                        exit(1)
                    lost += sent - received
                    output(('%s ' % dest.name) if received else 'X ')
            output('\n')
        if packets > 0:
            ploss = 100.0 * lost / packets
            received = packets - lost
            output("*** Results: %i%% dropped (%d/%d received)\n" %
                   (ploss, received, packets))
        else:
            ploss = 0
            output("*** Warning: No packets sent\n")
        return ploss

    # todo run on a specific node's interfaces
    def runRouter(self, router, **args):
        info("Starting router")
        self.__router_process = router.popen("./rust/router/target/debug/router", stdout=sys.stdout, stderr=sys.stdout,
                                             shell=True)
        return self.__router_process
