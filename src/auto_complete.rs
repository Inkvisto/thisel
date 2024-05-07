use multimap::MultiMap;
use std::fs;
use std::iter::Iterator;
use std::path::Path;
use std::str;

/// # Panics
///
/// Will panic if path not exists
pub fn autocomplete(query: &str) -> Vec<String> {
    let re = crate::regex!(r"\b\w+\b$");

    let path = Path::new("resources/grammar.json");
    let mut suggestions = MultiMap::new();

    let file = fs::File::open(path).expect("file should open read only");
    let contents: serde_json::Value =
        serde_json::from_reader(file).expect("file should be proper JSON");
    let elements = contents.as_object().unwrap().iter();

    for values in elements {
        let chars: Vec<char> = values.0.chars().collect();
        for idx in 1..chars.len() {
            let slice: String = chars[0..idx].iter().collect();
            suggestions.insert(slice.to_string(), values.0.to_string());
        }
    }
    if let Some(capture) = re.captures(query) {
        let last_word = capture.get(0).unwrap().as_str();
        get_suggestion(last_word, &suggestions)
    } else {
        vec![String::new()]
    }
}

fn get_suggestion(query: &str, suggestions: &MultiMap<String, String>) -> Vec<String> {
    let suggestion = suggestions.get_vec(query);
    match suggestion {
        Some(values) => values.clone(),
        None => Vec::<String>::new(),
    }
}
