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

    left_host = test.addHost('h1')
    right_host = test.addHost('h2')

    test.addLink(left_host, right_host)

    test.start()
    info('Example test starting\n')

    test.ping6()
    # ran twice to allow correct discovery
    test.ping6()
    info('Example test completed\n')
    test.stop()


if __name__ == '__main__':
    setLogLevel('info')
    run()
