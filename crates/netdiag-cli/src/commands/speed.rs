//! Speed test command implementation.

use crate::app::SpeedArgs;
use color_eyre::eyre::Result;
use console::style;
use indicatif::{ProgressBar, ProgressStyle};
use netdiag_speed::{
    HttpSpeedTest, IperfClient, SpeedTestConfig, SpeedTestProvider, SpeedTestResult,
};
use std::time::Duration;

/// Run the speed test command.
pub async fn run(args: SpeedArgs) -> Result<()> {
    println!("{}", style("Speed Test").bold().underlined());
    println!();

    let config = SpeedTestConfig {
        duration: Duration::from_secs(args.duration),
        connections: args.connections,
        server: args.server.clone(),
        warmup: Duration::from_secs(2),
        test_download: !args.upload_only,
        test_upload: !args.download_only,
    };

    // Determine provider
    let result = if let Some(iperf_server) = &args.iperf {
        println!(
            "Using {} server: {}",
            style("iPerf3").cyan(),
            style(iperf_server).yellow()
        );
        run_iperf_test(iperf_server, &config).await?
    } else {
        println!("Using {} speed test", style("HTTP").cyan());
        if let Some(server) = &args.server {
            println!("Server: {}", style(server).yellow());
        } else {
            println!("{}", style("Selecting best server...").dim());
        }
        run_http_test(&config).await?
    };

    println!();
    display_results(&result);

    Ok(())
}

/// Run HTTP-based speed test.
async fn run_http_test(config: &SpeedTestConfig) -> Result<SpeedTestResult> {
    let provider = HttpSpeedTest::new();

    // Check availability
    if !provider.is_available().await {
        return Err(color_eyre::eyre::eyre!(
            "HTTP speed test servers not reachable"
        ));
    }

    println!();

    // Measure latency first
    println!("{}", style("Measuring latency...").dim());
    match provider.measure_latency().await {
        Ok(latency) => {
            println!(
                "  {} {:.1} ms",
                style("Latency:").bold(),
                latency.as_secs_f64() * 1000.0
            );
        }
        Err(e) => {
            println!(
                "  {} {}",
                style("Latency:").yellow(),
                style(format!("Error: {}", e)).dim()
            );
        }
    }

    // Download test
    if config.test_download {
        println!();
        println!("{}", style("Testing download speed...").dim());

        let pb = create_progress_bar(config.duration.as_secs());

        let download = provider.test_download(config).await;

        pb.finish_and_clear();

        match &download {
            Ok(measurement) => {
                println!(
                    "  {} {}",
                    style("Download:").bold(),
                    style(measurement.format_speed()).green()
                );
            }
            Err(e) => {
                println!(
                    "  {} {}",
                    style("Download:").red(),
                    format!("Error: {}", e)
                );
            }
        }
    }

    // Upload test
    if config.test_upload {
        println!();
        println!("{}", style("Testing upload speed...").dim());

        let pb = create_progress_bar(config.duration.as_secs());

        let upload = provider.test_upload(config).await;

        pb.finish_and_clear();

        match &upload {
            Ok(measurement) => {
                println!(
                    "  {} {}",
                    style("Upload:").bold(),
                    style(measurement.format_speed()).green()
                );
            }
            Err(e) => {
                println!(
                    "  {} {}",
                    style("Upload:").red(),
                    format!("Error: {}", e)
                );
            }
        }
    }

    // Run full test to get result structure
    provider
        .run_full_test(config)
        .await
        .map_err(|e| color_eyre::eyre::eyre!("{}", e))
}

