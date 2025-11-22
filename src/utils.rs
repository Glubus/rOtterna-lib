use regex::Regex;
use std::str::FromStr;
pub fn parse_field<T: FromStr>(content: &str, pattern: &str, field: &mut T) {
    let re = Regex::new(pattern).unwrap();
    if let Some(cap) = re.captures(content) {
        if let Ok(value) = cap[1].trim().parse::<T>() {
            *field = value;
        }
    }
}

pub fn parse_pairs(content: &str, pattern: &str, list: &mut Vec<(f64, f64)>) {
    let re = Regex::new(pattern).unwrap();

    if let Some(cap) = re.captures(content) {
        let raw_str = cap[1].trim();
        // Nettoyage des sauts de ligne et espaces
        let clean_str = raw_str.replace(&['\n', '\r', ' '][..], "");

        println!("[parse_pairs] Raw: '{}' -> Clean: '{}'", raw_str, clean_str);

        for pair in clean_str.split(',') {
            if pair.is_empty() {
                continue;
            }

            // On tente de diviser sur '=' et de parser les deux côtés en f64
            if let Some((k, v)) = pair.split_once('=') {
                match (k.parse::<f64>(), v.parse::<f64>()) {
                    (Ok(key), Ok(val)) => list.push((key, val)),
                    _ => println!("[parse_pairs] Failed to parse pair: {}", pair),
                }
            }
        }
    } else {
        println!("[parse_pairs] No match found for pattern: {}", pattern);
    }
}
