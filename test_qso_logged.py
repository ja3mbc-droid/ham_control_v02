import socket
import struct

def qstring(s):
    b = s.encode("utf-8")
    return struct.pack(">I", len(b)) + b

packet = b""

# WSJT-X magic
packet += struct.pack(">I", 0xadbccbda)

# schema
packet += struct.pack(">I", 2)

# message type 12 = QSO Logged
packet += struct.pack(">I", 12)

# id
packet += qstring("WSJT-X")

# date_time_off
packet += qstring("20260715_010000")

# dx_call
packet += qstring("JA3TEST")

# dx_grid
packet += qstring("PM74")

# tx_frequency
packet += struct.pack(">Q", 7100000)

# mode
packet += qstring("FreeDV")

# report sent
packet += qstring("+05")

# report received
packet += qstring("+05")

# comments
packet += qstring("ham_control test")

# name
packet += qstring("TEST")

# date_time_on
packet += qstring("20260715_005900")

# my_call
packet += qstring("JA3MBC")

# my_grid
packet += qstring("PM74")

s = socket.socket(socket.AF_INET, socket.SOCK_DGRAM)
s.sendto(packet, ("127.0.0.1",2237))

print("QSO Logged packet sent")
