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

    default = test.addDefault('d0', 'fc00::', mac='ff:00:00:00:00:00')
    left = test.addIPv6Host('h1', 'fc00::1', mac='00:00:00:00:01:00')
    right = test.addIPv6Host('h2', 'fc00::2', mac='00:00:00:00:02:00')
    router = test.addRouter('r3', 'fc00::3', mac='00:00:00:00:03:00')

    test.addLink(default, router, addr2='00:00:00:00:03:00')
    test.addLink(left, router, addr1='00:00:00:00:01:00', addr2='00:00:00:00:03:01')
    test.addLink(right, router, addr1='00:00:00:00:02:00', addr2='00:00:00:00:03:02')

    test.start()
    info('1112 Static routing test starting\n')

    router_process = test.runRouter(router)
    time.sleep(1)
    ploss = test.ping6()

    state = 'Successful'

    if ploss != 0:
        state = "FAILED"

    # todo move from ping6 to direct packet sending, to avoid failure due to hop_limit decrement
    info('Static routing test completed:'+state+'\n')
    test.stop()


if __name__ == '__main__':
    setLogLevel('info')
    run()