use crate::cleaner::{CleanResult, clean_image_file};
use crate::cli::Cli;
use crate::error::CleanerError;
use rayon::prelude::*;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::Instant;
use walkdir::WalkDir;

#[derive(Debug, Default)]
pub struct BatchStats {
    pub total_scanned: usize,
    pub successfully_cleaned: usize,
    #[allow(dead_code)]
    pub skipped: usize,
    pub failed: usize,
    pub metadata_detected: usize,
    pub original_bytes: u64,
    pub cleaned_bytes: u64,
    pub elapsed_ms: u128,
}

pub fn is_supported_image(path: &Path) -> bool {
    if let Some(ext) = path.extension().and_then(|s| s.to_str()) {
        matches!(
            ext.to_ascii_lowercase().as_str(),
            "png" | "jpg" | "jpeg" | "webp" | "bmp" | "tiff"
        )
    } else {
        false
    }
}

pub fn collect_files(input: &Path, recursive: bool) -> Result<Vec<PathBuf>, CleanerError> {
    if input.is_file() {
        return Ok(vec![input.to_path_buf()]);
    }

    if !input.is_dir() {
        return Err(CleanerError::Io {
            path: input.to_path_buf(),
            source: std::io::Error::new(
                std::io::ErrorKind::NotFound,
                format!("Input path {:?} does not exist", input),
            ),
        });
    }

    let mut files = Vec::new();
    let walker = if recursive {
        WalkDir::new(input)
    } else {
        WalkDir::new(input).max_depth(1)
    };

    for entry in walker.into_iter().filter_map(|e| e.ok()) {
        let path = entry.path();
        if path.is_file() && is_supported_image(path) {
            files.push(path.to_path_buf());
        }
    }

    Ok(files)
}

pub fn determine_output_path(
    source_file: &Path,
    input_root: &Path,
    output_arg: Option<&Path>,
    in_place: bool,
) -> Result<PathBuf, CleanerError> {
    if in_place {
        return Ok(source_file.to_path_buf());
    }

    let default_output_dir = Path::new("cleaned");
    let target_dir = output_arg.unwrap_or(default_output_dir);

    if input_root.is_file() {
        if output_arg.is_some_and(|p| p.extension().is_some()) {
            // output was explicitly given as a file path
            return Ok(target_dir.to_path_buf());
        }
        let file_name = source_file
            .file_name()
            .ok_or_else(|| CleanerError::InvalidArgument("Invalid source file name".into()))?;
        return Ok(target_dir.join(file_name));
    }

    // Input is a directory: preserve relative path
    let rel_path = source_file.strip_prefix(input_root).unwrap_or(source_file);
    Ok(target_dir.join(rel_path))
}

pub fn run_batch(cli: &Cli) -> Result<BatchStats, CleanerError> {
    let start_time = Instant::now();

    // Configure Rayon thread pool if specified
    if let Some(threads) = cli.threads {
        let _ = rayon::ThreadPoolBuilder::new()
            .num_threads(threads)
            .build_global();
    }

    let files = collect_files(&cli.input, cli.recursive)?;
    let total_scanned = files.len();

    if total_scanned == 0 {
        return Ok(BatchStats {
            total_scanned: 0,
            elapsed_ms: start_time.elapsed().as_millis(),
            ..Default::default()
        });
    }

    let success_count = AtomicUsize::new(0);
    let fail_count = AtomicUsize::new(0);
    let meta_detected_count = AtomicUsize::new(0);

    let results: Vec<Result<CleanResult, (PathBuf, CleanerError)>> = files
        .par_iter()
        .map(|file| {
            let out_path = match determine_output_path(
                file,
                &cli.input,
                cli.output.as_deref(),
                cli.in_place,
            ) {
                Ok(p) => p,
                Err(e) => return Err((file.clone(), e)),
            };

            match clean_image_file(file, &out_path, cli.deep, cli.strength, cli.dry_run) {
                Ok(res) => {
                    success_count.fetch_add(1, Ordering::Relaxed);
                    if res.metadata_found.has_any() {
                        meta_detected_count.fetch_add(1, Ordering::Relaxed);
                    }
                    if cli.verbose {
                        let tags = if res.metadata_found.has_any() {
                            format!(
                                " [AI metadata stripped: {}]",
                                res.metadata_found.detected_tags.join(", ")
                            )
                        } else {
                            " [Clean, no AI chunks]".to_string()
                        };
                        let deep_str = if res.deep_applied {
                            " (Deep mode applied)"
                        } else {
                            ""
                        };
                        println!("Cleaned: {:?} -> {:?}{}{}", file, out_path, tags, deep_str);
                    }
                    Ok(res)
                }
                Err(e) => {
                    fail_count.fetch_add(1, Ordering::Relaxed);
                    eprintln!("Error cleaning {:?}: {}", file, e);
                    Err((file.clone(), e))
                }
            }
        })
        .collect();

    let mut total_original_bytes = 0u64;
    let mut total_cleaned_bytes = 0u64;

    for res in results.into_iter().flatten() {
        total_original_bytes += res.original_size;
        total_cleaned_bytes += res.cleaned_size;
    }

    Ok(BatchStats {
        total_scanned,
        successfully_cleaned: success_count.load(Ordering::Relaxed),
        skipped: 0,
        failed: fail_count.load(Ordering::Relaxed),
        metadata_detected: meta_detected_count.load(Ordering::Relaxed),
        original_bytes: total_original_bytes,
        cleaned_bytes: total_cleaned_bytes,
        elapsed_ms: start_time.elapsed().as_millis(),
    })
}
