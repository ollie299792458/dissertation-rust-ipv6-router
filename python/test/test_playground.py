import sys
import time

from mininet.log import info, setLogLevel
from .test_framework import TestFramework


def run():
    test = TestFramework()

    left_address = 'fc00::1'
    right_address = 'fc00::2'

    default = test.addDefault('d0', 'fc00::', mac='ff:00:00:00:00:00')
    left = test.addIPv6Host('h1', left_address, mac='00:00:00:00:01:00')
    right = test.addIPv6Host('h2', right_address, mac='00:00:00:00:02:00')
    router = test.addRouter('r3', 'fc00::3', mac='00:00:00:00:03:00')

    test.addLink(default, router, addr2='00:00:00:00:03:00')
    test.addLink(left, router, addr1='00:00:00:00:01:00', addr2='00:00:00:00:03:01')
    test.addLink(right, router, addr1='00:00:00:00:02:00', addr2='00:00:00:00:03:02')

    test.start()
    info('Example test starting\n')

    #default.cmdPrint("ifconfig")

    router_process = test.runRouter(router)

    server_process = right.popen(["./rust/test_server/target/debug/test_server", "h2-eth0"], stdout=sys.stdout, stderr=sys.stdout,
                                       shell=True)

    time.sleep(1)

    client_process = left.popen(["./rust/test_client/target/debug/test_client", "h1-eth0", "11212",
                                 '00:00:00:00:01:00',left_address, right_address], stdout=sys.stdout, stderr=sys.stdout,
                                shell=True)


    time.sleep(1)

    #client_process = right.popen([sys.executable,"./python/test/testing_tools/test_client.py", '11211', right_address,
     #                             left_address], stdout=sys.stdout, stderr=sys.stdout, shell=True)
    #time.sleep(1)

    info('Example test completed\n')
    server_process.kill()
    test.stop()


if __name__ == '__main__':
    setLogLevel('info')
    run()