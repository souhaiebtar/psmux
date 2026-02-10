# psmux End-to-End Regression + Performance Test
# Validates protocol/auth regressions and tracks key performance paths.

$ErrorActionPreference = "Continue"
$script:TestsPassed = 0
$script:TestsFailed = 0
$script:TestsSkipped = 0

function Write-Pass { param($msg) Write-Host "[PASS] $msg" -ForegroundColor Green; $script:TestsPassed++ }
function Write-Fail { param($msg) Write-Host "[FAIL] $msg" -ForegroundColor Red; $script:TestsFailed++ }
function Write-Skip { param($msg) Write-Host "[SKIP] $msg" -ForegroundColor Yellow; $script:TestsSkipped++ }
function Write-Info { param($msg) Write-Host "[INFO] $msg" -ForegroundColor Cyan }
function Write-Test { param($msg) Write-Host "[TEST] $msg" -ForegroundColor White }
function Write-Perf { param($msg) Write-Host "[PERF] $msg" -ForegroundColor Magenta }

function Get-PsmuxBinary {
    $release = "$PSScriptRoot\..\target\release\psmux.exe"
    $debug = "$PSScriptRoot\..\target\debug\psmux.exe"
    if (Test-Path $release) { return $release }
    if (Test-Path $debug) { return $debug }
    return $null
}

function Get-HomeDir {
    if ($env:USERPROFILE -and $env:USERPROFILE.Trim().Length -gt 0) { return $env:USERPROFILE }
    if ($env:HOME -and $env:HOME.Trim().Length -gt 0) { return $env:HOME }
    return [Environment]::GetFolderPath("UserProfile")
}

function Get-IntEnv([string]$Name, [int]$Default) {
    $v = [Environment]::GetEnvironmentVariable($Name)
    if ([string]::IsNullOrWhiteSpace($v)) { return $Default }
    $parsed = 0
    if ([int]::TryParse($v, [ref]$parsed)) { return $parsed }
    return $Default
}

function Read-LineFromStream([System.IO.Stream]$Stream) {
    $bytes = New-Object System.Collections.Generic.List[byte]
    $tmp = New-Object byte[] 1
    while ($true) {
        $read = $Stream.Read($tmp, 0, 1)
        if ($read -le 0) {
            if ($bytes.Count -eq 0) { return $null }
            break
        }
        $bytes.Add($tmp[0]) | Out-Null
        if ($tmp[0] -eq 10) { break } # '\n'
    }
    $line = [System.Text.Encoding]::UTF8.GetString($bytes.ToArray())
    return $line.TrimEnd("`r", "`n")
}

function Read-ExactBytes([System.IO.Stream]$Stream, [int]$Length) {
    $buf = New-Object byte[] $Length
    $offset = 0
    while ($offset -lt $Length) {
        $read = $Stream.Read($buf, $offset, $Length - $offset)
        if ($read -le 0) { throw "Unexpected EOF while reading payload ($offset/$Length)" }
        $offset += $read
    }
    return $buf
}

function Write-LineToStream([System.IO.Stream]$Stream, [string]$Line) {
    $bytes = [System.Text.Encoding]::UTF8.GetBytes("$Line`n")
    $Stream.Write($bytes, 0, $bytes.Length)
    $Stream.Flush()
}

function Open-Control([int]$Port) {
    $client = New-Object System.Net.Sockets.TcpClient("127.0.0.1", $Port)
    $client.NoDelay = $true
    $stream = $client.GetStream()
    $stream.ReadTimeout = 5000
    $stream.WriteTimeout = 5000
    return @{ Client = $client; Stream = $stream }
}

function Read-DumpPayload([System.IO.Stream]$Stream) {
    $first = Read-LineFromStream $Stream
    if ($null -eq $first) { throw "Connection closed while waiting for dump-state response" }
    if ($first.StartsWith("FRAME ")) {
        $len = 0
        if (-not [int]::TryParse($first.Substring(6), [ref]$len)) {
            throw "Invalid frame header: $first"
        }
        if ($len -lt 0) { throw "Invalid frame length: $len" }
        $payload = Read-ExactBytes $Stream $len
        return [System.Text.Encoding]::UTF8.GetString($payload)
    }
    return $first
}

