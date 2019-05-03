import socket
import sys
from BaseHTTPServer import HTTPServer
from SimpleHTTPServer import SimpleHTTPRequestHandler

#from https://gist.github.com/akorobov/7903307

class MyHandler(SimpleHTTPRequestHandler):
    def do_GET(self):
        if self.path == '/ip':
            self.send_response(200)
            self.send_header('Content-type', 'text/html')
            self.end_headers()
            self.wfile.write('Your IP address is %s' % self.client_address[0])
            return
        else:
            return SimpleHTTPRequestHandler.do_GET(self)

class HTTPServerV6(HTTPServer):
    address_family = socket.AF_INET6

def main():
    server = HTTPServerV6((sys.argv[1], int(sys.argv[2])), MyHandler)
    print "Starting server on:" +sys.argv[1]+", port:"+sys.argv[2]
    server.serve_forever()

if __name__ == '__main__':
    main()