extern crate reqwest;
extern crate scraper;
use std::collections::{HashMap, HashSet};
use std::fmt;
use std::time::Instant;

use scraper::{Html, Selector};

fn filter_by_incorrect_words(words: Vec<String>, incorrect_words: Vec<String>) -> Vec<String> {
    let mut response: Vec<String> = Vec::new();
    let mut incorrect_letters: HashSet<String> = HashSet::new();

    for i_words in incorrect_words {
        let mut word_split: Vec<&str> = i_words.trim().split("").collect();
        word_split.retain(|&s| s != String::from(""));
        for i_letter in word_split {
            incorrect_letters.insert(i_letter.to_string());
        }
    }

    let incorrect_letters_tuple = incorrect_letters
        .iter()
        .map(|v| (v, false))
        .collect::<Vec<_>>();

    let mut can_show = HashMap::new();

    for (k, v) in incorrect_letters_tuple {
        can_show.insert(k, v);
    }

    for word in words {
        for i_letter in &incorrect_letters {
            if !word.contains(i_letter) {
                can_show.insert(i_letter, true);
            }
        }

        if can_show.iter().all(|(_, can)| can == &true) {
            response.push(word);
        }

        for val in can_show.values_mut() {
            *val = false;
        }
    }
    response
}

fn filter_by_letter(words: Vec<String>, letter: &str) -> Vec<String> {
    let mut response: Vec<String> = Vec::new();
    for word in words {
        let mut word_split: Vec<&str> = word.split("").collect();
        word_split.retain(|&s| s != String::from(""));
        if word_split.contains(&letter) {
            response.push(word)
        }
    }
    response
}

fn includes_letters(words: Vec<String>, letters: Vec<String>) -> Vec<String> {
    let mut response: Vec<String> = Vec::new();
    let mut checks = vec![false; letters.len()];
    for word in words {
        for (index, letter) in letters.iter().enumerate() {
            if word.contains(letter) {
                checks[index] = true
            }
        }
        if checks.iter().all(|&x| x == true) {
            response.push(word)
        }
        checks = vec![false; letters.len()];
    }
    response
}

fn starts_with(words: Vec<String>, letter: &str) -> Vec<String> {
    let mut response: Vec<String> = Vec::new();
    for word in words {
        if word.starts_with(letter) {
            response.push(word)
        }
    }
    response
}

fn ends_with(words: Vec<String>, letter: &str) -> Vec<String> {
    let mut response: Vec<String> = Vec::new();
    for word in words {
        if word.ends_with(letter) {
            response.push(word)
        }
    }
    response
}

fn find_with_static_letters(words: Vec<String>, letters: Vec<String>) -> Vec<String> {
    let mut response: Vec<String> = Vec::new();
    let letter_to_look = letters.join("");
    for word in words {
        if word.contains(&letter_to_look) {
            response.push(word)
        }
    }
    response
}

fn find_with_pattern(words: Vec<String>, letters: Vec<String>) -> Vec<String> {
    let mut response: Vec<String> = Vec::new();
    let letters_to_look = letters.join("");
    let mut match_pattern_arr = [false; 5];
    for word in words {
        for (i, l) in word.chars().enumerate() {
            let empty_pattern = "_";

            match letters_to_look.chars().nth(i) {
                Some(value) => {
                    if value == empty_pattern.chars().next().expect("string is empty") || value == l
                    {
                        match_pattern_arr[i] = true;
                    }
                }
                None => continue,
            }
        }

        if match_pattern_arr.iter().all(|v| *v) {
            response.push(word)
        }
        match_pattern_arr = [false; 5]
    }
    response
}

struct QueryObject {
    url: String,
    selector: String,
}

async fn query_web_scrapping(
    query_values: &QueryObject,
    prefix: String,
) -> Result<Vec<String>, Box<dyn std::error::Error>> {
    let mut word_list: Vec<String> = Vec::new();
    let new_url = format!("{}/{}", query_values.url, prefix);

    let res = reqwest::get(new_url).await?;
    let body = res.text().await?;
    let fragment = Html::parse_document(&body);
    let stories = Selector::parse(&query_values.selector).unwrap();
    for words in fragment.select(&stories) {
        let words_txt = words.text().collect::<String>();
        word_list.push(words_txt);
    }

    Ok(word_list)
}

async fn get_words(lang: Langs) -> Result<Vec<String>, Box<dyn std::error::Error>> {
    let query_values = match lang {
        Langs::En => QueryObject {
            url: String::from("https://wordfind.com/length/5-letter-words"),
            selector: String::from("li.dl>a"),
        },
        Langs::Es => QueryObject {
            url: String::from("https://muchaspalabras.com/5-letras/diccionario"),
            selector: String::from("ul.inline-list.words.group0.sort>li>a"),
        },
    };
    let mut word_list: Vec<String> = Vec::new();

    match lang {
        Langs::Es => {
            for query in 1..40 {
                match query_web_scrapping(&query_values, query.to_string()).await {
                    Ok(value) => {
                        for v in value {
                            word_list.push(v);
                        }
                    }
                    Err(_) => continue,
                };
            }
            Ok(word_list)
        }
        Langs::En => {
            match query_web_scrapping(&query_values, String::new()).await {
                Ok(value) => word_list = value,
                Err(_) => {}
            };
            Ok(word_list)
        }
    }
}

