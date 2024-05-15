use std::fs;
use std::error::Error;
use std::env;
use colored::Colorize;
use regex::Regex;
use Option::Some;
use std::io::BufRead;

pub struct Config {
    // required parameters
    pub query: String,
    pub file_path: String,
    // optional parameters, have their own functions implementations
    pub is_case_insensitive: bool,
    pub find_only_full_words: bool,
    pub find_only_full_lines: bool,
    // optional parameters: don't have their own function implementations
    pub max_output: Option<u32>,
    pub invert_match: bool,
}

impl Config {
    pub fn build(mut args: impl Iterator<Item = String>) -> Result<Config, &'static str> {
        // skipping first not used parameter: [target/debug/RustyGrep]
        args.next();

        let query_value = match args.next() {
            Some(value) => value,
            None => return Err("Not enough required arguments: absent query parameter")
        };
        let file_path_value = match args.next() {
            Some(value) => value,
            None => return Err("Not enough required arguments: absent file_path parameter")
        };

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
            query: query_value,
            file_path: file_path_value,
            is_case_insensitive: env::var("IGNORE_CASE").is_ok(),
            find_only_full_words: env::var("FULL_WORDS").is_ok(),
            find_only_full_lines: env::var("FULL_LINES").is_ok(),
            max_output: max_output_val,
            invert_match: env::var("INVERT_MATCH").is_ok(),
        });
    }
}

struct AdditionalParameters {
    pub max_output: Option<u32>,
    pub invert_match: bool,
}

// Box<dyn Error> can be used for returning unknown type of Error,
// but it should implement Error trait
// () result types, tells that we run this function for side effects only
pub fn run(config: Config) -> Result<(), Box<dyn Error>> {
    // read content of the file
    let content = fs::read_to_string(config.file_path)?;
    // fill struct with params that must be passed to each function
    let additional_params = AdditionalParameters {
        max_output: config.max_output,
        invert_match: config.invert_match,
    };

    // case_insensitive + not full_words
    let results = if config.is_case_insensitive == true && config.find_only_full_words == false {
        search_case_insensitive(&config.query, &content, &additional_params)
        // not case_insensitive + not full_words
    } else if config.is_case_insensitive == false && config.find_only_full_words == false {
        search_case_sensitive(&config.query, &content, &additional_params)
        // not case_insensitive + full_words
    } else if config.is_case_insensitive == false && config.find_only_full_words == true{
        search_words_case_sensitive(&config.query, &content, &additional_params)
        // case_insensitive + full_words
    } else if config.is_case_insensitive == true && config.find_only_full_words == true {
        search_words_case_insensitive(&config.query, &content, &additional_params)
        // case_insensitive + full_lines
    } else if config.find_only_full_lines == true && config.is_case_insensitive == true {
        search_lines_case_insensitive(&config.query, &content, &additional_params)
        // not case_insensitive + full_lines
    } else {
        search_lines_case_sensitive(&config.query, &content, &additional_params)
    };

    for line in results.iter() {
        // number | match
        println!("line: {} | {}", line.0, line.1);
    }

    Ok(())
}

fn search_case_sensitive(query: &str, content: &str, params: &AdditionalParameters) -> Vec<(u32, String)> {
    content
        .lines()
        .enumerate()
        .filter_map(|(line_number, line)| {
            let match_found = line.contains(query);

            if params.invert_match {
                if !match_found {
                    Some((line_number as u32 + 1u32, line.to_string()))
            } else { None }

            } else if !params.invert_match && match_found {
                line.find(query).map(|index| {
                    let (before, after) = line.split_at(index);
                    let colored_line = format!("{}{}{}", before, &after[..query.len()].red(),
                                               &after[query.len()..]);

                    (line_number as u32 + 1, colored_line)
                })
            } else { None }
    })
        .take(params.max_output.unwrap_or(u32::MAX) as usize)
        .collect()
}

fn search_case_insensitive(query: &str, content: &str, params: &AdditionalParameters) -> Vec<(u32, String)> {
    content
        // split to lines
        .lines()
        // each element has the following type: (usize, &str)
        .enumerate()
        // filter values from content
        .filter_map(|(line_number, line)| {
            let match_found = line.to_lowercase().contains(&query.to_lowercase());

            if params.invert_match {
                if !match_found {
                    Some((line_number as u32 + 1u32, line.to_string()))
                } else { None }

            } else if !params.invert_match && match_found {
                line.to_lowercase().find(&query.to_lowercase()).map(|index| {
                    // this code applies only when find method results is Some(index)
                    // if it's None, map method returns None
                    let (before, after) = line.split_at(index);
                    let colored_line = format!("{}{}{}", before, &after[..query.len()].red(),
                                               &after[query.len()..]);
                    // returns tuple (u32, String)
                    (line_number as u32 + 1, colored_line)
                })
            } else { None }
        })
        // return only first MAX_OUTPUT lines. If variable is None, set it to u32::MAX
        .take(params.max_output.unwrap_or(u32::MAX) as usize)
        // collect all filtered element to the data type: Vec<u32, String>
        .collect::<Vec<(u32, String)>>()
}

