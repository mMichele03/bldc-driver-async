sudo picotool save -r 0x10100000 0x101249F0 telemetry.bin
sudo python3 tools/parse_telemetry.py telemetry.bin telemetry_parsed.csv
./telemetry-plot.sh
