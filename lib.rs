use std::fs;
use std::error::Error;
use std::env;
use colored::Colorize;
use regex::Regex;
use Option::Some;

pub struct Config {
    pub query: String,
    pub file_path: String,
    // additional parameters
    pub is_case_insensitive: bool,
    pub find_only_full_words: bool,
    pub max_output: Option<u32>,
}

impl Config {
    pub fn build(args: &[String]) -> Result<Config, &'static str> {
        if args.len() > 1 && args.len() < 3 {
            return Err("Not enough minimum arguments provided: \
                        you should provide 1 more: file_path");
        }
        else if args.len() < 3 {
            return Err("Not enough minimum arguments provided: \
                       you should provide 2 more: query and file_path");
        }

        // env::var returns Result<String, VarError> type
        let max_output_val = match env::var("MAX_OUTPUT") {
            // if it's a String type
            Ok(val) => {
                // try to convert String type to u32
                match val.parse::<u32>() {
                    // if it's a valid u32 value -> return it
                    Ok(parsed_val) => Some(parsed_val),
                    // if not, return None
                    Err(_) => {
                        eprintln!("MAX_OUTPUT variable is not a valid u32 value!");
                        None
                    }
                }
            },
            // if it's a VarError type, return None
            Err(_) => None
        };

        return Ok(Config {
            query: args[1].to_string(),
            file_path: args[2].to_string(),
            is_case_insensitive: env::var("IGNORE_CASE").is_ok(),
            find_only_full_words: env::var("FULL_WORDS").is_ok(),
            max_output: max_output_val,
        });
    }
}
// Box<dyn Error> can be used for returning unknown type of Error,
// but it should implement Error trait
// () result types, tells that we run this function for side effects only
pub fn run(config: Config) -> Result<(), Box<dyn Error>> {
    // read content of the file
    let content = fs::read_to_string(config.file_path)?;

        // case_insensitive + not full_words
    let results = if config.is_case_insensitive == true && config.find_only_full_words == false {
        search_case_insensitive(&config.query, &content, &config.max_output)
        // not case_insensitive + not full_words
    } else if config.is_case_insensitive == false && config.find_only_full_words == false {
        search_case_sensitive(&config.query, &content, &config.max_output)
        // not case_insensitive + full_words
    } else if config.is_case_insensitive == false && config.find_only_full_words == true{
        search_words_case_sensitive(&config.query, &content, &config.max_output)
        // case_insensitive + full_words
    } else {
        search_words_case_insensitive(&config.query, &content, &config.max_output)
    };

    for line in results.iter() {
        println!("line: {} | {}", line.0, line.1);
    }

    Ok(())
}
// query doesn't have an explicit lifetime, because we need to return
// a vector containing a slice of content string: then lifetimes of Vec<&str>
// and content &str must be the same, which guarantees that Vec<&str> lives
// just long enough as a content.
pub fn search_case_sensitive(query: &str, content: &str, max_count: &Option<u32>) -> Vec<(u32, String)> {
    let mut results = Vec::new();
    let mut line_number = 0;

    for line in content.lines() {
        line_number += 1;
        if line.contains(query) && results.len() < max_count.unwrap_or(u32::MAX) as usize {
            if let Some(index) = line.find(query) {
                let mut colored_line = String::new();
                // coloring found query in line
                colored_line.push_str(&line[..index]);
                colored_line.push_str(&line[index..index + query.len()].red().to_string());
                colored_line.push_str(&line[index + query.len()..]);

                results.push((line_number, colored_line));
            }
        }
    }
    results
}

pub fn search_case_insensitive(query: &str, content: &str, max_count: &Option<u32>) -> Vec<(u32, String)> {
    let mut results = Vec::new();
    let lowercase_query = query.to_lowercase();
    let mut line_number = 0;

    for line in content.lines() {
        let lowercase_line = line.to_lowercase();

        line_number += 1;

        if lowercase_line.contains(&lowercase_query) &&
            results.len() < max_count.unwrap_or(u32::MAX) as usize {

            if let Some(index) = lowercase_line.find(lowercase_query.as_str()) {
                let mut colored_line = String::new();
                // coloring found query in line
                colored_line.push_str(&line[..index]);
                colored_line.push_str(&line[index..index + query.len()].red().to_string());
                colored_line.push_str(&line[index + query.len()..]);

                results.push((line_number, colored_line));
            }
        }
    }
    results
}

