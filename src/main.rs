extern crate walkdir;
use walkdir::WalkDir;
use std::path::Path;
use std::fs;
use std::path::PathBuf;
use indicatif::ProgressBar;
use std::env::args;
use uuid::Uuid;


fn change_file_name(path: impl AsRef<Path>, name: &str) -> PathBuf {
    let path = path.as_ref();
    let mut result = path.to_owned();
    result.set_file_name(name);
    if let Some(ext) = path.extension() {
        result.set_extension(ext);
    }
    result
}

fn main() {
    let mut total_skipped = 0;
    let mut total_copied = 0;
    let mut total_files = 0;
    let mut total_bytes: u64 = 0;
    let mut source: String = "".to_owned();
    let mut target: String = "".to_owned();
    let mut out_dir: PathBuf = PathBuf::new();
    if args().count() < 2 {
        println!("Usage: ojen <SOURCE> [TARGET]

This program copies files from SOURCE to TARGET while renaming the destination
file names using random UUID names.

Options:
    SOURCE      Source path to scan for files.
    TARGET      Optional. Target folder to copy files.
");
        return
    }
    if args().count() >= 2 {
        source = args().nth(1).unwrap();
        println!("Scan source: \t\"{}\"", source);
        if !Path::new(source.as_str()).is_dir() {
            println!("Error: Source path \"{}\" does not exist!", source);
            return
        }
    }
    if args().count() >= 3 {
        target = args().nth(2).unwrap();
        println!("Target path: \t\"{}\"", target);
        if !Path::new(target.as_str()).is_dir() {
            println!("Error: Target path \"{}\" does not exist!", target);
            return
        }
        out_dir = PathBuf::from(target.clone());
    }
    let in_dir = PathBuf::from(source);
    let max_files_count = WalkDir::new(&in_dir).into_iter().filter_map(|file| file.ok()).count();
    let pb = ProgressBar::new(max_files_count.try_into().unwrap());
    for file in WalkDir::new(&in_dir).into_iter().filter_map(|file| file.ok()) { if file.metadata().unwrap().is_file() {
        total_files += 1;
        pb.inc(1);
        let from = file.path();
        let mut to = out_dir.clone();
        let mut j = 0;
        to = change_file_name(to.join(file.path().file_name().unwrap()), Uuid::new_v4().to_string().as_str());
        while std::path::Path::new(to.join(file.path().file_name().unwrap()).as_os_str()).exists() {
            j += 1;
            to = change_file_name(to, Uuid::new_v4().to_string().as_str());
            if j > 10 {
                println!("warning: all generated UUID filenames for this source already exists: {}", to.to_string_lossy());
                break;
            }
        }
        let to_dir = to.parent().expect("target path must be in some directory");
        if !Path::new(to_dir).is_dir() {
            fs::create_dir_all(to_dir).expect("destination path creation failed");
        }
        if std::path::Path::new(to.as_os_str()).exists() {
            total_skipped += 1;
        } else {
            if target.is_empty() {
                println!("Would copy {} to {}", file.file_name().to_string_lossy(), to.to_string_lossy());
                // report only
                continue
            }
            let result = fs::copy(from,to.clone());
            match result {
                Ok(bytes_copied) => {
                    total_copied += 1;
                    total_bytes += bytes_copied
                },
                Err(e) => println!("Error: {}", e),
            }
        }
    } }
    pb.finish_with_message("done");
    println!("total of {} files, {} skipped, {} copied, {} bytes", total_files, total_skipped, total_copied, total_bytes);
}
