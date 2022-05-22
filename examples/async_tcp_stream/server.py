import socketserver
from time import sleep

class MyUDPHandler(socketserver.StreamRequestHandler):
    def handle(self):
        print("Got an UDP Message from {}".format(self.client_address[0]))
        sleep(1)
        msgRecvd = self.rfile.readline().strip()
        print("The Message is {}".format(msgRecvd))
        self.wfile.write("Hello UDP Client! I received a message from you!".encode())

PORT = 1235
with socketserver.TCPServer(("", PORT), MyUDPHandler) as httpd:
    print("serving at port", PORT)
    httpd.serve_forever()