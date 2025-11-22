use crate::structs::{Chart, Measure, Beat};
use crate::structs::SmFile;
use crate::utils::{parse_field, parse_pairs};
use std::path::PathBuf;

// StepMania row system constants
const ROWS_PER_BEAT: f64 = 48.0;  // 1 beat = 48 rows (for 4/4 time)
const ROWS_PER_MEASURE: f64 = 192.0;  // 1 measure = 192 rows (4 beats * 48)

impl SmFile {
    pub fn from_file(path: PathBuf) -> Result<SmFile, String> {
        let content = std::fs::read_to_string(path).map_err(|e| e.to_string())?;
        SmFile::parse(&content)
    }

    pub fn from_string(content: &str) -> Result<SmFile, String> {
        SmFile::parse(content)
    }

    fn parse(content: &str) -> Result<SmFile, String> {
        let mut sm = SmFile::new();
        sm.metadata.parse(content);
        sm.parse_bpms(content);
        sm.parse_stops(content);
        // Parse offset
        parse_field(content, r"#OFFSET:([-\d.]+);", &mut sm.offset);
        sm.offset = sm.offset.abs() * 1000.0;
        sm.parse_charts(content).map_err(|e| e.to_string())?;
        return Ok(sm);
    }

    fn parse_bpms(&mut self, content: &str) {
        // Parse BPM pairs from the file
        // Format: #BPMS:beat1=bpm1,beat2=bpm2,...;
        parse_pairs(content, r"(?s)#BPMS:(.*?);", &mut self.bpms);

        // Convert beats to rows (1 beat = 48 rows in StepMania)
        // Store as (row, bpm) instead of (beat, bpm)
        for (beat, _bpm) in &mut self.bpms {
            *beat = *beat * ROWS_PER_BEAT;
        }

        // Sort by row (first element of tuple)
        self.bpms
            .sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap_or(std::cmp::Ordering::Equal));

        // Ensure we have at least one BPM change at row 0
        if self.bpms.is_empty() || self.bpms[0].0 > 0.0 {
            self.bpms.insert(0, (0.0, 120.0));
        }
    }

    fn parse_stops(&mut self, content: &str) {
        // Parse stop pairs from the file
        // Format: #STOPS:beat1=duration1,beat2=duration2,...;
        parse_pairs(content, r"(?s)#STOPS:(.*?);", &mut self.stops);

        // Convert beats to rows (1 beat = 48 rows in StepMania)
        // Store as (row, duration) instead of (beat, duration)
        for (beat, _duration) in &mut self.stops {
            *beat = *beat * ROWS_PER_BEAT;
        }

        // Sort by row (first element of tuple)
        self.stops
            .sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap_or(std::cmp::Ordering::Equal));
    }

    fn parse_charts(&mut self, content: &str) -> Result<(), String> {
        let notes_sections: Vec<&str> = content.split("#NOTES:").skip(1).collect();

        for notes_section in notes_sections {
            // Find end of section (next #NOTES: or end)
            let section_end = notes_section.find("#NOTES:").unwrap_or(notes_section.len());
            let section_content = &notes_section[..section_end];

            let chart = Chart::parse(section_content, &self.bpms).map_err(|e| e.to_string())?;
            self.charts.push(chart);
        }
        Ok(())
    }
}

