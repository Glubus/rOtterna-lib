use crate::structs::{SmFile, OsuSettings, Chart};

// StepMania row system constants (must match decode.rs)
const ROWS_PER_BEAT: f64 = 48.0;  // 1 beat = 48 rows (for 4/4 time)
pub fn create_basic_osu(sm_file: &SmFile, chart: &Chart, settings: &OsuSettings) -> Result<String, String> {
    // This is a placeholder - should use rosu-map instead
    let mut osu = String::new();
    
    osu.push_str("osu file format v14\n");
    osu.push_str("\n");
    osu.push_str("[General]\n");
    osu.push_str("AudioFilename: ");
    osu.push_str(&sm_file.metadata.music);
    osu.push_str("\n");
    osu.push_str("AudioLeadIn: 0\n");
    osu.push_str("PreviewTime: -1\n");
    osu.push_str("Countdown: 0\n");
    osu.push_str("SampleSet: Normal\n");
    osu.push_str("StackLeniency: 0.7\n");
    osu.push_str("Mode: 3\n"); // osu!mania
    osu.push_str("LetterboxInBreaks: 0\n");
    osu.push_str("WidescreenStoryboard: 0\n");
    osu.push_str("\n");
    osu.push_str("[Editor]\n");
    osu.push_str("\n");
    osu.push_str("[Metadata]\n");
    osu.push_str("Title:");
    osu.push_str(&sm_file.metadata.title);
    osu.push_str("\n");
    osu.push_str("TitleUnicode:");
    osu.push_str(&sm_file.metadata.title);
    osu.push_str("\n");
    osu.push_str("Artist:");
    osu.push_str(&sm_file.metadata.artist);
    osu.push_str("\n");
    osu.push_str("ArtistUnicode:");
    osu.push_str(&sm_file.metadata.artist);
    osu.push_str("\n");
    osu.push_str("Creator:");
    osu.push_str(&sm_file.metadata.credit);
    osu.push_str("\n");
    osu.push_str("Version:");
    osu.push_str(&chart.difficulty);
    osu.push_str("\n");
    osu.push_str("Source:\n");
    osu.push_str("Tags: rOtterna\n");
    osu.push_str("BeatmapID: 0\n");
    osu.push_str("BeatmapSetID: -1\n");
    osu.push_str("\n");
    osu.push_str("[Difficulty]\n");
    osu.push_str(&format!("HPDrainRate: {}\n", settings.hp));
    osu.push_str("CircleSize: 4\n");
    osu.push_str(&format!("OverallDifficulty: {}\n", settings.od));
    osu.push_str("ApproachRate: 5\n");
    osu.push_str("SliderMultiplier: 1.4\n");
    osu.push_str("SliderTickRate: 1\n");
    osu.push_str("\n");
    osu.push_str("[Events]\n");
    osu.push_str("//Background and Video events\n");
    if !sm_file.metadata.background.is_empty() {
        osu.push_str("0,0,\"");
        osu.push_str(&sm_file.metadata.background);
        osu.push_str("\",0,0\n");
    }
    osu.push_str("//Break Periods\n");
    osu.push_str("//Storyboard Layer 0 (Background)\n");
    osu.push_str("//Storyboard Layer 1 (Fail)\n");
    osu.push_str("//Storyboard Layer 2 (Pass)\n");
    osu.push_str("//Storyboard Layer 3 (Foreground)\n");
    osu.push_str("//Storyboard Sound Samples\n");
    osu.push_str("\n");
    
    // Generate timing points for all BPM changes
    osu.push_str("[TimingPoints]\n");
    
    println!("[create_basic_osu] Found {} BPM change(s)", sm_file.bpms.len());
    
    // Offset is already in milliseconds (converted in decode.rs)
    let offset_ms = sm_file.offset as i32;
    
    if sm_file.bpms.is_empty() {
        // Default BPM if none found
        println!("[create_basic_osu] No BPMs found, using default 120 BPM");
        osu.push_str(&format!("{},{},4,2,0,100,1,0\n", offset_ms, 500.0)); // 120 BPM = 500ms per beat
    } else {
        // Generate a timing point for each BPM change
        // BPMs are stored as (row, bpm) pairs, need to convert rows to beats and then to time
        let mut current_time_ms = 0.0;
        let mut current_row = 0.0;
        let mut current_bpm = sm_file.bpms[0].1;
        
        for (idx, (bpm_row, bpm)) in sm_file.bpms.iter().enumerate() {
            // Calculate time elapsed from previous BPM change
            if idx > 0 {
                let rows_elapsed = bpm_row - current_row;
                let beats_elapsed = rows_elapsed / ROWS_PER_BEAT;
                let time_elapsed_ms = (beats_elapsed / current_bpm) * 60000.0;
                current_time_ms += time_elapsed_ms;
            }
            
            // Apply offset: timing point starts at offset + calculated time
            let time_ms = (current_time_ms + sm_file.offset) as i32;
            
            // Calculate beat duration in milliseconds (60000ms / BPM)
            let beat_duration_ms = 60000.0 / bpm;
            
            // Convert row to beat for display
            let bpm_beat = bpm_row / ROWS_PER_BEAT;
            
            println!("[create_basic_osu] BPM change at row {} (beat {}) ({}ms): {} BPM ({}ms per beat)", 
                bpm_row, bpm_beat, time_ms, bpm, beat_duration_ms);
            
             // Format: time,beatLength,meter,sampleSet,sampleIndex,volume,uninherited,effects
             // uninherited = 1 means this is a timing point (not inherited)
             // beatLength can have decimals, don't round it
            osu.push_str(&format!("{},{},4,2,0,100,1,0\n", time_ms, beat_duration_ms));
            
            // Update for next iteration
            current_row = *bpm_row;
            current_bpm = *bpm;
        }
    }
    
    osu.push_str("\n");
    
    osu.push_str("[HitObjects]\n");
    // Convert notes
    let column_count = if chart.column_count > 0 { chart.column_count } else { 4 }; // Default to 4 columns if not set
    
    for measure in chart.measures.iter() {
        // Convert note row to osu format
        for beat in measure.beats.iter() {
            let time_ms = beat.time;
            
            for (note_idx, note) in beat.notes.iter().enumerate() {
                if *note {
                    // Calculate column position: osu!mania uses 512 pixels width, divide by column count
                    let column = (note_idx as f64 + 0.5) * 512.0 / column_count as f64;
                    // Format: x,y,time,type,hitSound,objectParams,hitSample
                    // For osu!mania: x is column position, y is 192 (center), type 1 = circle
                    // Apply offset: notes are already calculated from 0, add offset to match timing point
                    let note_time_ms = (time_ms + sm_file.offset) as i32;
                    osu.push_str(&format!("{},{},{},1,0,0:0:0:0:\n", column as i32, 192, note_time_ms));
                }
            }
        }
    }
    
    Ok(osu)
}
