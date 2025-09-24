use std::env;
use std::fs::File;
use std::io::{self, BufRead};
use std::process;

mod grep;

use grep::match_pattern;

fn grep_stdin(pattern: &str) -> i32 {
    let mut input_line = String::new();

    io::stdin().read_line(&mut input_line).unwrap();

    // Uncomment this block to pass the first stage
    if match_pattern(&input_line, &pattern) {
        process::exit(0);
    } else {
        process::exit(1);
    }
}

fn read_lines(filename: &str) -> io::Result<io::Lines<io::BufReader<File>>> {
    let file = File::open(filename)?;
    Ok(io::BufReader::new(file).lines())
}

fn grep_files(pattern: &str, files: &[String]) {
    let mut match_count = 0;
    
    for file in files {
        if let Ok(lines) = read_lines(file) {
            for line in lines.map_while(Result::ok) {
                if match_pattern(&line, pattern) {
                    match_count += 1;

                    if match_count > 1 {
                        println!("");
                    }

                    print!("{}", line);
                }
            }
        } else {
            process::exit(-2);
        }
    }

    if match_count > 0 {
        process::exit(0);
    } else {
        process::exit(1);
    }
}

// Usage: echo <input_text> | your_program.sh -E <pattern>
fn main() {
    if env::args().nth(1).unwrap() != "-E" {
        println!("Expected first argument to be '-E'");
        process::exit(1);
    }

    let pattern = env::args().nth(2).unwrap();

    if env::args().len() == 3 {
        grep_stdin(&pattern);
    } else if env::args().len() >= 4 {
        let files: Vec<String> = env::args().skip(3).collect();
        grep_files(&pattern, &files);
    } else {
        process::exit(-1);
    }
}
