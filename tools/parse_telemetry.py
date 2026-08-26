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
        ts, est, meas, speed, t1, t2, pad = struct.unpack('<QiiiIII', chunk)
    except struct.error:
        continue
    rows.append((ts, angle_to_deg(est), angle_to_deg(meas), speed, t1, t2))

with open(OUTFILE, 'w') as out:
    out.write('ts,est_raw,meas_raw,speed,t1,t2\n')
    for ts, est, meas, speed, t1, t2 in rows:
        out.write(f'{ts},{est},{meas},{speed},{t1},{t2}\n')

print(f'Wrote {len(rows)} rows to {OUTFILE}')
