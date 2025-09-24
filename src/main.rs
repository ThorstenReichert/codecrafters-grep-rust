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

fn read_lines(filename: &str) -> io::Result<io::Lines<io::BufReader<File>>>
{
    let file = File::open(filename)?;
    Ok(io::BufReader::new(file).lines())
}

fn grep_file(pattern: &str, file: &str) {
    if let Ok(lines) = read_lines(file) {
        let mut has_match = false;

        for line in lines.map_while(Result::ok) {
            println!("Matching Line '{}'", line);
            if match_pattern(&line, pattern) {
                if !has_match {
                    println!("");
                    has_match = true;
                }
                print!("{}", line);
            }
        }

        if has_match {
            process::exit(0);
        } else {
            process::exit(1);
        }
    } else {
        process::exit(-2);
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
    } else if env::args().len() == 4 {
        let file = env::args().nth(3).unwrap();
        grep_file(&pattern, &file);
    } else {
        process::exit(-1);
    }
}
