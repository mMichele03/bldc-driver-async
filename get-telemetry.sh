picotool save -r 0x10100000 0x10101380 telemetry.bin
python3 tools/parse_telemetry.py telemetry.bin telemetry_parsed.csv
