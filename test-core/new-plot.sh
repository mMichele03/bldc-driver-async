gnuplot -p -e 'set datafile separator ","; plot "output.csv" using 1:2 with lines, "" using 1:7 with lines'
gnuplot -p -e 'set datafile separator ","; plot "output.csv" using 1:4 with lines, "" using 1:5 with lines, "" using 1:6 with lines'
gnuplot -p -e 'set datafile separator ","; plot "output.csv" using 1:9 with lines, "" using 1:10 with lines, "" using 1:11 with lines'