/// Run iPerf3-based speed test.
async fn run_iperf_test(server: &str, config: &SpeedTestConfig) -> Result<SpeedTestResult> {
    let provider = IperfClient::new(server);

    if !provider.is_available().await {
        return Err(color_eyre::eyre::eyre!(
            "iPerf3 not available (ensure iperf3 is installed)"
        ));
    }

    println!();

    // Measure latency first
    println!("{}", style("Measuring latency...").dim());
    match provider.measure_latency().await {
        Ok(latency) => {
            println!(
                "  {} {:.1} ms",
                style("Latency:").bold(),
                latency.as_secs_f64() * 1000.0
            );
        }
        Err(e) => {
            println!(
                "  {} {}",
                style("Latency:").yellow(),
                style(format!("Error: {}", e)).dim()
            );
        }
    }

    // Download test (reverse mode)
    if config.test_download {
        println!();
        println!(
            "{}",
            style("Testing download speed (iPerf3 reverse mode)...").dim()
        );

        let pb = create_progress_bar(config.duration.as_secs());

        let download = provider.test_download(config).await;

        pb.finish_and_clear();

        match &download {
            Ok(measurement) => {
                println!(
                    "  {} {}",
                    style("Download:").bold(),
                    style(measurement.format_speed()).green()
                );
            }
            Err(e) => {
                println!(
                    "  {} {}",
                    style("Download:").red(),
                    format!("Error: {}", e)
                );
            }
        }
    }

    // Upload test
    if config.test_upload {
        println!();
        println!("{}", style("Testing upload speed...").dim());

        let pb = create_progress_bar(config.duration.as_secs());

        let upload = provider.test_upload(config).await;

        pb.finish_and_clear();

        match &upload {
            Ok(measurement) => {
                println!(
                    "  {} {}",
                    style("Upload:").bold(),
                    style(measurement.format_speed()).green()
                );
            }
            Err(e) => {
                println!(
                    "  {} {}",
                    style("Upload:").red(),
                    format!("Error: {}", e)
                );
            }
        }
    }

    provider
        .run_full_test(config)
        .await
        .map_err(|e| color_eyre::eyre::eyre!("{}", e))
}

/// Create a progress bar for the test duration.
fn create_progress_bar(duration_secs: u64) -> ProgressBar {
    let pb = ProgressBar::new(duration_secs);
    pb.set_style(
        ProgressStyle::default_bar()
            .template("  [{elapsed_precise}] {bar:40.cyan/blue} {pos}/{len}s")
            .unwrap()
            .progress_chars("█▓▒░"),
    );

    // Spawn a task to tick the progress bar
    let pb_clone = pb.clone();
    tokio::spawn(async move {
        for _ in 0..duration_secs {
            tokio::time::sleep(Duration::from_secs(1)).await;
            pb_clone.inc(1);
        }
    });

    pb
}

/// Display final results.
fn display_results(result: &SpeedTestResult) {
    println!();
    println!("{}", style("Results Summary").bold().underlined());
    println!();

    println!(
        "  {} {}",
        style("Server:").bold(),
        style(&result.server.name).cyan()
    );

    if let Some(location) = &result.server.location {
        if let Some(country) = &result.server.country {
            println!(
                "  {} {}, {}",
                style("Location:").bold(),
                location,
                country
            );
        } else {
            println!("  {} {}", style("Location:").bold(), location);
        }
    }

    println!("  {} {}", style("Provider:").bold(), result.provider);
    println!();

    if let Some(download) = &result.download {
        println!(
            "  {} {}",
            style("Download:").bold(),
            style(download.format_speed()).green().bold()
        );
        println!(
            "           {} transferred in {:?}",
            format_bytes(download.bytes),
            download.duration
        );
    }

    if let Some(upload) = &result.upload {
        println!(
            "  {} {}",
            style("Upload:").bold(),
            style(upload.format_speed()).green().bold()
        );
        println!(
            "           {} transferred in {:?}",
            format_bytes(upload.bytes),
            upload.duration
        );
    }

    if let Some(latency) = result.latency {
        println!(
            "  {} {:.2} ms",
            style("Latency:").bold(),
            latency.as_secs_f64() * 1000.0
        );
    }

    if let Some(jitter) = result.jitter {
        println!(
            "  {} {:.2} ms",
            style("Jitter:").bold(),
            jitter.as_secs_f64() * 1000.0
        );
    }

    println!();
    println!(
        "  {} {:?}",
        style("Test duration:").dim(),
        result.test_duration
    );
}

/// Format bytes to human-readable string.
fn format_bytes(bytes: u64) -> String {
    const KB: u64 = 1024;
    const MB: u64 = KB * 1024;
    const GB: u64 = MB * 1024;

    if bytes >= GB {
        format!("{:.2} GB", bytes as f64 / GB as f64)
    } else if bytes >= MB {
        format!("{:.2} MB", bytes as f64 / MB as f64)
    } else if bytes >= KB {
        format!("{:.2} KB", bytes as f64 / KB as f64)
    } else {
        format!("{} bytes", bytes)
    }
}
