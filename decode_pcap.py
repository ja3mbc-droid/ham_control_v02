#!/usr/bin/env python3
import struct, sys

def read_pcap_udp_payloads(path):
    with open(path, "rb") as f:
        data = f.read()
    magic = data[0:4]
    if magic == b'\xa1\xb2\xc3\xd4':
        endian = ">"
    elif magic == b'\xd4\xc3\xb2\xa1':
        endian = "<"
    else:
        raise ValueError(f"unknown pcap magic: {magic.hex()}")
    offset = 24  # global header
    payloads = []
    while offset < len(data):
        ts_sec, ts_usec, incl_len, orig_len = struct.unpack_from(endian+"IIII", data, offset)
        offset += 16
        pkt = data[offset:offset+incl_len]
        offset += incl_len
        # Ethernet(14) + IP header(variable) + UDP header(8)
        if len(pkt) < 14: continue
        eth_type = struct.unpack_from(">H", pkt, 12)[0]
        ip_off = 14
        if eth_type != 0x0800: continue  # not IPv4
        ihl = (pkt[ip_off] & 0x0F) * 4
        proto = pkt[ip_off+9]
        if proto != 17: continue  # not UDP
        udp_off = ip_off + ihl
        udp_payload = pkt[udp_off+8:]
        payloads.append((ts_sec, ts_usec, udp_payload))
    return payloads

def read_qstring(buf, off):
    ln = struct.unpack_from(">I", buf, off)[0]
    off += 4
    if ln == 0xFFFFFFFF:  # null string
        return "", off
    s = buf[off:off+ln].decode("utf-8", errors="replace")
    return s, off+ln

def decode_message(buf):
    off = 0
    magic = struct.unpack_from(">I", buf, off)[0]; off += 4
    schema = struct.unpack_from(">I", buf, off)[0]; off += 4
    msg_type = struct.unpack_from(">I", buf, off)[0]; off += 4
    client_id, off = read_qstring(buf, off)
    print(f"  magic=0x{magic:08x} schema={schema} type={msg_type} id={client_id!r} total_len={len(buf)}")
    if msg_type == 0:
        # Heartbeat: quint32 maxSchema, QString version, QString revision
        max_schema = struct.unpack_from(">I", buf, off)[0]; off += 4
        version, off = read_qstring(buf, off)
        revision, off = read_qstring(buf, off)
        print(f"    [Heartbeat] max_schema={max_schema} version={version!r} revision={revision!r}")
    elif msg_type == 5:
        # QSOLogged: QDateTime DateTimeOff, QString DXCall, QString DXGrid,
        # quint64 TXFrequency, QString Mode, QString ReportSent, QString ReportReceived,
        # QString TXPower, QString Comments, QString Name, QDateTime DateTimeOn,
        # QString OperatorCall, QString MyCall, QString MyGrid, ...
        def read_qdatetime(buf, off):
            jd = struct.unpack_from(">Q", buf, off)[0]; off += 8
            msec = struct.unpack_from(">I", buf, off)[0]; off += 4
            timespec = buf[off]; off += 1
            if timespec == 2:  # OffsetFromUTC
                off += 4
            return (jd, msec), off
        dt_off, off = read_qdatetime(buf, off)
        dx_call, off = read_qstring(buf, off)
        dx_grid, off = read_qstring(buf, off)
        tx_freq = struct.unpack_from(">Q", buf, off)[0]; off += 8
        mode, off = read_qstring(buf, off)
        report_sent, off = read_qstring(buf, off)
        report_recv, off = read_qstring(buf, off)
        tx_power, off = read_qstring(buf, off)
        comments, off = read_qstring(buf, off)
        name, off = read_qstring(buf, off)
        dt_on, off = read_qdatetime(buf, off)
        op_call, off = read_qstring(buf, off)
        my_call, off = read_qstring(buf, off)
        my_grid, off = read_qstring(buf, off)
        print(f"    [QSOLogged] dx_call={dx_call!r} dx_grid={dx_grid!r} freq={tx_freq} mode={mode!r}")
        print(f"                report_sent={report_sent!r} report_recv={report_recv!r} tx_power={tx_power!r}")
        print(f"                comments={comments!r} name={name!r} op_call={op_call!r} my_call={my_call!r} my_grid={my_grid!r}")
        print(f"                bytes_consumed={off} / total={len(buf)}")
    else:
        print(f"    [type {msg_type}] not decoded in detail, remaining bytes: {buf[off:off+40].hex()}")

if __name__ == "__main__":
    path = sys.argv[1] if len(sys.argv) > 1 else "freedv_qso_logged.pcap"
    payloads = read_pcap_udp_payloads(path)
    print(f"{len(payloads)} UDP packets found in {path}")
    for i, (sec, usec, buf) in enumerate(payloads):
        print(f"[{i}] len={len(buf)}")
        try:
            decode_message(buf)
        except Exception as e:
            print(f"  decode error: {e}")
