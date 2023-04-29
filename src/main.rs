use std::collections::HashMap;
use std::fs;
use std::io::{Error, ErrorKind};
use std::path::{Path, PathBuf};
use std::process::exit;
use std::sync::Mutex;

use clap::Parser;
use convert_case::{Case, Casing};
use rayon::prelude::*;
use regex::{escape, Regex};

fn generate_aliases(
    target_dirs: &Path,
) -> Result<HashMap<String, PathBuf>, Error> {
    if !target_dirs.is_dir() {
        Error::new(ErrorKind::NotFound, "not a directory");
    }

    let mut entries: HashMap<String, PathBuf> = HashMap::new();

    for entry in fs::read_dir(target_dirs)? {
        let entry = entry?;

        if !entry.path().is_dir() {
            continue;
        }

        let name = entry.path().file_name().expect("")
            .to_str().expect("")
            .to_string()
            .to_lowercase();

        if name.contains(",") {
            // Split with ","
            for n in name.split(",") {
                let s = n.trim().to_string();

                if entries.contains_key(&*s) {
                    Error::new(ErrorKind::AlreadyExists, "exists");
                }

                entries.insert(s, entry.path());
            }
        } else {
            // Use whole directory name
            if entries.contains_key(&*name) {
                Error::new(ErrorKind::AlreadyExists, "exists");
            }

            entries.insert(name, entry.path());
        }
    }

    Ok(entries)
}

// Sort keys by length
fn sort_keys(
    entries: HashMap<String, PathBuf>,
) -> Result<Vec<String>, Error> {
    let mut keys: Vec<String> = entries.keys()
        .map(|x|
            x.to_string()
        ).collect();

    keys.sort_by(|a, b|
        b.len().cmp(&a.len())
    );

    Ok(keys)
}

fn trim_str(mut s: String) -> String {
    let remove: &[_] = &['_', '.', '-', ' '];
    s = s.trim_matches(remove).to_string();
    s
}

// Search candidates to be sorted
fn search_candidates(
    entries: HashMap<String, PathBuf>,
    sources: Vec<PathBuf>,
) -> Result<
    (
        // Found
        HashMap<PathBuf, String>,
        // Multiple matches
        Vec<PathBuf>
    ), Error> {
    if sources.is_empty() {
        Error::new(ErrorKind::NotFound, "no sources");
    }

    let mut multiple_matches: Vec<PathBuf> = Vec::new();
    let mut candidates: HashMap<PathBuf, String> = HashMap::new();

    let aliases = sort_keys(entries).expect("");
    let mut alias_re: HashMap<String, Regex> = HashMap::new();

    for alias in aliases.to_owned() {
        // Must have boundary
        let escaped = format!(r"\b{}\b", escape(&alias));
        alias_re.insert(alias, Regex::new(&escaped).unwrap());
    }

    for dir in sources {
        if !dir.is_dir() {
            continue;
        }

        for entry in fs::read_dir(dir)? {
            let entry = entry?;

            if entry.path().is_dir() {
                continue;
            }

            let name = PathBuf::from(
                entry.path().file_name().unwrap()
            );

            let mut modified: String = name.file_stem().unwrap().to_str().unwrap().to_string();

            modified = trim_str(modified);
            modified = modified.replace(".", " ");
            modified = modified.to_case(Case::Lower);

            if modified.is_empty() {
                continue;
            }

            let alias_matches_lock = Mutex::new(Vec::new());

            aliases.clone()
                .par_iter()
                .for_each(|alias| {
                    if alias_re[alias].is_match(&modified) {
                        let a = alias.clone();
                        alias_matches_lock.lock().unwrap().push(a.to_string());
                    }
                });

            let alias_matches: Vec<String> = alias_matches_lock.lock().unwrap().to_vec();

            if alias_matches.is_empty() {
                // No matches
                continue;
            } else if alias_matches.len() > 1 {
                // Multiple matches, add to a list
                multiple_matches.push(entry.path());
                continue;
            }

            candidates.insert(entry.path(), alias_matches[0].clone());
        }
    }

    Ok((candidates, multiple_matches))
}

