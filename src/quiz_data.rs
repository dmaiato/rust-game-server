
use crate::models::Question;
use std::fs::File;
use std::io::{self, BufReader, BufRead};
use rand::seq::SliceRandom;
use rand::thread_rng;

pub fn load_and_shuffle_questions(filename: &str) -> io::Result<Vec<Question>> {
    let file: File = File::open(filename)?;
    let reader: BufReader<File> = BufReader::new(file);
    let mut questions: Vec<Question> = Vec::new();

    for line_result in reader.lines() {
        let line: String = line_result?;
        
        if line.trim().is_empty() || line.trim().starts_with("#") {
            continue;
        }

        let parts: Vec<&str> = line.split('|').collect();

        if parts.len() >= 5 {
            let correct_char: char = parts[4].chars().next().unwrap_or(' ');
            let options_str: String = format!("{} | {} | {}", parts[1].trim(), parts[2].trim(), parts[3].trim());

            questions.push(Question {
                text: parts[0].trim().to_string(),
                options: options_str,
                correct_option: correct_char,
            });
        } else {
            eprintln!("Aviso: Linha mal formatada no arquivo de perguntas: '{}'", line);
        }
    }

    let mut rng = thread_rng();
    questions.shuffle(&mut rng);

    Ok(questions)
}