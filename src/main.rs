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

fn grep_files(pattern: &str, files: &[String], prefix: bool) {
    let mut match_count = 0;

    for file in files {
        if let Ok(lines) = read_lines(file) {
            for line in lines.map_while(Result::ok) {
                if match_pattern(&line, pattern) {
                    match_count += 1;

                    if match_count > 1 {
                        println!("");
                    }

                    if prefix {
                        print!("{0}:{1}", file, line);
                    } else {
                        print!("{}", line);
                    }
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
    let Some(pattern_flag_index) = env::args().position(|arg| arg == "-E") else {
        println!("Pattern argument '-E' is required");
        process::exit(1);
    };

    let pattern = env::args().nth(pattern_flag_index + 1).unwrap();

    let arg_count = env::args().len();
    let recursive_flag = match env::args().find(|arg| arg == "-r") {
        Some(_) => true,
        None => false,
    };

    if arg_count < 4 {
        grep_stdin(&pattern);
    } else if recursive_flag {
        let mut files = vec![];
        let directory = env::args().nth(4).unwrap();

        let walker = walkdir::WalkDir::new(directory);
        for file in walker.into_iter().filter_map(|e| e.ok()) {
            if file.file_type().is_file() {
                let path = file.path().display().to_string();
                files.push(path);
            }
        }

        grep_files(&pattern, &files, true);
    } else {
        let files: Vec<String> = env::args().skip(3).collect();
        grep_files(&pattern, &files, files.len() > 1);
    }
}
