import random
import socket
import string
import subprocess
import sys
from logging import info
from struct import pack

# helper methods
from pip._vendor.ipaddress import IPv6Address


def getIPv6Header(version = 6,
                  traffic_class = 0, #todo split into dcn and non dcn
                  flow_label = 0,
                  payload_length = 16,
                  next_header = 58, # ICMPv6 by default
                  hop_limit = 255,
                  source_address = "::",
                  destination_address = "::"
        ):

    source_address = IPv6Address(u''+source_address)
    destination_address = IPv6Address(u''+destination_address)

    source_address_first = int(source_address.exploded[0:10].replace(':',''),16)
    destination_address_first = int(destination_address.exploded[0:10].replace(':',''),16)

    source_address_second = int(source_address.exploded[10:].replace(':',''))
    destination_address_second = int(destination_address.exploded[10:].replace(':',''))

    version_traffic_class_flow_label = (version << 28) + (traffic_class << 20) + flow_label

    ip_header = pack('!LHBBQQQQ',version_traffic_class_flow_label,payload_length,next_header,hop_limit,
                     source_address_first, source_address_second,destination_address_first, destination_address_second)
    return ip_header

# different test packets


def get11211packet(source_address, destination_address):
    payload_length = 0
    ip_header = getIPv6Header(payload_length=payload_length, source_address=source_address, destination_address=destination_address)
    data = ''.join(random.choice(string.ascii_lowercase) for x in range(payload_length))
    packet = ip_header + data
    return packet, destination_address

# main code
switcher = {
    '11211': get11211packet,
}

s = socket.socket(socket.AF_INET6, socket.SOCK_RAW, socket.IPPROTO_RAW)

# s.setsockopt(socket.IPPROTO_IP, socket.IP_HDRINCL, 1)

source_ip = sys.argv[2]
(packet, dest_ip) = switcher.get(sys.argv[1])(source_ip,sys.argv[3])

s.bind((source_ip,0))

print "Client sending packet"

s.sendto(packet, (dest_ip, 0))

print "Client packet sent"

print subprocess.check_output(['ifconfig'])

s = socket.socket(socket.AF_INET6, socket.SOCK_DGRAM)
s.bind((source_ip, 0))

print source_ip + " " + dest_ip

s.sendto("hiya", (dest_ip, 0))

print "test complete"