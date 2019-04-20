import socket
import sys
from logging import info


s = socket.socket(socket.AF_INET6, socket.SOCK_DGRAM)
s.bind((sys.argv[1],0))

print "Server Running"

while True:
    data, addr = s.recvfrom(512)
    print ("%s: %s\n" % (addr, data))