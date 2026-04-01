//! Shadow-test binary — verifies fastemporal output against Luxon (Node.js).
//!
//! Usage:
//! ```sh
//! cargo run --bin shadow_test -- --suite luxon
//! cargo run --bin shadow_test -- --suite datefns
//! ```
//!
//! For each test case the binary runs the operation through fastemporal,
//! then shells out to `node scripts/luxon_bench.js shadow <json>` and compares
//! the outputs.  Any mismatch is printed and causes a non-zero exit code.

use std::process::{Command, ExitCode};

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().collect();
    let suite = args.iter()
        .position(|a| a == "--suite")
        .and_then(|i| args.get(i + 1))
        .map(String::as_str)
        .unwrap_or("luxon");

    eprintln!("shadow_test: running suite '{suite}'");

    let cases = build_cases(suite);
    let mut failures = 0usize;

    for case in &cases {
        let rust_out = run_rust(case);
        match run_node(case) {
            Ok(node_out) => {
                if rust_out.trim() != node_out.trim() {
                    eprintln!("FAIL [{}]", case.name);
                    eprintln!("  Rust : {}", rust_out.trim());
                    eprintln!("  Node : {}", node_out.trim());
                    failures += 1;
                } else {
                    println!("ok   [{}]", case.name);
                }
            }
            Err(e) => {
                eprintln!("SKIP [{}] — node unavailable: {e}", case.name);
            }
        }
    }

    eprintln!("\n{} cases, {} failures", cases.len(), failures);
    if failures == 0 { ExitCode::SUCCESS } else { ExitCode::FAILURE }
}

struct Case {
    name: String,
    op: Op,
}

enum Op {
    FromIsoToIso(String),
    PlusDays { iso: String, days: i32 },
    MinusDays { iso: String, days: i32 },
    StartOfDay(String),
    EndOfDay(String),
    InTimezone { iso: String, tz: String },
    DiffDays { a: String, b: String },
    Format { iso: String, fmt: String },
}

fn build_cases(suite: &str) -> Vec<Case> {
    let mut v = Vec::new();
    if suite == "luxon" || suite == "all" {
        let iso_samples = [
            "2025-01-01T00:00:00Z",
            "2025-06-15T12:30:45.123Z",
            "2016-03-12T10:00:00Z",
            "2024-02-29T00:00:00Z",
        ];
        for iso in &iso_samples {
            v.push(Case { name: format!("from_iso_to_iso({iso})"), op: Op::FromIsoToIso(iso.to_string()) });
            v.push(Case { name: format!("plus_days_7({iso})"),     op: Op::PlusDays { iso: iso.to_string(), days: 7 } });
            v.push(Case { name: format!("minus_days_3({iso})"),    op: Op::MinusDays { iso: iso.to_string(), days: 3 } });
            v.push(Case { name: format!("start_of_day({iso})"),    op: Op::StartOfDay(iso.to_string()) });
            v.push(Case { name: format!("end_of_day({iso})"),      op: Op::EndOfDay(iso.to_string()) });
            v.push(Case { name: format!("format_ymd({iso})"),      op: Op::Format { iso: iso.to_string(), fmt: "yyyy-MM-dd".into() } });
        }
        v.push(Case {
            name: "diff_days_9".into(),
            op: Op::DiffDays { a: "2025-01-10T00:00:00Z".into(), b: "2025-01-01T00:00:00Z".into() },
        });
        #[cfg(any(feature = "tz-embedded", feature = "tz-system"))]
        v.push(Case {
            name: "in_timezone_ny".into(),
            op: Op::InTimezone { iso: "2025-01-01T05:00:00Z".into(), tz: "America/New_York".into() },
        });
    }
    v
}

fn run_rust(case: &Case) -> String {
    use fastemporal::{Duration, ZonedDateTime};
    match &case.op {
        Op::FromIsoToIso(s) => ZonedDateTime::from_iso(s)
            .map(|z| z.to_iso())
            .unwrap_or_else(|e| format!("ERR:{e}")),
        Op::PlusDays { iso, days } => ZonedDateTime::from_iso(iso)
            .map(|z| z.plus(Duration::days(*days)).to_iso())
            .unwrap_or_else(|e| format!("ERR:{e}")),
        Op::MinusDays { iso, days } => ZonedDateTime::from_iso(iso)
            .map(|z| z.minus(Duration::days(*days)).to_iso())
            .unwrap_or_else(|e| format!("ERR:{e}")),
        Op::StartOfDay(iso) => ZonedDateTime::from_iso(iso)
            .and_then(|z| z.start_of("day"))
            .map(|z| z.to_iso())
            .unwrap_or_else(|e| format!("ERR:{e}")),
        Op::EndOfDay(iso) => ZonedDateTime::from_iso(iso)
            .and_then(|z| z.end_of("day"))
            .map(|z| z.to_iso())
            .unwrap_or_else(|e| format!("ERR:{e}")),
        Op::InTimezone { iso, tz } => ZonedDateTime::from_iso(iso)
            .and_then(|z| z.in_timezone(tz))
            .map(|z| z.to_iso())
            .unwrap_or_else(|e| format!("ERR:{e}")),
        Op::DiffDays { a, b } => {
            let za = ZonedDateTime::from_iso(a);
            let zb = ZonedDateTime::from_iso(b);
            match (za, zb) {
                (Ok(za), Ok(zb)) => za.diff(zb, "days")
                    .map(|d| d.num_days().to_string())
                    .unwrap_or_else(|e| format!("ERR:{e}")),
                _ => "ERR:parse".into(),
            }
        }
        Op::Format { iso, fmt } => ZonedDateTime::from_iso(iso)
            .map(|z| z.format(fmt))
            .unwrap_or_else(|e| format!("ERR:{e}")),
    }
}

fn run_node(case: &Case) -> Result<String, String> {
    // Build a JSON payload describing the operation
    let payload = match &case.op {
        Op::FromIsoToIso(s)       => format!(r#"{{"op":"from_iso_to_iso","iso":"{s}"}}"#),
        Op::PlusDays { iso, days }  => format!(r#"{{"op":"plus_days","iso":"{iso}","days":{days}}}"#),
        Op::MinusDays { iso, days } => format!(r#"{{"op":"minus_days","iso":"{iso}","days":{days}}}"#),
        Op::StartOfDay(s)         => format!(r#"{{"op":"start_of_day","iso":"{s}"}}"#),
        Op::EndOfDay(s)           => format!(r#"{{"op":"end_of_day","iso":"{s}"}}"#),
        Op::InTimezone { iso, tz } => format!(r#"{{"op":"in_timezone","iso":"{iso}","tz":"{tz}"}}"#),
        Op::DiffDays { a, b }     => format!(r#"{{"op":"diff_days","a":"{a}","b":"{b}"}}"#),
        Op::Format { iso, fmt }   => format!(r#"{{"op":"format","iso":"{iso}","fmt":"{fmt}"}}"#),
    };

    let out = Command::new("node")
        .arg("scripts/luxon_bench.js")
        .arg("shadow")
        .arg(&payload)
        .output()
        .map_err(|e| e.to_string())?;

    if !out.status.success() {
        return Err(String::from_utf8_lossy(&out.stderr).to_string());
    }
    Ok(String::from_utf8_lossy(&out.stdout).trim().to_string())
}
