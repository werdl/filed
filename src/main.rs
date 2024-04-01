use std::io::Read;

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Hash)]
struct FileSignature {
    header: Option<String>,
    trailer: Option<String>,
    offset: u64,
    ext: String,
    category: String,
    desc: String,
}

fn read_to_u8_blocks(filename: String) -> Vec<u8> {
    let mut file = std::fs::File::open(filename).expect("file not found");
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer).expect("error reading file");
    buffer
}

trait HexFormatter {
    fn to_hex(&self) -> String;
}

impl HexFormatter for Vec<u8> {
    fn to_hex(&self) -> String {
        self.iter().map(|byte| format!("{:02x} ", byte)).collect::<String>().to_ascii_lowercase()
    }
}

impl HexFormatter for Option<Vec<u8>> {
    fn to_hex(&self) -> String {
        match self {
            Some(bytes) => bytes.to_hex(),
            None => String::from("None"),
        }
    }

}

impl HexFormatter for &[u8] {
    fn to_hex(&self) -> String {
        self.iter().map(|byte| format!("{:02x} ", byte)).collect::<String>().to_ascii_lowercase()
    }
}

fn read_sigs(filename: String) -> Vec<FileSignature> {
    let mut file = std::fs::File::open(filename).expect("file not found");
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer).expect("error reading file");
    let sigs: Vec<FileSignature> = serde_json::from_slice(&buffer).expect("error parsing json");

    sigs
}

trait MatchAgainst {
    fn match_against(&self, buffer: FileSignature) -> bool;
}

impl MatchAgainst for Vec<u8> {
    fn match_against(&self, buffer: FileSignature) -> bool {
        let mut matched = true;
        
        for (i, segment) in buffer.header.unwrap_or_default().split_whitespace().enumerate() {
            let byte = u8::from_str_radix(segment, 16).unwrap();

            if self.get(i as usize) != Some(&byte) {
                matched = false;
                break;
            }
        }

        matched
    }
}

fn vec_to_human_readable_string<T>(vec: Vec<T>) -> String 
where T: std::fmt::Display {
    let mut string = String::new();

    if vec.len() == 0 || (vec.first().unwrap().to_string() == "") {
        return String::from("variable or empty extension");
    }

    if vec.len() == 1 {
        return vec.first().unwrap().to_string();
    }

    if vec.len() == 2 {
        return format!("{} or {}", vec.first().unwrap(), vec.last().unwrap());
    }

    for i in 0..vec.len() - 1 {
        string.push_str(&format!("{}, ", vec[i]));
    }

    string.push_str(&format!("or {}", vec.last().unwrap()));

    string
}

fn main() {
    // read the file given, or else error out
    let filename = std::env::args().nth(1).expect("no filename given");

    // read the file into two byte blocks
    let blocks = read_to_u8_blocks(filename);

    let sigs = read_sigs("file_sigs.json".to_string());

    let mut matched = Vec::new();

    for sig in sigs {
        if blocks.match_against(sig.clone()) {
            matched.push(sig);
        }
    }

    // now we check how many matched. if there are more than one, prioritize first any with a trailer, then the one with the longest header
    let mut best_match = matched.first().unwrap().clone();

    for sig in matched.clone() {
        if sig.clone().trailer.is_some() && best_match.trailer.is_none() {
            best_match = sig.clone();
        } else if sig.clone().header.unwrap_or_default().len() > best_match.clone().header.unwrap_or_default().len() {
            best_match = sig.clone();
        }
    }

    println!("Most likely match: {} ({}, usually ends with {})", best_match.desc, best_match.category, vec_to_human_readable_string(best_match.ext.split("|").collect()));

    
    if matched.len() != 1 {
        println!("Other possible matches:");
    }
    for sig in matched.iter() {
        if sig != &best_match {
            println!("  {} ({}, usually ends with {})", sig.desc, sig.category, vec_to_human_readable_string(sig.ext.split("|").collect()));
        }
    }
}
