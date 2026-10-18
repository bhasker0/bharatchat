#!/usr/bin/env python3
import http.server
import socketserver
import sys
import os

class NoCacheHTTPRequestHandler(http.server.SimpleHTTPRequestHandler):
    def end_headers(self):
        self.send_header('Cache-Control', 'no-cache, no-store, must-revalidate')
        self.send_header('Pragma', 'no-cache')
        self.send_header('Expires', '0')
        self.send_header('Access-Control-Allow-Origin', '*')
        super().end_headers()

def run(port=3000, directory=None):
    if directory:
        os.chdir(directory)
    handler = NoCacheHTTPRequestHandler
    socketserver.TCPServer.allow_reuse_address = True
    with socketserver.TCPServer(("", port), handler) as httpd:
        print(f"Serving HTTP on 0.0.0.0 port {port} with No-Cache headers from {directory or os.getcwd()} ...", flush=True)
        httpd.serve_forever()

if __name__ == '__main__':
    port = int(sys.argv[1]) if len(sys.argv) > 1 else 3000
    dir_path = sys.argv[2] if len(sys.argv) > 2 else '.'
    run(port, dir_path)
