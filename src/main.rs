use std::io::{self, Write};
use std::sync::mpsc::{Receiver, RecvTimeoutError};
use std::thread::JoinHandle;
use std::time::{Duration, Instant};

use anyhow::{Context, Result};

use rivu::evaluation::Snapshot;
use rivu::tasks::PrequentialEvaluator;
use rivu::ui::cli::{drivers::InquireDriver, wizard::prompt_choice};
use rivu::ui::types::build::{build_evaluator, build_learner, build_stream};
use rivu::ui::types::choices::TaskChoice;

const RESET: &str = "\x1b[0m";
const BOLD: &str = "\x1b[1m";
const DIM: &str = "\x1b[2m";
const FG_CYAN: &str = "\x1b[36m";
const FG_GREEN: &str = "\x1b[32m";
const FG_MAGENTA: &str = "\x1b[35m";
const FG_BLUE: &str = "\x1b[34m";
const FG_GREY: &str = "\x1b[90m";

fn main() -> Result<()> {
    let driver = InquireDriver;

    let task: TaskChoice =
        prompt_choice::<TaskChoice, _>(&driver).context("failed while prompting for task")?;

    let render: JoinHandle<()>;

    let mut runner = match task {
        TaskChoice::EvaluatePrequential(p) => {
            let stream_choice = p.stream;
            let evaluator_choice = p.evaluator;
            let learner_choice = p.learner;
            let max_instances = p.max_instances;
            let max_seconds = p.max_seconds;
            let sample_freq = p.sample_frequency;
            let mem_check_freq = p.mem_check_frequency;

            let header: Vec<String> = vec![
                format!("{BOLD}{FG_CYAN}▶ Prequential Evaluation{RESET}"),
                format!(
                    "{DIM}sample_freq={}{RESET}  {DIM}mem_check_freq={}{RESET}  {}",
                    sample_freq,
                    mem_check_freq,
                    timestamp_now()
                ),
                format!(
                    "{FG_GREY}────────────────────────────────────────────────────────────────────────{RESET}"
                ),
            ];

            let stream = build_stream(stream_choice).context("failed to build stream")?;
            let evaluator =
                build_evaluator(evaluator_choice).context("failed to build evaluator")?;
            let learner = build_learner(learner_choice).context("failed to build learner")?;

            let (tx, rx) = std::sync::mpsc::channel();

            render = std::thread::spawn(move || {
                render_status_with_header(rx, header, 150, max_instances, max_seconds)
            });

            PrequentialEvaluator::new(
                learner,
                stream,
                evaluator,
                max_instances,
                max_seconds,
                sample_freq,
                mem_check_freq,
            )
            .context("failed to construct PrequentialEvaluator")?
            .with_progress(tx)
        }
    };

    runner.run().context("runner failed")?;

    drop(runner);
    let _ = render.join();

    // TODO: Implement file dumping

    Ok(())
}

/// Print header once, then refresh a single line with status.
/// Shows: seen, acc, κ, κₜ/κₘ (if present in `extras`), ips (throughput),
/// RAM-hours, elapsed time, and small progress bars for instances/time if limits exist.
pub fn render_status_with_header(
    rx: Receiver<Snapshot>,
    header_lines: Vec<String>,
    repaint_every_ms: u64,
    max_instances: Option<u64>,
    max_seconds: Option<u64>,
) {
    for line in &header_lines {
        println!("{line}");
    }

    println!();
    let _ = io::stdout().flush();

    let tick = Duration::from_millis(repaint_every_ms);
    let mut last_draw = Instant::now();
    let mut last_snap: Option<Snapshot> = None;
    let mut prev_for_ips: Option<Snapshot> = None;

    loop {
        match rx.recv_timeout(tick) {
            Ok(s) => {
                prev_for_ips = last_snap.clone();
                last_snap = Some(s);
            }
            Err(RecvTimeoutError::Timeout) => {}
            Err(RecvTimeoutError::Disconnected) => {
                if let Some(s) = last_snap.take() {
                    print!(
                        "\r{}\x1B[K\n",
                        format_status(&s, prev_for_ips.as_ref(), max_instances, max_seconds)
                    );
                    let _ = io::stdout().flush();
                }
                break;
            }
        }

        if last_draw.elapsed() >= tick {
            if let Some(s) = last_snap.as_ref() {
                let line = format_status(s, prev_for_ips.as_ref(), max_instances, max_seconds);
                print!("\r{}\x1B[K", line);
                let _ = io::stdout().flush();
            }
            last_draw = Instant::now();
        }
    }
}