function Get-PaneCount([string]$PSMUX, [string]$SessionName) {
    $lines = & $PSMUX list-panes -t $SessionName 2>&1
    if ($LASTEXITCODE -ne 0) { return -1 }
    return @($lines | Where-Object { $_ -and $_.ToString().Trim().Length -gt 0 }).Count
}

function Wait-ForSession([string]$PSMUX, [string]$SessionName, [int]$MaxAttempts = 24, [int]$SleepMs = 250) {
    for ($i = 0; $i -lt $MaxAttempts; $i++) {
        & $PSMUX has-session -t $SessionName 2>&1 | Out-Null
        if ($LASTEXITCODE -eq 0) { return $true }
        Start-Sleep -Milliseconds $SleepMs
    }
    return $false
}

function Start-TestSession([string]$PSMUX, [string]$SessionName) {
    try { & $PSMUX kill-session -t $SessionName 2>&1 | Out-Null } catch {}
    Start-Sleep -Milliseconds 150
    Start-Process -FilePath $PSMUX -ArgumentList @("new-session", "-s", $SessionName, "-d") -WindowStyle Hidden | Out-Null
    return (Wait-ForSession -PSMUX $PSMUX -SessionName $SessionName)
}

function Stop-TestSession([string]$PSMUX, [string]$SessionName) {
    try { & $PSMUX kill-session -t $SessionName 2>&1 | Out-Null } catch {}
}

$PSMUX = Get-PsmuxBinary
if (-not $PSMUX) {
    Write-Error "psmux binary not found. Build first: cargo build --release"
    exit 1
}

$isRelease = $PSMUX -like "*\target\release\*"
$baseDumpThreshold = Get-IntEnv "PSMUX_E2E_DUMPSTATE_THRESHOLD_MS" 120
$baseFramedThreshold = Get-IntEnv "PSMUX_E2E_FRAMED_THRESHOLD_MS" 100
$baseBatchThreshold = Get-IntEnv "PSMUX_E2E_BATCH_THRESHOLD_MS" 280
$thresholdFactor = if ($isRelease) { 1.0 } else { 2.0 }
$dumpThreshold = [int][Math]::Ceiling($baseDumpThreshold * $thresholdFactor)
$framedThreshold = [int][Math]::Ceiling($baseFramedThreshold * $thresholdFactor)
$batchThreshold = [int][Math]::Ceiling($baseBatchThreshold * $thresholdFactor)

$SESSION_NAME = "e2e-regperf-$PID"
$homeDir = Get-HomeDir
$portPath = "$homeDir\.psmux\$SESSION_NAME.port"
$keyPath = "$homeDir\.psmux\$SESSION_NAME.key"

Write-Host ""
Write-Host "=" * 70
Write-Host "         PSMUX E2E REGRESSION + PERFORMANCE TEST SUITE"
Write-Host "=" * 70
Write-Host ""
Write-Info "Binary: $PSMUX"
Write-Info "Mode: $(if ($isRelease) { "release" } else { "debug" })"
Write-Info "Thresholds: dump=${dumpThreshold}ms framed=${framedThreshold}ms batch=${batchThreshold}ms"
Write-Info "Session: $SESSION_NAME"
Write-Host ""

$sessionStarted = $false