// Add (N) suffix to path if we have existing dir/file
// Bugs:
// - Path as a whole might be too long (example: ISO 9660)
// - File name might be too long
fn rename_destination(
    source_path: PathBuf, // What we're moving
    target_dir: PathBuf, // To where
) -> Result<PathBuf, Error> {
    if !target_dir.is_dir() {
        Error::new(ErrorKind::InvalidInput, "not dir");
    }

    // Target name, with possible rename(s), see loop below
    let mut new_path = PathBuf::from(target_dir.clone());
    new_path = new_path.join(source_path.file_name().unwrap());

    let extension = new_path.clone().extension().unwrap().to_owned();

    // Capture (N) suffix "example file (1).dat"
    let re_suffix = Regex::new(r" \((\d+)\)$").unwrap();

    loop {
        // Loop until we have a new file name

        if !new_path.exists() {
            // Move file/dir
            break;
        }

        if !re_suffix.is_match(new_path.file_stem().unwrap().to_str().unwrap()) {
            // Add " (1)" suffix
            let fname = PathBuf::from(
                format!("{} (1).{}",
                        new_path.file_stem().unwrap().to_str().unwrap(),
                        extension.clone().to_str().unwrap()
                )
            );

            new_path = PathBuf::from(target_dir.clone());
            new_path = new_path.join(fname.clone());
        } else {
            // Increase "(1)" suffix to "(2)"

            // Get suffix number from file name
            let fname = new_path.file_stem().unwrap();
            let m = re_suffix.captures(fname.to_str().unwrap()).unwrap();
            let num: u64 = m.get(1).map(|x|
                x.as_str().parse().unwrap()
            ).unwrap();

            // Position where suffix begins
            let start = m.get(1).unwrap().start();

            // Remove old suffix
            let new_fname = &new_path.file_stem().unwrap().to_str().unwrap()[0..start - 2];

            // Create new file name with new suffix
            let fname = PathBuf::from(
                format!("{} ({}).{}",
                        new_fname,
                        num + 1,
                        extension.clone().to_str().unwrap()
                )
            );

            new_path = PathBuf::from(target_dir.clone());
            new_path = new_path.join(fname.clone());
        }
    }

    Ok(new_path)
}

// CLI arguments
// See: https://docs.rs/clap/latest/clap/
#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct CLIArgs {
    #[clap(short = 't', long, required = true,
    help = "Target directory for sorted files")]
    target: PathBuf,

    #[clap(short = 'Y', long, help = "Move files? If enabled, files are actually moved")]
    move_files: bool,

    #[clap(
    help = "Path(s) to scan for files to be sorted",
    required = true)]
    paths: Vec<PathBuf>,
}

fn main() -> Result<(), Error> {
    let args: CLIArgs = CLIArgs::parse();

    if !args.target.is_dir() {
        eprintln!("not a directory: {}", args.target.display());
        exit(1);
    }

    if args.paths.is_empty() {
        eprintln!("no directories given to be scanned");
        exit(1);
    }

    // Only directories as sources
    for p in args.paths.clone() {
        if !p.is_dir() {
            eprintln!("not a directory: {}", p.display());
            exit(1);
        }
    }

    println!("Using {} as sorting target directory", args.target.display());

    let aliases = generate_aliases(&args.target)?;

    if aliases.is_empty() {
        eprintln!("target directory {} is empty?", args.target.display());
        exit(1);
    }

    println!("Finding matches...");

    let (candidates, multiple_matches) = search_candidates(aliases.clone(), args.paths)?;

    if !candidates.is_empty() {
        println!("Matches:");

        for (candidate, alias) in candidates.clone() {
            let target_dir = aliases[&alias.clone()].to_owned();
            let new_path = rename_destination(candidate.clone(), target_dir)?;

            if args.move_files {
                // Actually move
                match fs::rename(candidate.clone(), new_path.clone()) {
                    Ok(()) => {
                        println!("Moved {} to {}", candidate.display(), new_path.display());
                    }
                    Err(e) => eprintln!("error: {:?}", e),
                }
            } else {
                println!("Not moving {} to {}", candidate.display(), new_path.display());
            }
        }
    }

    if !multiple_matches.is_empty() {
        println!("Multiple matches (not moved):");

        for p in multiple_matches {
            println!("{}", p.display())
        }
    }

    Ok(())
}