pub fn search_words_case_sensitive(query: &str, content: &str, max_count: &Option<u32>) -> Vec<(u32, String)> {
    let mut results = Vec::new();

    let pattern = format!(r"\b{}\b", regex::escape(query));
    let re = Regex::new(&pattern).unwrap();

    for (number, line) in content.lines().enumerate() {
        let mut colored_result = line.to_string();
        let mut last_end = 0;

        // iterator over the all regex matches
        // in this case we find all the words that contain "query"
        for mat in re.find_iter(line) {
            let start = mat.start();
            let end = mat.end();
            let word = &line[start..end];
            // check if word is full
            if word.chars().all(|c| c.is_alphabetic()) {
                colored_result = format!("{}{}{}", &line[..start], &line[start..end].red(), &line[end..]);
                // updating last end index of the found word that matches all conditions
                last_end = end;
            }
        }
        // if we found word that contains query and is full
        // return this line
        if last_end > 0 && results.len() < max_count.unwrap_or(u32::MAX) as usize {
            results.push((number as u32 + 1_u32, colored_result));
        }
    }
    results
}

pub fn search_words_case_insensitive(query: &str, content: &str, max_count: &Option<u32>) -> Vec<(u32, String)> {
    let mut results = Vec::new();

    let pattern = format!(r"\b{}\b", regex::escape(&query.to_lowercase()));
    let re = Regex::new(&pattern).unwrap();

    for (number, line) in content.lines().enumerate() {
        let mut colored_result = line.to_string();
        let mut last_end = 0;

        // iterator over the all regex matches
        // in this case we find all the words that contain "query"
        for mat in re.find_iter(&line.to_lowercase()) {
            let start = mat.start();
            let end = mat.end();
            let word = &line[start..end];
            // check if word is full
            if word.to_lowercase().chars().all(|c| c.is_alphabetic()) {
                colored_result = format!("{}{}{}", &line[..start], &line[start..end].red(), &line[end..]);
                // updating last end index of the found word that matches all conditions
                last_end = end;
            }
        }
        // if we found word in line that contains query and is full
        // add this lines to the results vector
        if last_end > 0 && results.len() < max_count.unwrap_or(u32::MAX) as usize {
            results.push((number as u32 + 1_u32, colored_result));
        }
    }
    results
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn case_sensitive() {
        let query = "saf";
        let max_count = Some(1_u32);
        let content = "\
Rust:
safe, fast, productive.
Pick three.";

        let red_word = "saf".red().to_string();
        let rest_of_line = "e, fast, productive.".to_string();

        assert_eq!(vec![(2, format!("{red_word}{rest_of_line}"))], search_case_sensitive(&query, &content, &max_count));
    }

    #[test]
    fn case_insensitive() {
        let query = "rUsT";
        let max_count = Some(2_u32);
        let content = "\
Rust:
safe, fast, productive.
Pick three.
Trust me.";

        let first_occasion = format!("{}{}", "Rust".red().to_string(), ":".to_string());
        let second_occasion = "Trust me.".to_string().replace("rust", &"rust".red().to_string());

        assert_eq!(vec![(1, first_occasion), (4, second_occasion)], search_case_insensitive(&query, &content, &max_count));
    }

    #[test]
    fn case_sensitive_full_words() {
        let query = "rust";
        let max_count = Some(1_u32);
        let content = "\
rust:
safe, fast, productive.
Pick three.
Trust me.";

        assert_eq!(vec![(1, format!("{}{}", "rust".red().to_string(), ":".to_string()))],
                   search_words_case_sensitive(&query, &content, &max_count));
    }

    #[test]
    fn case_insensitive_full_words() {
        let query = "rUsT";
        let max_count = Some(2_u32);
        let content = "\
Rust:
safe, fast, productive.
Pick three.
It's up to you.
But i love RUST!";

        assert_eq!(vec![(1, format!("{}{}", "Rust".red(), ":")),
                        (5, format!("{}{}{}", "But i love ", "RUST".red(), "!"))],
        search_case_insensitive(&query, &content, &max_count));
    }
}