fn search_words_case_sensitive(query: &str, content: &str, params: &AdditionalParameters) -> Vec<(u32, String)> {
    let mut results = Vec::new();
    let mut invert_match_results = Vec::new();

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
        if last_end > 0 && results.len() < params.max_output.unwrap_or(u32::MAX) as usize {
            results.push((number as u32 + 1_u32, colored_result));
        } else {
            invert_match_results.push((number as u32 + 1_u32, line.to_string()));
        }
    }
    if params.invert_match {
        invert_match_results
    } else {
        results
    }
}

fn search_words_case_insensitive(query: &str, content: &str, params: &AdditionalParameters) -> Vec<(u32, String)> {
    let mut results = Vec::new();
    let mut invert_match_results = Vec::new();

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
        if last_end > 0 && results.len() < params.max_output.unwrap_or(u32::MAX) as usize {
            results.push((number as u32 + 1_u32, colored_result));
        } else {
            invert_match_results.push((number as u32 + 1_u32, line.to_string()));
        }
    }
    if params.invert_match {
        invert_match_results
    } else {
        results
    }
}

fn search_lines_case_sensitive(query: &str, content: &str, params: &AdditionalParameters) -> Vec<(u32, String)> {
    content
        .lines()
        .enumerate()
        .filter_map(|(line_number, line)| {
            let match_found = line == query;

            if !params.invert_match {
                if match_found {
                    Some((line_number as u32 + 1u32, line.red().to_string()))
                } else { None }
            } else if params.invert_match {
                if !match_found {
                    Some((line_number as u32 + 1u32, line.to_string()))
                } else { None }
            } else { None }
        })
        .take(params.max_output.unwrap_or(u32::MAX) as usize)
        .collect::<Vec<(u32, String)>>()
}

fn search_lines_case_insensitive(query: &str, content: &str, params: &AdditionalParameters) -> Vec<(u32, String)> {
    content
        .lines()
        .enumerate()
        .filter_map(|(line_number, line)| {
            let match_found = line.to_lowercase() == query.to_lowercase();

            if !params.invert_match {
                if match_found {
                    Some((line_number as u32 + 1u32, line.red().to_string()))
                } else { None }
            } else if params.invert_match {
                if !match_found {
                    Some((line_number as u32 + 1u32, line.to_string()))
                } else { None }
            } else { None }
        })
        .take(params.max_output.unwrap_or(u32::MAX) as usize)
        .collect::<Vec<(u32, String)>>()
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn case_sensitive() {
        let query = "saf";
        let content = "\
Rust:
safe, fast, productive.
Pick three.";

        let params = AdditionalParameters {
            max_output: Some(1_u32),
            invert_match: false
        };

        let red_word = "saf".red().to_string();
        let rest_of_line = "e, fast, productive.".to_string();

        assert_eq!(vec![(2, format!("{red_word}{rest_of_line}"))], search_case_sensitive(&query, &content, &params));
    }

    #[test]
    fn case_insensitive() {
        let query = "rUsT";
        let content = "\
Rust:
safe, fast, productive.
Pick three.
Trust me.";

        let params = AdditionalParameters {
            max_output: Some(2_u32),
            invert_match: false
        };

        let first_occasion = format!("{}{}", "Rust".red().to_string(), ":".to_string());
        let second_occasion = "Trust me.".to_string().replace("rust", &"rust".red().to_string());

        assert_eq!(vec![(1, first_occasion), (4, second_occasion)], search_case_insensitive(&query, &content, &params));
    }

    #[test]
    fn case_sensitive_full_words() {
        let query = "rust";
        let content = "\
rust:
safe, fast, productive.
Pick three.
Trust me.";

        let params = AdditionalParameters {
            max_output: Some(1_u32),
            invert_match: false
        };

        assert_eq!(vec![(1, format!("{}{}", "rust".red().to_string(), ":".to_string()))],
                   search_words_case_sensitive(&query, &content, &params));
    }

    #[test]
    fn case_insensitive_full_words() {
        let query = "rUsT";
        let content = "\
Rust:
safe, fast, productive.
Pick three.
It's up to you.
But i love RUST!";

        let params = AdditionalParameters {
            max_output: Some(3_u32),
            invert_match: false
        };

        assert_eq!(vec![(1, format!("{}{}", "Rust".red(), ":")),
                        (5, format!("{}{}{}", "But i love ", "RUST".red(), "!"))],
                   search_case_insensitive(&query, &content, &params));
    }

    #[test]
    fn case_insensitive_full_lines() {
        let query = "saFe, fAst, prodUctive.";
        let content = "\
Rust:
safe, fast, productive.
Pick three.
It's up to you.
But I love rust!";

        let params = AdditionalParameters {
            max_output: Some(10u32),
            invert_match: false
        };

        assert_eq!(vec![(2, "safe, fast, productive.".red().to_string())],
                   search_lines_case_insensitive(&query, &content, &params));
    }

    #[test]
    fn case_sensitive_full_lines() {
        let query = "safe, fast, productive.";
        let content = "\
Rust:
safe, fast, productive.
Pick three.
It's up to you.
But I love rust!";

        let params = AdditionalParameters {
            max_output: Some(10u32),
            invert_match: false
        };

        assert_eq!(vec![(2, query.red().to_string())],
                   search_lines_case_sensitive(&query, &content, &params));
    }
}