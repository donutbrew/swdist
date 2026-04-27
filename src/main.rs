use std::env;
use std::fs::File;
use std::io::{self, BufRead, BufReader};
use std::process;

use strsim::{
    damerau_levenshtein, hamming, jaro, jaro_winkler, levenshtein,
    normalized_damerau_levenshtein, normalized_levenshtein, osa_distance, sorensen_dice,
};
use unicode_normalization::UnicodeNormalization;

#[derive(Debug, Clone, Default)]
struct Config {
    metric: String,
    files: Option<(String, String)>,
    args: Vec<String>,
    show_line_numbers: bool,

    norm: bool,
    ascii: bool,
    strip: bool,
    alnum: bool,
    space: bool,
}

fn usage() -> ! {
    eprintln!(
        r#"Usage:
  swdist <metric> <string1> <string2> [options]
  swdist <metric> --files <file1> <file2> [options]

Metrics:
  hamming
  lev | levenshtein
  osa
  damerau | damerau_levenshtein
  jaro
  jw | jaro_winkler
  nlev | normalized_levenshtein
  ndamerau | normalized_damerau_levenshtein
  dice | sorensen_dice

Normalization:
  --norm     lowercase + strip + ascii-fold + alnum + collapse-whitespace
  --ascii    remove accents/diacritics (e.g. café -> cafe)
  --strip    trim leading/trailing whitespace
  --alnum    remove non-alphanumeric chars
  --space    collapse runs of whitespace to a single space

Other options:
  --files <file1> <file2>   compare corresponding lines
  --no-line-numbers         suppress line numbers in file mode

Examples:
  swdist lev kitten sitting
  swdist jw martha marhta
  swdist lev --files old.txt new.txt
  swdist nlev --files a.txt b.txt --norm
  swdist lev "café au lait" "cafeaulait" --norm
"#
    );
    process::exit(2);
}

fn parse_args() -> Config {
    let mut args = env::args().skip(1);

    let metric = args.next().unwrap_or_else(|| usage());

    let mut cfg = Config {
        metric,
        show_line_numbers: true,
        ..Default::default()
    };

    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--files" => {
                let f1 = args.next().unwrap_or_else(|| usage());
                let f2 = args.next().unwrap_or_else(|| usage());
                cfg.files = Some((f1, f2));
            }
            "--no-line-numbers" => cfg.show_line_numbers = false,
            "--norm" => cfg.norm = true,
            "--ascii" => cfg.ascii = true,
            "--strip" => cfg.strip = true,
            "--alnum" => cfg.alnum = true,
            "--space" => cfg.space = true,
            _ => cfg.args.push(arg),
        }
    }

    if cfg.files.is_none() && cfg.args.len() != 2 {
        usage();
    }

    if cfg.files.is_some() && !cfg.args.is_empty() {
        eprintln!("error: do not pass string arguments together with --files");
        process::exit(2);
    }

    cfg
}

fn ascii_fold(s: &str) -> String {
    s.nfd().filter(|c| c.is_ascii()).collect()
}

fn collapse_whitespace(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let mut prev_ws = false;

    for ch in s.chars() {
        if ch.is_whitespace() {
            if !prev_ws {
                out.push(' ');
                prev_ws = true;
            }
        } else {
            out.push(ch);
            prev_ws = false;
        }
    }

    out
}

fn normalize(mut s: String, cfg: &Config) -> String {
    if cfg.norm {
        s = s.to_lowercase();
        s = s.trim().to_string();
        s = ascii_fold(&s);
        s = s.chars().filter(|c| c.is_alphanumeric()).collect();
        s = collapse_whitespace(&s);
        return s;
    }

    if cfg.strip {
        s = s.trim().to_string();
    }

    if cfg.ascii {
        s = ascii_fold(&s);
    }

    if cfg.alnum {
        s = s.chars().filter(|c| c.is_alphanumeric()).collect();
    }

    if cfg.space {
        s = collapse_whitespace(&s);
    }

    s
}

fn compare(metric: &str, a: &str, b: &str) -> Result<String, String> {
    match metric {
        "hamming" => hamming(a, b)
            .map(|v| v.to_string())
            .map_err(|e| e.to_string()),

        "lev" | "levenshtein" => Ok(levenshtein(a, b).to_string()),

        "osa" => Ok(osa_distance(a, b).to_string()),

        "damerau" | "damerau_levenshtein" => Ok(damerau_levenshtein(a, b).to_string()),

        "jaro" => Ok(jaro(a, b).to_string()),

        "jw" | "jaro_winkler" => Ok(jaro_winkler(a, b).to_string()),

        "nlev" | "normalized_levenshtein" => Ok(normalized_levenshtein(a, b).to_string()),

        "ndamerau" | "normalized_damerau_levenshtein" => {
            Ok(normalized_damerau_levenshtein(a, b).to_string())
        }

        "dice" | "sorensen_dice" => Ok(sorensen_dice(a, b).to_string()),

        _ => Err(format!("unknown metric '{metric}'")),
    }
}

fn open_lines(path: &str) -> io::Result<io::Lines<BufReader<File>>> {
    let f = File::open(path)?;
    Ok(BufReader::new(f).lines())
}

fn run_file_mode(cfg: &Config, f1: &str, f2: &str) -> io::Result<i32> {
    let mut left = open_lines(f1)?;
    let mut right = open_lines(f2)?;
    let mut line_no: usize = 0;

    loop {
        let l1 = left.next().transpose()?;
        let l2 = right.next().transpose()?;

        if l1.is_none() && l2.is_none() {
            break;
        }

        line_no += 1;

        let a = normalize(l1.unwrap_or_default(), cfg);
        let b = normalize(l2.unwrap_or_default(), cfg);

        match compare(&cfg.metric, &a, &b) {
            Ok(result) => {
                if cfg.show_line_numbers {
                    println!("{}\t{}", line_no, result);
                } else {
                    println!("{}", result);
                }
            }
            Err(err) => {
                if cfg.show_line_numbers {
                    println!("{}\tERR\t{}", line_no, err);
                } else {
                    println!("ERR\t{}", err);
                }
            }
        }
    }

    Ok(0)
}

fn main() {
    let cfg = parse_args();

    if let Some((f1, f2)) = &cfg.files {
        match run_file_mode(&cfg, f1, f2) {
            Ok(code) => process::exit(code),
            Err(err) => {
                eprintln!("error: {}", err);
                process::exit(1);
            }
        }
    }

    let a = normalize(cfg.args[0].clone(), &cfg);
    let b = normalize(cfg.args[1].clone(), &cfg);

    match compare(&cfg.metric, &a, &b) {
        Ok(result) => println!("{}", result),
        Err(err) => {
            eprintln!("error: {}", err);
            process::exit(1);
        }
    }
}
