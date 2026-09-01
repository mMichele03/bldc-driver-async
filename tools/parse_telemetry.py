#!/usr/bin/env python3
import struct
import sys
import math

INFILE = sys.argv[1] if len(sys.argv) > 1 else 'telemetry.bin'
OUTFILE = sys.argv[2] if len(sys.argv) > 2 else 'telemetry_parsed.csv'
RECORD_SIZE = 32  # bytes: u64, i32, i32, i32, u32
ENCODER_BITS = 14
DEG_PRECISION = 2

def angle_to_deg(a):
    return round(a * 360 / pow(2, ENCODER_BITS), DEG_PRECISION)

print(f'Reading {INFILE} -> {OUTFILE}')
with open(INFILE, 'rb') as f:
    data = f.read()

rows = []
for i in range(0, len(data) - RECORD_SIZE + 1, RECORD_SIZE):
    chunk = data[i:i+RECORD_SIZE]
    try:
        ts, tenc, test, angle, aest, vest, pad = struct.unpack('<QIIiiiI', chunk)
    except struct.error:
        continue
    if ts > 1000000000 or ts == 0:
        break
    rows.append((ts, tenc, test, angle_to_deg(angle), angle_to_deg(aest), vest))

with open(OUTFILE, 'w') as out:
    out.write('ts,enc_ts,est_ts,angle,angle_est,velocity_est\n')
    for ts, tenc, test, angle, aest, vest in rows:
        out.write(f'{ts},{tenc},{test},{angle},{aest},{vest}\n')

print(f'Wrote {len(rows)} rows to {OUTFILE}')
