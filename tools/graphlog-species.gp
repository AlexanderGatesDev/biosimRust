#!/usr/bin/gnuplot --persist

# Requires a text file named "epoch-log.txt" in the log directory
# Supports both old format (5 columns) and new format with species statistics (13 columns)
# If species statistics are not available, species plots will be empty

set term png size 2000, 600
set output "./images/log-species.png"

# Note: Run this script from the project root directory, not from tools/
# Usage: gnuplot tools/graphlog-species.gp

# Handle missing species statistics gracefully
set datafile missing "NaN"

# Calculate max values from data for dynamic scaling
stats "./logs/epoch-log.txt" using 2 name "survivors" nooutput
stats "./logs/epoch-log.txt" using 3 name "diversity" nooutput

# Set Y-axis range with 10% padding above max survivors
max_survivors = survivors_max * 1.1
max_survivors = int(max_survivors / 50) * 50 + 50
if (max_survivors < 100) max_survivors = 100

# Try to get species stats (may fail if column doesn't exist)
max_species = 50
stats "./logs/epoch-log.txt" using 6 name "species" nooutput
# Calculate max_species only if we have valid data
max_species = (species_records > 0 && species_max > 0) ? species_max * 1.2 : 50
max_species = int(max_species / 5) * 5 + 5
if (max_species < 10) max_species = 10

set mxtics
set ytics autofreq nomirror tc lt 2
set yrange [ 0:max_survivors ]
set y2range [ 0:max_species ]
set y2tics autofreq nomirror tc lt 1
set grid
set key lmargin

ScaleSurvivors(s) = s
ScaleDiversity(d) = d
ScaleSpeciesCount(s) = s

# Plot survivors, diversity, and species count (if available)
plot "./logs/epoch-log.txt" \
     using 1:(ScaleSurvivors($2)) with lines lw 2 linecolor 2 title "Survivors", \
  "" using 1:(ScaleDiversity($3)) with lines lw 2 linecolor 1 title "Diversity" axes x1y2, \
  "" using 1:(ScaleSpeciesCount($6)) with lines lw 2 linecolor 3 title "Species Count" axes x1y2