impl Chart {
    fn parse(content: &str, bpms: &[(f64, f64)]) -> Result<Chart, String> {
        let lines: Vec<&str> = content.lines().map(|l| l.trim()).collect();

        let mut chart = Chart::new();
    
        // Parse chart header
        let mut idx = chart.parse_header(&lines);
    
        // Parse measures
        // Timing state - using row-based system
        let mut current_bpm = if bpms.is_empty() { 120.0 } else { bpms[0].1 };
        let mut current_time_ms = 0.0; // Time in MILLISECONDS
        let mut current_row = 0.0; // Position in rows (not beats!)
        let mut bpm_index = 0;

        while idx < lines.len() {
            // Parse next measure (it will handle BPM changes internally)
            let (measure, next_idx, new_time_ms, new_row) = Measure::parse(
                &lines, 
                idx, 
                bpms,
                &mut current_bpm,
                &mut bpm_index,
                current_time_ms, 
                current_row
            );
            
            // Always add measure, even if empty (empty measures represent time)
            chart.measures.push(measure);

            current_time_ms = new_time_ms;
            current_row = new_row;
            idx = next_idx;

            // Check if we hit a semicolon (end of chart)
            if idx > 0 && idx <= lines.len() {
                let prev_line = lines[idx - 1].trim();
                let line_without_comment = if let Some(comment_pos) = prev_line.find("//") {
                    &prev_line[..comment_pos]
                } else {
                    prev_line
                }
                .trim();
                
                if line_without_comment == ";" {
                    break;
                }
            }

            // If we're at EOF, stop
            if idx >= lines.len() {
                break;
            }
        }
    
        Ok(chart)
    }

    fn parse_header(&mut self, lines: &[&str]) -> usize {
        let mut idx = 0;
    
        // Skip empty lines
        while idx < lines.len() && lines[idx].is_empty() {
            idx += 1;
        }
    
        // Stepstype
        if idx < lines.len() {
            self.stepstype = lines[idx].to_string();
            idx += 1;
        }
    
        // Skip description (empty or ":")
        while idx < lines.len() && (lines[idx].is_empty() || lines[idx] == ":") {
            idx += 1;
        }
    
        // Difficulty
        if idx < lines.len() {
            self.difficulty = lines[idx].to_string();
            idx += 1;
        }
    
        // Skip empty lines
        while idx < lines.len() && lines[idx].is_empty() {
            idx += 1;
        }
    
        // Meter
        if idx < lines.len() {
            self.meter = lines[idx].parse().unwrap_or(0);
            idx += 1;
        }
    
        // Skip empty lines
        while idx < lines.len() && lines[idx].is_empty() {
            idx += 1;
        }
    
        // Radar values
        if idx < lines.len() {
            for val in lines[idx].split(',') {
                if let Ok(v) = val.trim().parse::<f64>() {
                    self.radar_values.push(v);
                }
            }
            idx += 1;
        }
    
        idx
    }
}

