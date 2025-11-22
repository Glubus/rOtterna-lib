use rotterna_lib::structs::{SmFile, OsuSettings};
use rotterna_lib::converter::osu::create_basic_osu;
use std::path::PathBuf;
use std::fs;

fn main() {
    // Load MEGALOVANIA.sm file
    let sm_path = PathBuf::from("assets/MEGALOVANIA.sm");
    
    println!("Loading SM file: {:?}", sm_path);
    
    let sm_file = match SmFile::from_file(sm_path) {
        Ok(file) => file,
        Err(e) => {
            eprintln!("Error loading SM file: {}", e);
            std::process::exit(1);
        }
    };
    println!("Offset: {}", sm_file.offset);
    println!("Loaded {} chart(s)", sm_file.charts.len());
    
    // Convert each chart to .osu format
    for (chart_idx, chart) in sm_file.charts.iter().enumerate() {
        println!("\nConverting chart {}: {} ({})", 
            chart_idx + 1, 
            chart.difficulty.trim_end_matches(':'),
            chart.stepstype.trim_end_matches(':')
        );
        
        // Create OsuSettings (you can adjust these values)
        let settings = OsuSettings {
            hp: 5.0,  // HP Drain Rate
            od: 8.0,  // Overall Difficulty
        };
        
        // Convert to .osu format
        match create_basic_osu(&sm_file, chart, &settings) {
            Ok(osu_content) => {
                // Generate output filename
                let difficulty_name = chart.difficulty.trim_end_matches(':');
                let sanitized_name = difficulty_name
                    .chars()
                    .map(|c| if c.is_alphanumeric() || c == '-' || c == '_' { c } else { '_' })
                    .collect::<String>();
                
                let output_path = format!("output/MEGALOVANIA_{}.osu", sanitized_name);
                
                // Create output directory if it doesn't exist
                if let Some(parent) = PathBuf::from(&output_path).parent() {
                    fs::create_dir_all(parent).expect("Failed to create output directory");
                }
                
                // Write .osu file
                fs::write(&output_path, osu_content)
                    .expect("Failed to write .osu file");
                
                println!("  ✓ Saved to: {}", output_path);
            }
            Err(e) => {
                eprintln!("  ✗ Error converting chart: {}", e);
            }
        }
    }
    
    println!("\nConversion complete!");
}