async fn query_words(
    input: WordleCLI,
    wordle_filtered: Vec<String>,
) -> Result<Vec<String>, Box<dyn std::error::Error>> {
    let word_list: Vec<String>;
    if wordle_filtered.len() == 0 {
        word_list = match get_words(input.lang).await {
            Ok(value) => value,
            Err(_) => vec![String::new()],
        }
    } else {
        word_list = wordle_filtered
    }

    let response: Vec<String>;

    match input.action {
        Actions::FirstLetter => {
            response = starts_with(word_list, &input.clean_input[0]);
        }
        Actions::LastLetter => {
            response = ends_with(word_list, &input.clean_input[0]);
        }
        Actions::Contain => {
            response = filter_by_letter(word_list, &input.clean_input[0]);
        }
        Actions::Contains => {
            response = includes_letters(word_list, input.clean_input);
        }
        Actions::IncorrectWords => {
            response = filter_by_incorrect_words(word_list, input.clean_input);
        }
        Actions::StaticLetters => {
            response = find_with_static_letters(word_list, input.clean_input);
        }
        Actions::Pattern => {
            response = find_with_pattern(word_list, input.clean_input);
        }
        Actions::NoAction => response = vec![String::from("no action required")],
    }

    Ok(response)
}

enum Actions {
    FirstLetter,
    LastLetter,
    Contains,
    Contain,
    IncorrectWords,
    StaticLetters,
    Pattern,
    NoAction,
}

struct WordleCLI<'l> {
    action: Actions,
    clean_input: Vec<String>,
    lang: &'l Langs,
}

impl WordleCLI<'l, T> {
    pub fn new(action: Actions, raw_input: &String, lang: &'l Langs) -> Self {
        WordleCLI {
            clean_input: WordleCLI::clean_raw_input(&action, String::from(raw_input)),
            action,
            lang,
        }
    }

    fn clean_raw_input(action: &Actions, raw_input: String) -> Vec<String> {
        match action {
            Actions::FirstLetter => vec![raw_input],
            Actions::LastLetter => vec![raw_input],
            Actions::Contain => vec![raw_input],
            Actions::StaticLetters => vec![raw_input],
            Actions::Contains => {
                let values: Vec<String> = raw_input.split(",").map(|val| val.to_string()).collect();
                values
            }
            Actions::IncorrectWords => {
                let values: Vec<String> = raw_input.split(",").map(|val| val.to_string()).collect();
                values
            }
            Actions::Pattern => {
                let mut values: Vec<String> =
                    raw_input.split("").map(|val| val.to_string()).collect();
                values.retain(|s| s != &String::from(""));
                values
            }
            Actions::NoAction => vec![raw_input],
        }
    }
}

impl fmt::Display for Actions {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self {
            Actions::LastLetter => write!(f, "Last letter <a>"),
            Actions::IncorrectWords => write!(f, "Incorrect Words <house,dogs>"),
            Actions::FirstLetter => write!(f, "First letter <a>"),
            Actions::Contains => write!(f, "Contains letters <a,b,c>"),
            Actions::Contain => write!(f, "Contain letter <a>"),
            Actions::StaticLetters => write!(f, "Contain Static Letters <amb>"),
            Actions::Pattern => write!(f, "Check word with pattern <_rom_>"),
            Actions::NoAction => write!(f, "No Action required"),
        }
    }
}

impl fmt::Display for WordleCLI {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "lang: {} | {}, {:?}",
            &self.lang, &self.action, &self.clean_input
        )
    }
}

enum Langs {
    Es,
    En,
}

impl fmt::Display for Langs {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self {
            Langs::Es => write!(f, "Español"),
            Langs::En => write!(f, "English"),
        }
    }
}

#[tokio::main]
async fn main() {
    let start = Instant::now();
    let lang_input = std::env::args().nth(1).expect("no lang given");
    let action = std::env::args().nth(2).expect("no action given");
    let input = std::env::args().nth(3).expect("no input given");

    let mut wordle_obj: WordleCLI;
    let lang: Langs = match lang_input.as_str() {
        "es" => Langs::Es,
        "en" => Langs::En,
        _ => Langs::En,
    };

    let first_time = true;

    loop {
        match action.as_str() {
            "firstLetter" => wordle_obj = WordleCLI::new(Actions::FirstLetter, &input, &lang),
            "lastLetter" => wordle_obj = WordleCLI::new(Actions::LastLetter, input, lang),
            "containsLetters" => wordle_obj = WordleCLI::new(Actions::Contains, input, lang),
            "containSingleLetter" => wordle_obj = WordleCLI::new(Actions::Contain, input, lang),
            "filterByIncorrectWords" => {
                wordle_obj = WordleCLI::new(Actions::IncorrectWords, input, lang)
            }
            "pattern" => wordle_obj = WordleCLI::new(Actions::Pattern, input, lang),
            "staticLetters" => wordle_obj = WordleCLI::new(Actions::StaticLetters, input, lang),
            _ => wordle_obj = WordleCLI::new(Actions::NoAction, String::from(" :( "), lang),
        };
        println!("{}", wordle_obj);
        match query_words(wordle_obj, vec![]).await {
            Ok(response) => {
                println!("{:?}", response)
            }
            Err(_) => {
                println!("error")
            }
        }
        let duration = start.elapsed();
        println!("Time elapsed {:?}", duration);
        first_time = false;
    }
}