impl Measure {
    fn parse(
        lines: &[&str], 
        start_idx: usize, 
        bpms: &[(f64, f64)],  // (row, bpm) pairs
        current_bpm: &mut f64,
        bpm_index: &mut usize,
        start_time_ms: f64, 
        start_row: f64,
    ) -> (Measure, usize, f64, f64) {
        let mut measure = Measure::new();
        let mut idx = start_idx;

        // Parse lines until we hit a comma or semicolon
        let mut note_lines = Vec::new();
        while idx < lines.len() {
            let line = lines[idx].trim();

            if line.is_empty() {
                idx += 1;
                continue;
            }

            // Remove comments from line
            let line_without_comment = if let Some(comment_pos) = line.find("//") {
                &line[..comment_pos]
            } else {
                line
            }
            .trim();

            // Check if line is a measure separator (comma or semicolon)
            if line_without_comment == "," || line_without_comment == ";" {
                break;
            } else if Beat::is_note_line(line_without_comment) {
                // Store note lines for later processing
                note_lines.push(line_without_comment);
            }

            idx += 1;
        }

        // Calculate quantization (rows per note line)
        // A measure has 192 rows total
        let num_lines = note_lines.len();
        let quantization = if num_lines > 0 {
            // Determine quantization by checking if measure uses full 192 rows
            if num_lines == ROWS_PER_MEASURE as usize {
                // Full 192-row measure - check for custom quantization
                // Try to find the smallest valid quantization
                let mut found_quant = ROWS_PER_MEASURE as usize;
                for test_quant in [4, 8, 12, 16, 24, 32, 48, 64, 96] {
                    if ROWS_PER_MEASURE as usize % test_quant == 0 {
                        // For now, accept the first valid quantization
                        // A more sophisticated check would verify that compressed rows are empty
                        found_quant = test_quant;
                        break;
                    }
                }
                found_quant
            } else if num_lines > 0 {
                // Calculate quantization from number of lines
                ROWS_PER_MEASURE as usize / num_lines
            } else {
                ROWS_PER_MEASURE as usize
            }
        } else {
            ROWS_PER_MEASURE as usize
        };

        // Process note lines and calculate timings
        measure.start_time = start_time_ms;
        let mut current_time = start_time_ms;
        let mut current_row = start_row;

        for (line_idx, line) in note_lines.iter().enumerate() {
            // Calculate row position for this note line
            let row_offset = if num_lines > 0 {
                // Handle quantization
                if ROWS_PER_MEASURE as usize % num_lines == 0 {
                    (line_idx * quantization) as f64
                } else {
                    // Non-uniform spacing
                    (ROWS_PER_MEASURE as f64 / num_lines as f64) * line_idx as f64
                }
            } else {
                0.0
            };

            let note_row = start_row + row_offset;

            // Check for BPM changes up to this row
            while *bpm_index < bpms.len() {
                let (bpm_row, new_bpm) = bpms[*bpm_index];
                if bpm_row <= note_row {
                    // Calculate time elapsed from previous position to this BPM change
                    if bpm_row > current_row {
                        let rows_elapsed = bpm_row - current_row;
                        let beats_elapsed = rows_elapsed / ROWS_PER_BEAT;
                        let time_elapsed_ms = (beats_elapsed / *current_bpm) * 60000.0;
                        current_time += time_elapsed_ms;
                        current_row = bpm_row;
                    }
                    *current_bpm = new_bpm;
                    *bpm_index += 1;
                } else {
                    break;
                }
            }

            // Calculate time for this note line
            if note_row > current_row {
                let rows_elapsed = note_row - current_row;
                let beats_elapsed = rows_elapsed / ROWS_PER_BEAT;
                let time_elapsed_ms = (beats_elapsed / *current_bpm) * 60000.0;
                current_time += time_elapsed_ms;
                current_row = note_row;
            }

            // Parse and store the beat
            let mut beat = Beat::parse(line);
            beat.time = current_time;
            measure.beats.push(beat);
        }

        // Calculate final row and time for the end of the measure
        let end_row = start_row + ROWS_PER_MEASURE;
        
        // Check for any remaining BPM changes before end of measure
        while *bpm_index < bpms.len() {
            let (bpm_row, new_bpm) = bpms[*bpm_index];
            if bpm_row < end_row {
                // Calculate time elapsed to this BPM change
                if bpm_row > current_row {
                    let rows_elapsed = bpm_row - current_row;
                    let beats_elapsed = rows_elapsed / ROWS_PER_BEAT;
                    let time_elapsed_ms = (beats_elapsed / *current_bpm) * 60000.0;
                    current_time += time_elapsed_ms;
                    current_row = bpm_row;
                }
                *current_bpm = new_bpm;
                *bpm_index += 1;
            } else {
                break;
            }
        }

        // Calculate time from current position to end of measure
        if end_row > current_row {
            let rows_elapsed = end_row - current_row;
            let beats_elapsed = rows_elapsed / ROWS_PER_BEAT;
            let time_elapsed_ms = (beats_elapsed / *current_bpm) * 60000.0;
            current_time += time_elapsed_ms;
        }

        let next_idx = if idx < lines.len() && (lines[idx].trim() == "," || lines[idx].trim() == ";") {
            idx + 1
        } else {
            idx
        };

        (measure, next_idx, current_time, end_row)
    }
}

impl Beat {
    pub fn is_note_line(line: &str) -> bool {
        line.chars()
            .all(|c| matches!(c, '0' | '1' | '2' | '3' | '4' | 'M'))
    }

    pub fn parse(line: &str) -> Beat {
        let notes = line.chars().map(|c| c != '0').collect();
        Beat {
            time: 0.0, // Will be calculated when measure ends
            notes,
        }
    }
}

