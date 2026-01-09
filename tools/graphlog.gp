#!/usr/bin/gnuplot --persist

# Requires a text file named "epoch-log.txt" in the log directory
# Supports both old format (5 columns) and new format with species statistics (13 columns)
# Old format: generation survivors diversity avg_genome_length murder_count
# New format: generation survivors diversity avg_genome_length murder_count num_species avg_species_size largest_species smallest_species new_species extinct_species avg_species_age avg_stagnation

set term png size 2000, 400
set output "./images/log.png"

# Left Y axis gets scaled to the max survivors.
# Right Y axis gets scaled to 0..255.
#   1:2 Survivors 0..N        => 0..N
#   1:3 Genome length 0..50   => 0..255
#   1:4 Diversity 0..0.7      => 0..255
#   1:5 Murder count 0..N     => 0..N
#   (Columns 6-13 are species statistics, available when speciation is enabled)

# Calculate max values from data for dynamic scaling
stats "./logs/epoch-log.txt" using 2 name "survivors" nooutput
stats "./logs/epoch-log.txt" using 3 name "diversity" nooutput

# Set Y-axis range with 10% padding above max survivors
max_survivors = survivors_max * 1.1
# Round up to next nice number (multiple of 50)
max_survivors = int(max_survivors / 50) * 50 + 50
if (max_survivors < 100) max_survivors = 100

set mxtics
set ytics autofreq nomirror tc lt 2
set yrange [ 0:max_survivors ]
set y2range [ 0:1 ]
set y2tics autofreq nomirror tc lt 1
set grid
set key lmargin

ScaleSurvivors(s) = s
ScaleGenomeLength(y)= y*2
ScaleDiversity(d)= d
#ScaleMurders(m) = m

plot "./logs/epoch-log.txt" \
       using 1:(ScaleSurvivors($2)) with lines lw 2 linecolor 2 title "Survivors", \
    "" using 1:(ScaleDiversity($3)) with lines lw 2 linecolor 1 title "Diversity" axes x1y2

