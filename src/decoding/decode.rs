use crate::structs::{Chart, Measure, Beat};
use crate::structs::SmFile;
use crate::utils::{parse_field, parse_pairs};
use std::path::PathBuf;

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
        // 1. Utilisation de la fonction générique pour remplir le vecteur
        parse_pairs(content, r"(?s)#BPMS:(.*?);", &mut self.bpms);

        // 2. Tri (Sort) par beat (le premier élément du tuple)
        self.bpms
            .sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap_or(std::cmp::Ordering::Equal));

        // 3. Gestion de la valeur par défaut si vide
        if self.bpms.is_empty() {
            self.bpms.push((0.0, 120.0));
        }
    }

    fn parse_stops(&mut self, content: &str) {
        parse_pairs(content, r"(?s)#STOPS:(.*?);", &mut self.stops);

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
        // Timing state
        let mut current_bpm = if bpms.is_empty() { 120.0 } else { bpms[0].1 };
        let mut current_time_ms = 0.0; // Time in MILLISECONDS
        let mut current_beat = 0.0; // Position in beats
        let mut bpm_index = 0;

        while idx < lines.len() {
            // Check if BPM changes before this measure
            update_bpm_if_needed(&mut current_bpm, current_beat, &mut bpm_index, bpms);

            // Parse next measure (it will calculate timings internally)
            let (measure, next_idx, new_time_ms, new_beat) = Measure::parse(
                &lines, 
                idx, 
                current_bpm, 
                current_time_ms, 
                current_beat
            );
            
            // Always add measure, even if empty (empty measures represent time)
            chart.measures.push(measure);

            current_time_ms = new_time_ms;
            current_beat = new_beat;
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
        bpm: f64, 
        start_time_ms: f64, 
        start_beat: f64,
    ) -> (Measure, usize, f64, f64) {
        let mut measure = Measure::new();
        let mut idx = start_idx;

        // Parse lines until we hit a comma or semicolon
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
                // End of measure - calculate timings for all beats
                let beats_in_measure = measure.beats.len();
                
                // If measure is empty, assume it has 4 beats (standard measure length)
                let actual_beats = if beats_in_measure == 0 { 4 } else { beats_in_measure };
                
                measure.start_time = start_time_ms;
                
                // Calculate time per beat in MILLISECONDS
                // A measure always represents 4 beats of music in StepMania
                // Formula: time_per_beat_ms = (60000 / BPM * 4) / beats_in_measure
                // This divides the total measure time (4 beats of music) by the number of lines
                let beats_in_measure_f64 = actual_beats as f64;
                let measure_duration_ms = (60000.0 / bpm) * 4.0; // 4 beats of music per measure
                let time_per_beat_ms = measure_duration_ms / beats_in_measure_f64;

                // Update each beat with timing (if any)
                let mut current_time = start_time_ms;
                let mut current_beat = start_beat;
                for beat in measure.beats.iter_mut() {
                    beat.time = current_time;
                    current_time += time_per_beat_ms;
                    current_beat += 1.0;
                }
                
                // Advance time even if measure is empty (empty measures still take time)
                let new_time = start_time_ms + (time_per_beat_ms * actual_beats as f64);
                let new_beat = start_beat + actual_beats as f64;

                return (measure, idx + 1, new_time, new_beat);
            } else if Beat::is_note_line(line_without_comment) {
                // Parse note line using Beat::parse()
                let beat = Beat::parse(line_without_comment);
                measure.beats.push(beat);
            }

            idx += 1;
        }

        // End of file - calculate timings for all beats
        let beats_in_measure = measure.beats.len();
        
        // If measure is empty, assume it has 4 beats (standard measure length)
        let actual_beats = if beats_in_measure == 0 { 4 } else { beats_in_measure };
        
        measure.start_time = start_time_ms;
        
        // Calculate time per beat in MILLISECONDS
        // A measure always represents 4 beats of music in StepMania
        // Formula: time_per_beat_ms = (60000 / BPM * 4) / beats_in_measure
        let beats_in_measure_f64 = actual_beats as f64;
        let measure_duration_ms = (60000.0 / bpm) * 4.0; // 4 beats of music per measure
        let time_per_beat_ms = measure_duration_ms / beats_in_measure_f64;

        // Update each beat with timing (if any)
        let mut current_time = start_time_ms;
        let mut current_beat = start_beat;
        for beat in measure.beats.iter_mut() {
            beat.time = current_time;
            current_time += time_per_beat_ms;
            current_beat += 1.0;
        }
        
        // Advance time even if measure is empty (empty measures still take time)
        let new_time = start_time_ms + (time_per_beat_ms * actual_beats as f64);
        let new_beat = start_beat + actual_beats as f64;

        (measure, idx, new_time, new_beat)
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

fn update_bpm_if_needed(
    current_bpm: &mut f64,
    current_beat: f64,
    bpm_index: &mut usize,
    bpms: &[(f64, f64)],
) {
    while *bpm_index < bpms.len() {
        let (bpm_beat, new_bpm) = bpms[*bpm_index];
        if bpm_beat <= current_beat {
            *current_bpm = new_bpm;
            *bpm_index += 1;
        } else {
            break;
        }
    }
}
