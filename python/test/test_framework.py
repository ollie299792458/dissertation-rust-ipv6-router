import os
import subprocess
import sys
from time import sleep

from mininet.link import Intf, Link
from mininet.log import info, error, output
from mininet.net import Mininet
from mininet.node import OVSKernelSwitch, Host, DefaultController
from mininet.topo import Topo
from pip._vendor.ipaddress import IPv6Address


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
        self.__mac_pairs = {}
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

    def __add_ipv6_address(self, node, address):
        name = node.name
        # add address to interface
        for port, intf in node.intfs.iteritems():
            node.cmd("ifconfig " + intf.name + " inet6 add " + address + "/64")
        # add addresses to lookup maps
        self.__host_to_address[name] = address
        self.__address_to_host[address] = name
        #info("Given IPv6 address: " + address + " to node: " + node.name + "\n")
        return address

    #must be called after __add_ipv6_address
    def __add_default_route(self, node, gateway_address):
        while "tentative" in node.cmd("ip -6 addr"):
            sleep(0.1)
        node.cmd("ip -6 route add default via " + gateway_address +" src "+self.address(node.name))
        #info("Set default route: "+gateway_address+", for: "+node.name+"\n")

    # TODO may return stale addresses if hosts are removed
    def address(self, name):
        return self.__host_to_address[name]

    def name(self, address):
        return self.__address_to_host[address]

    def start(self):
        info("Thank you for using Oliver's Mininet Test Framework\n")

        result = super(TestFramework, self).start()

        for switch in self.switches:
            switch.cmd("sysctl net.ipv6.conf.all.disable_ipv6=0")

        self.__add_ipv6_address(self.__default, self.__host_to_address.get(self.__default.name))

        for host in self.hosts:
            if (not host == self.__router) and (not host == self.__default):
                self.__add_ipv6_address(host, self.__host_to_address.get(host.name))

        router_address = self.__add_ipv6_address(self.__router, self.__host_to_address.get(self.__router.name))

        self.__add_default_route(self.__default, router_address)

        for host in self.hosts:
            # pings from router and default are slightly meaningless
            if (not host == self.__router) and (not host == self.__default):
                self.__add_default_route(host, router_address)

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

        result = super(TestFramework, self).addLink(node1, node2, port1, port2,
                                                  cls, **params)

        nodeone = node1
        nodetwo = node2
        intf1 = result.intf1
        intf2 = result.intf2
        if nodetwo == self.__router:
            nodeone = node2
            nodetwo = node1
            intf2 = result.intf1
            intf1 = result.intf2

        if nodeone == self.__router:
            if nodetwo == self.__default:
                self.__mac_pairs[nodeone.name+nodetwo.name] = intf1.mac+",ff:00:00:00:00:00"
            else:
                self.__mac_pairs[nodeone.name+nodetwo.name] = intf1.mac+","+intf2.mac


        return result

    def addIPv6Host( self, name, ipv6_address, cls=None, **params ):
        result = self.addHost(name, cls, **params)
        self.__add_ipv6_address(result,ipv6_address)
        self.__host_to_address[name] = ipv6_address
        self.__address_to_host[ipv6_address] = name
        return result

    # todo only allow a single router & default to be added
    def addRouter(self, name, ipv6_address, cls=None, **params):
        self.__router = self.addIPv6Host(name, ipv6_address, cls, **params)
        return self.__router

    def addDefault(self, name, ipv6_address, cls=None, **params):
        self.__default = self.addIPv6Host(name, ipv6_address, cls, **params)
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
        info("Starting router\n")

        file_location = "./rust/router/resource/routing.txt"

        #generate router config file
        f = open(file_location,"w+")

        f.write(self.__host_to_address[self.__default.name]+"\n")

        f.write(self.__host_to_address[self.__router.name]+"\n")

        for host in self.hosts:
            if not host == self.__router:
                f.write(self.__host_to_address[host.name]+"@"+self.__mac_pairs[router.name+host.name]+"\n")

        #add solicted multicast address

        for host in self.hosts:
            if not host == self.__router:
                solicited_multicast_address = str(IPv6Address(
                    u'ff02::1:ff' +
                    str(str(IPv6Address(u''+self.__host_to_address[host.name]).exploded)[32:])
                ))
                f.write(solicited_multicast_address +"@"+self.__mac_pairs[router.name+host.name]+"\n")

        f.close()

        #start router
        self.__router_process = router.popen(["./rust/router/target/debug/router", file_location], stdout=sys.stdout, stderr=sys.stdout,
                                             shell=True)
        return self.__router_process
