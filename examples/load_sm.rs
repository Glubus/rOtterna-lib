use rotterna_lib::structs::SmFile;
use std::path::PathBuf;

fn main() {
    // Load a .sm file
    let path = PathBuf::from("assets/MEGALOVANIA.sm");
    
    println!("Loading file: {:?}", path);
    
    match SmFile::from_file(path) {
        Ok(sm_file) => {
            println!("\n=== METADATA ===");
            println!("Title: {}", sm_file.metadata.title);
            println!("Artist: {}", sm_file.metadata.artist);
            println!("Subtitle: {}", sm_file.metadata.subtitle);
            println!("Offset: {}", sm_file.offset);
            
            println!("\n=== BPMS ===");
            for (beat, bpm) in &sm_file.bpms {
                println!("  Beat {}: {} BPM", beat, bpm);
            }
            
            println!("\n=== STOPS ===");
            for (beat, duration) in &sm_file.stops {
                println!("  Beat {}: {} seconds", beat, duration);
            }
            
            println!("\n=== CHARTS ===");
            println!("Number of charts: {}", sm_file.charts.len());
            
            for (chart_idx, chart) in sm_file.charts.iter().enumerate() {
                println!("\n--- Chart {} ---", chart_idx + 1);
                println!("Stepstype: {}", chart.stepstype);
                println!("Difficulty: {}", chart.difficulty);
                println!("Meter: {}", chart.meter);
                println!("Number of measures: {}", chart.measures.len());
                
                // Print last measure to check total duration
                if let Some(last_measure) = chart.measures.last() {
                    if let Some(last_beat) = last_measure.beats.last() {
                        let total_duration_sec = last_beat.time / 1000.0;
                        println!("Last beat time: {:.2} ms ({:.2} seconds / {:.2} minutes)", 
                            last_beat.time, total_duration_sec, total_duration_sec / 60.0);
                    } else if !chart.measures.is_empty() {
                        // Empty measure - calculate from start_time
                        let total_duration_sec = last_measure.start_time / 1000.0;
                        println!("Last measure start time: {:.2} ms ({:.2} seconds / {:.2} minutes)", 
                            last_measure.start_time, total_duration_sec, total_duration_sec / 60.0);
                    }
                }
                
                // Print first few measures
                for (measure_idx, measure) in chart.measures.iter().take(3).enumerate() {
                    println!("\n  Measure {}:", measure_idx + 1);
                    println!("    Start time: {:.3} ms", measure.start_time);
                    println!("    Number of beats: {}", measure.beats.len());
                    
                    // Print first few beats
                    for (beat_idx, beat) in measure.beats.iter().take(3).enumerate() {
                        println!("      Beat {}: time={:.3} ms, notes={:?}", 
                            beat_idx + 1, beat.time, beat.notes);
                    }
                    if measure.beats.len() > 3 {
                        println!("      ... ({} more beats)", measure.beats.len() - 3);
                    }
                }
                if chart.measures.len() > 3 {
                    println!("\n  ... ({} more measures)", chart.measures.len() - 3);
                }
            }
        }
        Err(e) => {
            eprintln!("Error loading file: {}", e);
            std::process::exit(1);
        }
    }
}

