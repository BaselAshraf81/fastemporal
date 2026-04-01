<#
.SYNOPSIS
    Runs all fastemporal benchmarks and updates README.md.

.DESCRIPTION
    1. cargo bench  (Rust, criterion)
    2. node scripts/luxon_bench.js bench  (JS, Luxon)
    3. python/python3 scripts/gen_bench_table.py  (generate table)

.EXAMPLE
    pwsh -File scripts/run_benchmarks.ps1
#>
Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"

Write-Host "=== 1/3  Running Rust benchmarks ===" -ForegroundColor Cyan
cargo bench
if ($LASTEXITCODE -ne 0) { throw "cargo bench failed" }

Write-Host ""
Write-Host "=== 2/3  Running Luxon benchmarks ===" -ForegroundColor Cyan
$luxonOut = "$PSScriptRoot\..\luxon_results.txt"
node "$PSScriptRoot\luxon_bench.js" bench | Out-File -Encoding utf8 $luxonOut
if ($LASTEXITCODE -ne 0) { throw "luxon bench failed" }
Write-Host "Luxon results written to: $luxonOut"

Write-Host ""
Write-Host "=== 3/3  Generating README table ===" -ForegroundColor Cyan
$pyCmd = if (Get-Command python3 -ErrorAction SilentlyContinue) { "python3" } else { "python" }
& $pyCmd "$PSScriptRoot\gen_bench_table.py" --luxon $luxonOut
if ($LASTEXITCODE -ne 0) { throw "gen_bench_table.py failed" }

Write-Host ""
Write-Host "Done! README.md benchmark table updated." -ForegroundColor Green