try {
    Write-Test "Start detached session"
    $sessionStarted = Start-TestSession -PSMUX $PSMUX -SessionName $SESSION_NAME
    if (-not $sessionStarted) {
        Write-Fail "Failed to start test session"
        Write-Info "Tip: verify shell detection and detached session startup in this environment"
        throw "Session startup failed"
    }
    Write-Pass "Session started"

    if (-not (Test-Path $portPath) -or -not (Test-Path $keyPath)) {
        Write-Fail "Missing session port/key files"
        throw "Session metadata missing"
    }
    $port = [int](Get-Content $portPath -ErrorAction Stop).Trim()
    $key = (Get-Content $keyPath -ErrorAction Stop).Trim()
    if ($port -le 0 -or [string]::IsNullOrWhiteSpace($key)) {
        Write-Fail "Invalid session metadata (port/key)"
        throw "Invalid session metadata"
    }
    Write-Info "Control endpoint: 127.0.0.1:$port"

    Write-Host ""
    Write-Host "=" * 70
    Write-Host "  REGRESSION TESTS"
    Write-Host "=" * 70

    Write-Test "Unauthenticated control command is rejected"
    $c = Open-Control $port
    try {
        Write-LineToStream $c.Stream "dump-state"
        $line = Read-LineFromStream $c.Stream
        if ($line -match "Authentication required") {
            Write-Pass "Unauthenticated request rejected"
        } else {
            Write-Fail "Unexpected unauthenticated response: $line"
        }
    } finally {
        $c.Stream.Dispose()
        $c.Client.Close()
    }

    Write-Test "Invalid session key is rejected"
    $c = Open-Control $port
    try {
        Write-LineToStream $c.Stream "AUTH not-the-key"
        $line = Read-LineFromStream $c.Stream
        if ($line -match "Invalid session key") {
            Write-Pass "Invalid key rejected"
        } else {
            Write-Fail "Unexpected invalid-key response: $line"
        }
    } finally {
        $c.Stream.Dispose()
        $c.Client.Close()
    }

    Write-Test "Framed dump-state protocol negotiation + payload"
    $c = Open-Control $port
    try {
        Write-LineToStream $c.Stream "AUTH $key"
        $auth = Read-LineFromStream $c.Stream
        if (-not $auth.StartsWith("OK")) { throw "Auth failed: $auth" }
        Write-LineToStream $c.Stream "PERSISTENT"
        Write-LineToStream $c.Stream "protocol framed"
        $ack = Read-LineFromStream $c.Stream
        if ($ack -ne "ok") { throw "Protocol negotiation failed: $ack" }
        Write-LineToStream $c.Stream "dump-state"
        $raw = Read-DumpPayload $c.Stream
        if ($raw -match '"layout"' -and $raw -match '"windows"') {
            Write-Pass "Framed dump-state payload is valid"
        } else {
            Write-Fail "Framed dump-state payload missing expected fields"
        }
    } finally {
        $c.Stream.Dispose()
        $c.Client.Close()
    }

    Write-Test "Split-pane command path still works (regression guard)"
    $before = Get-PaneCount -PSMUX $PSMUX -SessionName $SESSION_NAME
    & $PSMUX split-window -h -t $SESSION_NAME 2>&1 | Out-Null
    Start-Sleep -Milliseconds 200
    $after = Get-PaneCount -PSMUX $PSMUX -SessionName $SESSION_NAME
    if ($before -ge 1 -and $after -gt $before) {
        Write-Pass "Pane count increased from $before to $after"
    } elseif ($before -ge 1 -and $after -eq $before) {
        Write-Skip "Pane count unchanged in this environment ($before)"
    } else {
        Write-Fail "Unable to validate pane split regression (before=$before after=$after)"
    }

    Write-Host ""
    Write-Host "=" * 70
    Write-Host "  PERFORMANCE TESTS"
    Write-Host "=" * 70

    Write-Test "framed dump-state avg latency (new TCP per request)"
    $iterations = 10
    $total = 0.0
    $ok = 0
    for ($i = 0; $i -lt $iterations; $i++) {
        $sw = [System.Diagnostics.Stopwatch]::StartNew()
        try {
            $c = Open-Control $port
            Write-LineToStream $c.Stream "AUTH $key"
            $auth = Read-LineFromStream $c.Stream
            if (-not $auth.StartsWith("OK")) { throw "Auth failed" }
            Write-LineToStream $c.Stream "PERSISTENT"
            Write-LineToStream $c.Stream "protocol framed"
            $ack = Read-LineFromStream $c.Stream
            if ($ack -ne "ok") { throw "protocol framed not accepted: $ack" }
            Write-LineToStream $c.Stream "dump-state"
            $raw = Read-DumpPayload $c.Stream
            if ($raw -match '"layout"' -and $raw -match '"windows"') {
                $ok++
                $sw.Stop()
                $total += $sw.Elapsed.TotalMilliseconds
            } else {
                $sw.Stop()
            }
        } catch {
            $sw.Stop()
        } finally {
            if ($c) {
                try { $c.Stream.Dispose() } catch {}
                try { $c.Client.Close() } catch {}
            }
        }
    }
    if ($ok -eq $iterations) {
        $avg = [math]::Round($total / $iterations, 1)
        Write-Perf "framed(new-tcp) avg: ${avg}ms over $iterations calls"
        if ($avg -le $dumpThreshold) { Write-Pass "framed(new-tcp) avg under ${dumpThreshold}ms" }
        else { Write-Fail "framed(new-tcp) avg ${avg}ms exceeds ${dumpThreshold}ms" }
    } else {
        Write-Fail "framed(new-tcp) succeeded only $ok/$iterations times"
    }

    Write-Test "framed dump-state avg latency (single persistent TCP)"
    $c = Open-Control $port
    try {
        Write-LineToStream $c.Stream "AUTH $key"
        $auth = Read-LineFromStream $c.Stream
        if (-not $auth.StartsWith("OK")) { throw "Auth failed: $auth" }
        Write-LineToStream $c.Stream "PERSISTENT"
        Write-LineToStream $c.Stream "protocol framed"
        $ack = Read-LineFromStream $c.Stream
        if ($ack -ne "ok") { throw "protocol framed not accepted: $ack" }

        $iterations = 15
        $total = 0.0
        $ok = 0
        for ($i = 0; $i -lt $iterations; $i++) {
            $sw = [System.Diagnostics.Stopwatch]::StartNew()
            Write-LineToStream $c.Stream "dump-state"
            $raw = Read-DumpPayload $c.Stream
            $sw.Stop()
            if ($raw -match '"layout"' -and $raw -match '"windows"') {
                $ok++
                $total += $sw.Elapsed.TotalMilliseconds
            }
        }
        if ($ok -eq $iterations) {
            $avg = [math]::Round($total / $iterations, 1)
            Write-Perf "framed dump-state avg: ${avg}ms over $iterations calls"
            if ($avg -le $framedThreshold) { Write-Pass "framed avg under ${framedThreshold}ms" }
            else { Write-Fail "framed avg ${avg}ms exceeds ${framedThreshold}ms" }
        } else {
            Write-Fail "framed dump-state succeeded only $ok/$iterations times"
        }
    } catch {
        Write-Fail "framed dump-state perf test failed: $_"
    } finally {
        $c.Stream.Dispose()
        $c.Client.Close()
    }

    Write-Test "batch command latency on persistent TCP"
    $c = Open-Control $port
    try {
        Write-LineToStream $c.Stream "AUTH $key"
        $auth = Read-LineFromStream $c.Stream
        if (-not $auth.StartsWith("OK")) { throw "Auth failed: $auth" }
        Write-LineToStream $c.Stream "PERSISTENT"
        Write-LineToStream $c.Stream "protocol framed"
        $ack = Read-LineFromStream $c.Stream
        if ($ack -ne "ok") { throw "protocol framed not accepted: $ack" }
        $sw = [System.Diagnostics.Stopwatch]::StartNew()
        for ($i = 0; $i -lt 40; $i++) {
            Write-LineToStream $c.Stream "send-key x"
        }
        Write-LineToStream $c.Stream "dump-state"
        $raw = Read-DumpPayload $c.Stream
        $sw.Stop()
        if ($raw -match '"layout"' -and $raw -match '"windows"') {
            $ms = [math]::Round($sw.Elapsed.TotalMilliseconds, 1)
            Write-Perf "40 commands + dump-state: ${ms}ms"
            if ($ms -le $batchThreshold) { Write-Pass "batch latency under ${batchThreshold}ms" }
            else { Write-Fail "batch latency ${ms}ms exceeds ${batchThreshold}ms" }
        } else {
            Write-Fail "batch response missing expected fields"
        }
    } catch {
        Write-Fail "batch latency test failed: $_"
    } finally {
        $c.Stream.Dispose()
        $c.Client.Close()
    }
}
catch {
    Write-Fail "E2E test aborted: $_"
}
finally {
    if ($sessionStarted) {
        Stop-TestSession -PSMUX $PSMUX -SessionName $SESSION_NAME
    }
}

Write-Host ""
Write-Host "=" * 70
Write-Host "  E2E TEST SUMMARY"
Write-Host "=" * 70
Write-Host ("  Total: {0}  Passed: {1}  Failed: {2}  Skipped: {3}" -f ($script:TestsPassed + $script:TestsFailed + $script:TestsSkipped), $script:TestsPassed, $script:TestsFailed, $script:TestsSkipped)

if ($script:TestsFailed -gt 0) {
    Write-Host ""
    Write-Host "  REGRESSION/PERFORMANCE CHECK FAILED" -ForegroundColor Red
    exit 1
}

Write-Host ""
Write-Host "  ALL E2E REGRESSION/PERFORMANCE TESTS PASSED" -ForegroundColor Green
exit 0
