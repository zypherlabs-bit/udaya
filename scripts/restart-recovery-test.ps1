$ErrorActionPreference = "Stop"

function Get-BlockCount($port) {
    $body = '{"jsonrpc":"2.0","id":1,"method":"getblockcount","params":[]}'
    $r = Invoke-RestMethod -Uri "http://127.0.0.1:$port/" -Method Post -Body $body -ContentType "application/json" -TimeoutSec 5
    return $r.result
}

function Get-BlockHash($port, $height) {
    $body = "{`"jsonrpc`":`"2.0`",`"id`":1,`"method`":`"getblockhash`",`"params`":[$height]}"
    $r = Invoke-RestMethod -Uri "http://127.0.0.1:$port/" -Method Post -Body $body -ContentType "application/json" -TimeoutSec 5
    return $r.result
}

Write-Host "=== Restart Recovery Test ===" -ForegroundColor Cyan

# Stop all nodes
Write-Host "Stopping all nodes..." -ForegroundColor Yellow
Get-Process -Name "Udayad" -ErrorAction SilentlyContinue | Stop-Process -Force
Start-Sleep -Seconds 3

# Restart all nodes
Write-Host "Restarting all nodes..." -ForegroundColor Yellow
Start-Process -FilePath "C:\Projects\Udaya\target\release\Udayad.exe" -ArgumentList "--config","config\node1.conf","--datadir","temp_testnet_data\node1","start" -WindowStyle Hidden
Start-Process -FilePath "C:\Projects\Udaya\target\release\Udayad.exe" -ArgumentList "--config","config\node2.conf","--datadir","temp_testnet_data\node2","start" -WindowStyle Hidden
Start-Process -FilePath "C:\Projects\Udaya\target\release\Udayad.exe" -ArgumentList "--config","config\node3.conf","--datadir","temp_testnet_data\node3","start" -WindowStyle Hidden

Write-Host "Waiting for recovery (up to 5 minutes)..." -ForegroundColor Yellow
$recovered = $false
$maxWait = [TimeSpan]::FromMinutes(5)
$stopwatch = [System.Diagnostics.Stopwatch]::StartNew()

while (-not $recovered -and $stopwatch.Elapsed -lt $maxWait) {
    Start-Sleep -Seconds 5
    try {
        $h1 = Get-BlockCount 18332
        $h2 = Get-BlockCount 18334
        $h3 = Get-BlockCount 18336
        
        $time = Get-Date -Format "HH:mm:ss"
        Write-Host "[$time] N1=$h1 N2=$h2 N3=$h3" -ForegroundColor Gray
        
        if ($h1 -gt 0 -and $h2 -eq $h1 -and $h3 -eq $h1) {
            $t1 = Get-BlockHash 18332 $h1
            $t2 = Get-BlockHash 18334 $h2
            $t3 = Get-BlockHash 18336 $h3
            
            if ($t1 -eq $t2 -and $t2 -eq $t3) {
                $recovered = $true
                Write-Host "`n✅ RESTART RECOVERY SUCCESSFUL!" -ForegroundColor Green
                Write-Host "   Height: $h1" -ForegroundColor Green
                Write-Host "   Best Block Hash: $t1" -ForegroundColor Green
                break
            }
        }
    } catch {
        # Nodes still starting
    }
}

$stopwatch.Stop()

if (-not $recovered) {
    Write-Host "`n❌ Restart recovery FAILED or timed out" -ForegroundColor Red
    exit 1
} else {
    Write-Host "`n=== TEST RESULT: PASS ===" -ForegroundColor Green
    exit 0
}