fn format_status(
    s: &Snapshot,
    prev: Option<&Snapshot>,
    max_instances: Option<u64>,
    max_seconds: Option<u64>,
) -> String {
    let seen = s.instances_seen;
    let acc = fmtf(s.accuracy, 6);
    let kappa = fmtf(s.kappa, 6);

    let (mut kappa_t, mut kappa_m, mut prec, mut rec, mut f1) = (
        String::new(),
        String::new(),
        String::new(),
        String::new(),
        String::new(),
    );

    #[allow(unused_variables)]
    if let Some(extras) = snapshot_extras(s) {
        if let Some(v) = extras.get("kappa_t") {
            kappa_t = format!("  {DIM}κₜ{RESET} {}", fmtf(*v, 6));
        }
        if let Some(v) = extras.get("kappa_m") {
            kappa_m = format!("  {DIM}κₘ{RESET} {}", fmtf(*v, 6));
        }
        if let Some(v) = extras.get("precision") {
            prec = format!("  {DIM}P{RESET} {}", fmtf(*v, 6));
        }
        if let Some(v) = extras.get("recall") {
            rec = format!("  {DIM}R{RESET} {}", fmtf(*v, 6));
        }
        if let Some(v) = extras.get("f1") {
            f1 = format!("  {DIM}F1{RESET} {}", fmtf(*v, 6));
        }
    }

    let ips = prev.and_then(|p| {
        let ds = (s.instances_seen as i64 - p.instances_seen as i64) as f64;
        let dt = (s.seconds - p.seconds).max(0.0);
        if dt > 0.0 { Some(ds / dt) } else { None }
    });
    let ips_str = if let Some(x) = ips {
        fmt_int(x)
    } else {
        "—".into()
    };

    let bar_w = 20usize;
    let inst_bar = progress_bar(seen as f64, max_instances.map(|m| m as f64), bar_w);
    let time_bar = progress_bar(s.seconds, max_seconds.map(|m| m as f64), bar_w);

    format!(
        "{FG_GREEN}{BOLD}seen{RESET} {:>9}  \
         {FG_CYAN}{BOLD}acc{RESET} {:>7}  \
         {FG_MAGENTA}{BOLD}κ{RESET} {:>7} \
         {}{}{}{}{}  \
         {FG_BLUE}{BOLD}ips{RESET} {:>8}  \
         {DIM}ram_h{RESET} {:>8.3}  \
         {DIM}t{RESET} {:>7.2}s  \
         {DIM}[inst]{RESET} {}  \
         {DIM}[time]{RESET} {}",
        seen,
        acc,
        kappa,
        kappa_t,
        kappa_m,
        prec,
        rec,
        f1,
        ips_str,
        s.ram_hours,
        s.seconds,
        inst_bar,
        time_bar
    )
}

fn snapshot_extras(s: &Snapshot) -> Option<&std::collections::BTreeMap<String, f64>> {
    Some(&s.extras)
}

fn progress_bar(current: f64, total: Option<f64>, width: usize) -> String {
    match total {
        Some(t) if t.is_finite() && t > 0.0 => {
            let ratio = (current / t).clamp(0.0, 1.0);
            let filled = (ratio * width as f64).round() as usize;
            let empty = width.saturating_sub(filled);
            format!(
                "[{}{}] {:>3.0}%",
                "█".repeat(filled),
                "░".repeat(empty),
                ratio * 100.0
            )
        }
        _ => format!("[{}]   —%", "░".repeat(width)),
    }
}

fn fmtf(x: f64, prec: usize) -> String {
    if x.is_nan() {
        format!("{DIM}NaN{RESET}")
    } else {
        format!("{:>1$.prec$}", x, 6, prec = prec)
    }
}
fn fmt_int(x: f64) -> String {
    if x.is_nan() || !x.is_finite() {
        "NaN".into()
    } else {
        format!("{:.0}", x)
    }
}
fn timestamp_now() -> String {
    use chrono::{Local, SecondsFormat};
    let now = Local::now();
    format!(
        "{DIM}{}{}",
        now.to_rfc3339_opts(SecondsFormat::Secs, true),
        RESET
    )
}
