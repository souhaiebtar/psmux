param(
    [ValidateSet("start", "stop", "cancel")]
    [string]$Action = "start",
    [string]$Output = "psmux-perf.etl"
)

$ErrorActionPreference = "Stop"

$wpr = Get-Command wpr -ErrorAction SilentlyContinue
if (-not $wpr) {
    Write-Error "wpr.exe was not found. Install Windows Performance Toolkit (WPT) first."
    exit 1
}

if (-not [System.IO.Path]::IsPathRooted($Output)) {
    $Output = Join-Path (Get-Location) $Output
}

switch ($Action) {
    "start" {
        & $wpr.Source -cancel 2>$null | Out-Null
        & $wpr.Source -start GeneralProfile -filemode
        if ($LASTEXITCODE -ne 0) {
            Write-Error "Failed to start WPR trace."
            exit $LASTEXITCODE
        }
        Write-Host "WPR tracing started (GeneralProfile)." -ForegroundColor Green
        Write-Host "Run your psmux workload, then stop with:" -ForegroundColor Cyan
        Write-Host ".\\scripts\\perf-trace.ps1 -Action stop -Output `"$Output`"" -ForegroundColor Cyan
    }
    "stop" {
        & $wpr.Source -stop $Output
        if ($LASTEXITCODE -ne 0) {
            Write-Error "Failed to stop WPR trace."
            exit $LASTEXITCODE
        }
        Write-Host "WPR trace saved to $Output" -ForegroundColor Green
    }
    "cancel" {
        & $wpr.Source -cancel
        if ($LASTEXITCODE -ne 0) {
            Write-Error "Failed to cancel WPR trace."
            exit $LASTEXITCODE
        }
        Write-Host "WPR trace canceled." -ForegroundColor Yellow
    }
}
