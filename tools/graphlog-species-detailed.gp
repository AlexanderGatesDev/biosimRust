#!/usr/bin/gnuplot --persist

# Detailed species statistics visualization
# Requires epoch-log.txt with species statistics (13 columns)
# Plots multiple species metrics on different axes

set term png size 2400, 800
set output "./images/log-species-detailed.png"

# Note: Run this script from the project root directory, not from tools/
# Usage: gnuplot tools/graphlog-species-detailed.gp

set datafile missing "NaN"

# Calculate max values from data for dynamic scaling
stats "./logs/epoch-log.txt" using 2 name "survivors" nooutput
stats "./logs/epoch-log.txt" using 6 name "species" nooutput
stats "./logs/epoch-log.txt" using 8 name "size_max" nooutput
stats "./logs/epoch-log.txt" using 10 name "new_species" nooutput
stats "./logs/epoch-log.txt" using 11 name "extinct_species" nooutput
stats "./logs/epoch-log.txt" using 12 name "age" nooutput
stats "./logs/epoch-log.txt" using 13 name "stagnation" nooutput

# Calculate dynamic ranges with padding
max_survivors = survivors_max * 1.1
max_survivors = int(max_survivors / 50) * 50 + 50
if (max_survivors < 100) max_survivors = 100

max_species = 50
max_species = (species_records > 0 && species_max > 0) ? species_max * 1.2 : 50
max_species = int(max_species / 5) * 5 + 5
if (max_species < 10) max_species = 10

max_size = 50
max_size = (size_max_records > 0 && size_max_max > 0) ? size_max_max * 1.2 : 50
max_size = int(max_size / 5) * 5 + 5
if (max_size < 10) max_size = 10

max_turnover = 5
new_max = (new_species_records > 0) ? new_species_max : 0
extinct_max = (extinct_species_records > 0) ? extinct_species_max : 0
max_turnover = (new_max > extinct_max) ? new_max : extinct_max
max_turnover = max_turnover * 1.2
max_turnover = int(max_turnover) + 1
if (max_turnover < 5) max_turnover = 5

max_age = 200
max_age = (age_records > 0 && age_max > 0) ? age_max * 1.2 : 200
max_age = int(max_age / 10) * 10 + 10
if (max_age < 50) max_age = 50

max_stagnation = 20
max_stagnation = (stagnation_records > 0 && stagnation_max > 0) ? stagnation_max * 1.2 : 20
max_stagnation = int(max_stagnation) + 1
if (max_stagnation < 5) max_stagnation = 5

set multiplot layout 2,2 title "Species Statistics Over Time"

# Top left: Species count and survivors
set title "Population and Species Count"
set ylabel "Count" tc lt 2
set y2label "Species" tc lt 3
set yrange [0:max_survivors]
set y2range [0:max_species]
set y2tics autofreq nomirror tc lt 3
set grid
plot "./logs/epoch-log.txt" \
     using 1:2 with lines lw 2 linecolor 2 title "Survivors", \
  "" using 1:6 with lines lw 2 linecolor 3 title "Species Count" axes x1y2

# Top right: Species size distribution
set title "Species Size Distribution"
set ylabel "Size"
set y2label ""
unset y2tics
set yrange [0:max_size]
plot "./logs/epoch-log.txt" \
     using 1:7 with lines lw 2 linecolor 4 title "Avg Species Size", \
  "" using 1:8 with lines lw 1 linecolor 5 title "Largest Species", \
  "" using 1:9 with lines lw 1 linecolor 6 title "Smallest Species"

# Bottom left: Species turnover
set title "Species Creation and Extinction"
set ylabel "Count"
set yrange [0:max_turnover]
plot "./logs/epoch-log.txt" \
     using 1:10 with lines lw 2 linecolor 7 title "New Species", \
  "" using 1:11 with lines lw 2 linecolor 8 title "Extinct Species"

# Bottom right: Species age and stagnation
set title "Species Age and Stagnation"
set ylabel "Generations"
set y2label "Stagnation" tc lt 9
set y2range [0:max_stagnation]
set y2tics autofreq nomirror tc lt 9
set yrange [0:max_age]
plot "./logs/epoch-log.txt" \
     using 1:12 with lines lw 2 linecolor 9 title "Avg Species Age", \
  "" using 1:13 with lines lw 2 linecolor 10 title "Avg Stagnation" axes x1y2

unset multiplot
