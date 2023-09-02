import socket
import struct
import argparse
import time

def make_message(address, data):
    address_bytes = address.to_bytes(2, byteorder='big')
    data_length = len(data)
    data_length_bytes = data_length.to_bytes(2, byteorder='big')
    message = address_bytes + data_length_bytes + data.encode('utf-8')
    return message

def send_message(host, port, message):
    with socket.socket(socket.AF_INET, socket.SOCK_STREAM) as s:
        s.connect((host, port))
        s.sendall(message)
        time.sleep(3)
        print("Message sent successfully.")

def main():
    parser = argparse.ArgumentParser(description='Send a message via TCP socket.')
    parser.add_argument('address', type=int, help='Message address (16-bit)')
    parser.add_argument('data', type=str, help='Message data (UTF-8 string)')
    parser.add_argument('--host', default='localhost', help='Destination host')
    parser.add_argument('--port', type=int, default=8000, help='Destination port')
    args = parser.parse_args()

    message = make_message(args.address, args.data)
    send_message(args.host, args.port, message)

if __name__ == '__main__':
    main()

