use crate::utils::parse_field;
#[derive(Debug, Clone)]
pub struct SmFile {
    pub metadata: Metadata,
    pub offset: f64, // Time in MILLISECONDS
    pub bpms: Vec<(f64, f64)>,  // (row, bpm) - row position and BPM value
    pub stops: Vec<(f64, f64)>, // (row, duration) - row position and duration in seconds
    pub charts: Vec<Chart>,
}

impl SmFile {
    pub fn new() -> SmFile {
        SmFile {
            metadata: Metadata::new(),
            offset: 0.0,
            bpms: Vec::new(),
            stops: Vec::new(),
            charts: Vec::new(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Metadata {
    pub title: String,
    pub subtitle: String,
    pub artist: String,
    pub title_translit: String,
    pub artist_translit: String,
    pub credit: String,
    pub music: String,
    pub banner: String,
    pub background: String,
}

impl Metadata {
    pub fn new() -> Metadata {
        Metadata {
            title: String::new(),
            subtitle: String::new(),
            artist: String::new(),
            title_translit: String::new(),
            artist_translit: String::new(),
            credit: String::new(),
            music: String::new(),
            banner: String::new(),
            background: String::new(),
        }
    }
    pub fn parse(&mut self, content: &str) {
        parse_field(content, r"#TITLE:(.*?);", &mut self.title);
        parse_field(content, r"#SUBTITLE:(.*?);", &mut self.subtitle);
        parse_field(content, r"#ARTIST:(.*?);", &mut self.artist);
        parse_field(content, r"#TITLETRANSLIT:(.*?);", &mut self.title_translit);
        parse_field(
            content,
            r"#ARTISTTRANSLIT:(.*?);",
            &mut self.artist_translit,
        );
        parse_field(content, r"#CREDIT:(.*?);", &mut self.credit);
        parse_field(content, r"#MUSIC:(.*?);", &mut self.music);
        parse_field(content, r"#BANNER:(.*?);", &mut self.banner);
        parse_field(content, r"#BACKGROUND:(.*?);", &mut self.background);
    }
}

#[derive(Debug, Clone)]
pub struct Chart {
    pub stepstype: String,
    pub description: String,
    pub difficulty: String,
    pub meter: u32,
    pub radar_values: Vec<f64>,
    pub column_count: u32,
    pub measures: Vec<Measure>,
}

impl Chart {
    pub fn new() -> Chart {
        Chart {
            stepstype: String::new(),
            description: String::new(),
            difficulty: String::new(),
            meter: 0,
            radar_values: Vec::new(),
            column_count: 0,
            measures: Vec::new(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Measure {
    pub beats: Vec<Beat>,
    pub start_time: f64, // Time in MILLISECONDS
}

impl Measure {
    pub fn new() -> Measure {
        Measure {
            beats: Vec::new(),
            start_time: 0.0,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Beat {
    pub time: f64,        // Time in MILLISECONDS
    pub notes: Vec<bool>, // true = note, false = empty
}
impl Beat {
    pub fn new() -> Beat {
        Beat {
            time: 0.0,
            notes: Vec::new(),
        }
    }
}



pub struct OsuSettings
{
    pub od: f64, 
    pub hp: f64,
}