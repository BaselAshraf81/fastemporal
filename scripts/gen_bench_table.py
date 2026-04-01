#!/usr/bin/env python3
"""
gen_bench_table.py — generates the README.md benchmark table.

Usage
-----
  # After: cargo bench && node scripts/luxon_bench.js bench > luxon_results.txt
  python  scripts/gen_bench_table.py --luxon luxon_results.txt          # update README
  python  scripts/gen_bench_table.py --luxon luxon_results.txt --stdout # print only

Criterion folder naming (0.4 / 0.5)
-------------------------------------
  group "fastemporal", bench "now"          → fastemporal_now/new/estimates.json
  group "fastemporal/from_iso", param "..." → fastemporal_from_iso/<param>/new/estimates.json
  Colons in parameter names become underscores on all platforms.
"""
import argparse, json, re, sys
from pathlib import Path

CRITERION_DIR = Path("target") / "criterion"
README        = Path("README.md")
START_MARKER  = "<!-- BENCH_TABLE_START -->"
END_MARKER    = "<!-- BENCH_TABLE_END -->"

# (label, criterion_folder, criterion_param_subfolder_or_None, luxon_tsv_key)
ROWS = [
    ("now()",              "fastemporal_now",              None,                       "now"),
    ("from_iso (parse)",   "fastemporal_from_iso",         "2025-06-07T14_32_00.000Z", "from_iso"),
    ("plus(days:7)",       "fastemporal_plus_days_7",      None,                       "plus_days_7"),
    ("in_timezone()",      "fastemporal_in_timezone_NY",   None,                       "in_timezone"),
    ("to_iso()",           "fastemporal_to_iso",           None,                       "to_iso"),
    ("format(yyyy-MM-dd)", "fastemporal_format_yyyy_MM_dd",None,                       "format_ymd"),
    ("start_of('day')",    "fastemporal_start_of_day",     None,                       "start_of_day"),
    ("diff(days)",         "fastemporal_diff_days",        None,                       "diff_days"),
    ("1 M tight loop",     "fastemporal_1M_plus_days",     None,                       "1M_plus_days"),
]


def criterion_ns(folder, sub, base_dir=None):
    p = (base_dir or CRITERION_DIR) / folder
    if sub:
        p = p / sub
    for leaf in ("new", ""):
        est = (p / leaf / "estimates.json") if leaf else (p / "estimates.json")
        if est.exists():
            try:
                d = json.loads(est.read_text(encoding="utf-8"))
                return d["mean"]["point_estimate"]
            except (KeyError, json.JSONDecodeError):
                pass
    return None


def luxon_tsv(path):
    out = {}
    if not path:
        return out
    p = Path(path)
    if not p.exists():
        print(f"Warning: Luxon results file not found: {path}", file=sys.stderr)
        return out
    for line in p.read_text(encoding="utf-8").splitlines():
        if "\t" not in line:
            continue
        k, _, v = line.partition("\t")
        try:
            out[k.strip()] = float(v.strip())
        except ValueError:
            pass
    return out


def fmt(ns):
    if ns is None:
        return "—"
    if ns < 1_000:
        return f"{ns:.1f} ns"
    if ns < 1_000_000:
        return f"{ns / 1_000:.2f} µs"
    return f"{ns / 1_000_000:.2f} ms"


def speedup(rust, js):
    if rust is None or js is None or rust <= 0:
        return "—"
    r = js / rust
    return f"**{r:.0f}×**" if r >= 10 else f"**{r:.1f}×**"


def build_table(luxon_file, crit_dir=None):
    crit = Path(crit_dir) if crit_dir else CRITERION_DIR
    lux = luxon_tsv(luxon_file)
    lines = [
        "| Benchmark | fastemporal | Luxon (Node.js) | Speedup |",
        "|-----------|:-----------:|:---------------:|:-------:|",
    ]
    for label, folder, sub, lkey in ROWS:
        rns = criterion_ns(folder, sub, crit)
        jns = lux.get(lkey)
        lines.append(f"| `{label}` | {fmt(rns)} | {fmt(jns)} | {speedup(rns, jns)} |")
    lines += [
        "",
        "> Rust: `cargo bench`.  "
        "JS: `node scripts/luxon_bench.js bench > luxon_results.txt`.",
        "> Regenerate: `python scripts/gen_bench_table.py --luxon luxon_results.txt`",
    ]
    return "\n".join(lines)


def update_readme(table):
    if not README.exists():
        sys.exit(f"README.md not found at {README.resolve()}")
    text = README.read_text(encoding="utf-8")
    if START_MARKER not in text or END_MARKER not in text:
        README.write_text(
            text + f"\n{START_MARKER}\n{table}\n{END_MARKER}\n", encoding="utf-8"
        )
        print("Markers not found — table appended to README.md")
        return
    pat = re.compile(re.escape(START_MARKER) + r".*?" + re.escape(END_MARKER), re.DOTALL)
    README.write_text(pat.sub(f"{START_MARKER}\n{table}\n{END_MARKER}", text), encoding="utf-8")
    print("README.md updated.")


def main():
    ap = argparse.ArgumentParser(description=__doc__,
                                 formatter_class=argparse.RawDescriptionHelpFormatter)
    ap.add_argument("--luxon",          metavar="FILE", help="Luxon TSV output file")
    ap.add_argument("--stdout",         action="store_true", help="Print to stdout only")
    ap.add_argument("--criterion-dir",  metavar="DIR", default=str(CRITERION_DIR),
                    help=f"Criterion output dir (default: {CRITERION_DIR})")
    args = ap.parse_args()

    table = build_table(args.luxon, args.criterion_dir)
    if args.stdout:
        print(table)
    else:
        update_readme(table)


if __name__ == "__main__":
    main()
