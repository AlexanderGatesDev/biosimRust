#!/usr/bin/gnuplot --persist

# Requires a text file named "epoch-log.txt" in the log directory
# Supports both old format (5 columns) and new format with species statistics (13 columns)
# Old format: generation survivors diversity avg_genome_length murder_count
# New format: generation survivors diversity avg_genome_length murder_count num_species avg_species_size largest_species smallest_species new_species extinct_species avg_species_age avg_stagnation

set term png size 2000, 400
set output "/biosimrust/images/log.png"

# Left Y axis gets scaled to the max survivors.
# Right Y axis gets scaled to 0..255.
#   1:2 Survivors 0..N        => 0..N
#   1:3 Diversity 0..1.0     => 0..255
#   1:4 Genome length 0..50   => 0..255
#   (Columns 6-13 are species statistics, available when speciation is enabled)

# Calculate max values from data for dynamic scaling
stats "/biosimrust/logs/epoch-log.txt" using 2 name "survivors" nooutput
stats "/biosimrust/logs/epoch-log.txt" using 3 name "diversity" nooutput
stats "/biosimrust/logs/epoch-log.txt" using 4 name "genome" nooutput

# Set Y-axis range with 10% padding above max survivors
max_survivors = survivors_max * 1.1
# Round up to next nice number (multiple of 50)
max_survivors = int(max_survivors / 50) * 50 + 50
if (max_survivors < 100) max_survivors = 100

# Set Y2-axis range for diversity (0-1) and genome length (scaled)
max_genome_scaled = genome_max * 2
max_y2 = (diversity_max > max_genome_scaled / 350) ? diversity_max : max_genome_scaled / 350
max_y2 = max_y2 * 1.1
if (max_y2 > 1.0) max_y2 = 1.0

set mxtics
set ytics autofreq nomirror tc lt 2
set yrange [ 0:max_survivors ]
set y2range [ 0:max_y2 ]
set y2tics autofreq nomirror tc lt 1
set grid
set key lmargin

ScaleGenomeLength(y)= y*2
ScaleDiversity(d)= 350*d

plot "/biosimrust/logs/epoch-log.txt" using 1:2 with lines lw 1 linecolor 2 title "Survivors", \
    "" using 1:(ScaleDiversity($3)) with lines lw 1 linecolor 1 title "Diversity" axes x1y2, \
    "" using 1:(ScaleGenomeLength($4)) with lines lw 1 linecolor 6 title "Genome Len" axes x1y2

