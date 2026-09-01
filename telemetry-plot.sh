# pkill gnuplot
gnuplot -p -e 'set datafile separator ","; plot "telemetry_parsed.csv" using 1:4 with lines, "" using 1:5 with lines'
gnuplot -p -e 'set datafile separator ","; plot "telemetry_parsed.csv" using 1:6 with lines'
gnuplot -p -e 'set datafile separator ","; plot "telemetry_parsed.csv" using 1:3 with lines'
