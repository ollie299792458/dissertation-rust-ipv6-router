import sys
import time

from mininet.log import info, setLogLevel
from .test_framework import TestFramework

#    This file is part of Software IPv6 Router in Rust.
#
#    Software IPv6 Router in Rust is free software: you can redistribute it and/or modify
#    it under the terms of the GNU General Public License as published by
#    the Free Software Foundation, either version 3 of the License, or
#    (at your option) any later version.
#
#    Software IPv6 Router in Rust is distributed in the hope that it will be useful,
#    but WITHOUT ANY WARRANTY; without even the implied warranty of
#    MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
#    GNU General Public License for more details.
#
#    You should have received a copy of the GNU General Public License
#    along with Software IPv6 Router in Rust.  If not, see <https://www.gnu.org/licenses/>.
#
#    Copyright 2018,2019 Oliver Black

def run():
    test = TestFramework()

    left_address = 'fc00::1'
    right_address = 'fc00::2'
    router_address = 'fc00::3'

    default = test.addDefault('d0', 'fc00::', mac='ff:00:00:00:00:00')
    left = test.addIPv6Host('h1', left_address, mac='00:00:00:00:01:00')
    right = test.addIPv6Host('h2', right_address, mac='00:00:00:00:02:00')
    router = test.addRouter('r3', router_address, mac='00:00:00:00:03:00')

    test.addLink(default, router, addr2='00:00:00:00:03:00')
    test.addLink(left, router, addr1='00:00:00:00:01:00', addr2='00:00:00:00:03:01')
    test.addLink(right, router, addr1='00:00:00:00:02:00', addr2='00:00:00:00:03:02')

    test.start()
    info('Example test starting\n')

    #default.cmdPrint("ifconfig")

    routing = \
        "fe80::200:ff:fe00:100@00:00:00:00:03:01,00:00:00:00:01:00\n" + \
        "fe80::200:ff:fe00:200@00:00:00:00:03:02,00:00:00:00:02:00\n"

    router_process = test.runRouter(router, appended=routing)

    left.cmdPrint("ifconfig")
    right.cmdPrint("ifconfig")
    router.cmdPrint("ifconfig")



    time.sleep(1)

    server_process = left.popen([sys.executable, "-m", "python.luxing.my_simple_http_server", left_address, "8000"], stdout=sys.stdout, stderr=sys.stdout,
                                      shell=True)

    time.sleep(1)

    client_process = right.cmdPrint("curl --interface "+right_address+" -g -6 \"http://["+left_address+"]:8000/\"")
    time.sleep(1)

    info('Example test completed\n')
    server_process.kill()
    test.stop()


if __name__ == '__main__':
    setLogLevel('info')
    run()