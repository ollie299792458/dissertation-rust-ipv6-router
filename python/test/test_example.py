from mininet.log import info, setLogLevel
from .test_framework import TestFramework


def run():
    test = TestFramework()

    left_host = test.addHost('h1')
    right_host = test.addHost('h2')
    #switch = test.addSwitch('s3')

    test.addLink(left_host, right_host)
    #test.addLink(switch, right_host)

    test.start()
    info('Example test starting\n')

    host = test.get('h1')
    host2 = test.get('h2')

    host.cmdPrint("ifconfig")
    host2.cmdPrint("ifconfig")
    print(test.address('h1'))
    test.ping6()
    test.ping6()
    #test.ping()
    host.cmdPrint("ping6 -W 10 " + test.address(str(host2)))
    host.cmdPrint("route -6 -n")
    #host.cmdPrint("ip -6 neighbor show")
    info('Example test completed\n')
    test.stop()


if __name__ == '__main__':
    setLogLevel('info')
    run()
