# pkill gnuplot
gnuplot -p -e 'set datafile separator ","; plot "output.csv" using 1:6 with lines, "" using 1:8 with lines'
gnuplot -p -e 'set datafile separator ","; plot "output.csv" using 1:2 with lines, "" using 1:4 with lines'
gnuplot -p -e 'set datafile separator ","; plot "output.csv" using 1:10 with lines, "" using 1:11 with lines, "" using 1:12 with lines'
