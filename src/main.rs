use ai_image_cleaner::batch;
use ai_image_cleaner::cli::Cli;
use clap::Parser;
use std::process;

fn main() {
    let cli = Cli::parse();

    if cli.in_place && cli.output.is_some() {
        eprintln!("Error: --in-place and --output flags cannot be used together.");
        process::exit(1);
    }

    if cli.dry_run {
        println!("🔍 Running in DRY-RUN mode (inspecting metadata only)...");
    }

    match batch::run_batch(&cli) {
        Ok(stats) => {
            println!("\n==========================================");
            println!("  AI Image Cleaner — Processing Summary  ");
            println!("==========================================");
            println!("Total files scanned:      {}", stats.total_scanned);
            println!("Successfully processed:   {}", stats.successfully_cleaned);
            println!("Files with AI metadata:   {}", stats.metadata_detected);
            if stats.failed > 0 {
                println!("Failed files:             {}", stats.failed);
            }
            if !cli.dry_run && stats.original_bytes > 0 {
                let diff = stats.original_bytes as i64 - stats.cleaned_bytes as i64;
                println!(
                    "Size before / after:      {:.2} MB -> {:.2} MB (saved {:.2} KB)",
                    stats.original_bytes as f64 / 1_048_576.0,
                    stats.cleaned_bytes as f64 / 1_048_576.0,
                    diff as f64 / 1024.0
                );
            }
            println!(
                "Elapsed time:             {:.2}s",
                stats.elapsed_ms as f64 / 1000.0
            );
            println!("==========================================");

            if stats.failed > 0 {
                process::exit(2);
            }
        }
        Err(e) => {
            eprintln!("Fatal error: {}", e);
            process::exit(1);
        }
    }
}
