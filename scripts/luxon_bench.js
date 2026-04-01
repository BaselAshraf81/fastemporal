#!/usr/bin/env node
/**
 * luxon_bench.js — Luxon benchmark harness + shadow-test helper.
 *
 * Modes:
 *   node luxon_bench.js bench         — runs perf benchmarks, prints ns/op to stdout
 *   node luxon_bench.js shadow <json> — executes a single operation, prints result
 *
 * Install Luxon first:
 *   npm install luxon
 */
"use strict";

let DateTime, Duration;
try {
  ({ DateTime, Duration } = require("luxon"));
} catch (_) {
  console.error("Luxon not found. Run: npm install luxon");
  process.exit(1);
}

const [, , mode, ...rest] = process.argv;

// ── Shadow-test ───────────────────────────────────────────────────────────────
if (mode === "shadow") {
  const payload = JSON.parse(rest[0] || "{}");
  console.log(runOp(payload));
  process.exit(0);
}

// ── Benchmark ─────────────────────────────────────────────────────────────────
if (mode === "bench" || mode === undefined) {
  const WARMUP = 10_000;
  const ITERS  = 200_000;

  function measure(label, fn) {
    for (let i = 0; i < WARMUP; i++) fn();
    const start = process.hrtime.bigint();
    for (let i = 0; i < ITERS; i++) fn();
    const end = process.hrtime.bigint();
    const nsPerOp = Number(end - start) / ITERS;
    console.log(`${label}\t${nsPerOp.toFixed(1)}`);
    return nsPerOp;
  }

  console.log("benchmark\tns_per_op");
  measure("now", () => DateTime.now());
  const ISO = "2025-06-07T14:32:00.000Z";
  measure("from_iso", () => DateTime.fromISO(ISO));
  const dt = DateTime.fromISO(ISO);
  measure("plus_days_7", () => dt.plus({ days: 7 }));
  measure("in_timezone", () => dt.setZone("America/New_York"));
  measure("to_iso", () => dt.toISO());
  measure("format_ymd", () => dt.toFormat("yyyy-MM-dd"));
  measure("start_of_day", () => dt.startOf("day"));
  const dt2 = DateTime.fromISO("2025-01-01T00:00:00Z");
  measure("diff_days", () => dt.diff(dt2, "days"));

  const LOOP = 1_000_000;
  const dur1 = Duration.fromObject({ days: 1 });
  const t0 = process.hrtime.bigint();
  let cur = DateTime.fromISO("2020-01-01T00:00:00Z");
  for (let i = 0; i < LOOP; i++) cur = cur.plus(dur1);
  const t1 = process.hrtime.bigint();
  console.log(`1M_plus_days\t${(Number(t1 - t0) / LOOP).toFixed(1)}`);
  process.exit(0);
}

console.error(`Unknown mode: ${mode}. Use: bench | shadow <json>`);
process.exit(1);

// ── Op dispatcher ─────────────────────────────────────────────────────────────
function runOp({ op, iso, days, tz, a, b, fmt }) {
  switch (op) {
    case "from_iso_to_iso":
      return DateTime.fromISO(iso, { setZone: true }).toISO();
    case "plus_days":
      return DateTime.fromISO(iso, { setZone: true }).plus({ days }).toISO();
    case "minus_days":
      return DateTime.fromISO(iso, { setZone: true }).minus({ days }).toISO();
    case "start_of_day":
      return DateTime.fromISO(iso, { setZone: true }).startOf("day").toISO();
    case "end_of_day":
      return DateTime.fromISO(iso, { setZone: true }).endOf("day").toISO();
    case "in_timezone":
      return DateTime.fromISO(iso, { setZone: true }).setZone(tz).toISO();
    case "diff_days":
      return String(Math.trunc(
        DateTime.fromISO(a, { setZone: true }).diff(
          DateTime.fromISO(b, { setZone: true }), "days"
        ).days
      ));
    case "format":
      return DateTime.fromISO(iso, { setZone: true }).toFormat(fmt);
    default:
      return `ERR:unknown_op:${op}`;
  }
}
