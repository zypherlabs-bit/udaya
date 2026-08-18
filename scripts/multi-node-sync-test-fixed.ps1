# Udaya Multi-Node Synchronization Test (Fixed)
param(
    [int]$TestDurationMinutes = 15,
    [int]$MineBlocks = 10
)

$ErrorActionPreference = "Stop"
$UDAYA = "C:\Projects\Udaya\target\release\Udayad.exe"
$BASE_DIR = "C:\Projects\Udaya\temp_testnet_data"

Remove-Item -Recurse -Force $BASE_DIR -ErrorAction SilentlyContinue
New-Item -ItemType Directory -Force -Path "$BASE_DIR\node1" | Out-Null
New-Item -ItemType Directory -Force -Path "$BASE_DIR\node2" | Out-Null
New-Item -ItemType Directory -Force -Path "$BASE_DIR\node3" | Out-Null

Write-Host "=== Udaya Multi-Node Sync Test ===" -ForegroundColor Cyan

function Get-Rpc($port, $method, $params = @()) {
    $paramsJson = $params | ConvertTo-Json -Depth 10
    $body = "{`"jsonrpc`":`"2.0`",`"id`":1,`"method`":`"$method`",`"params`":$paramsJson}"
    
    try {
        $response = Invoke-RestMethod -Uri "http://127.0.0.1:$port/" -Method Post -Body $body -ContentType "application/json" -TimeoutSec 5
        return $response.result
    } catch {
        return $null
    }
}

# Start Node 1 (mining)
Write-Host "`n[1/3] Starting Node 1 (miner)..." -ForegroundColor Yellow
$node1 = Start-Process -FilePath $UDAYA -ArgumentList "--config", "config\node1.conf", "--datadir", "$BASE_DIR\node1", "start" -PassThru -NoNewWindow -RedirectStandardOutput "$BASE_DIR\node1.log" -RedirectStandardError "$BASE_DIR\node1.err"
Start-Sleep -Seconds 3

# Start Node 2
Write-Host "[2/3] Starting Node 2..." -ForegroundColor Yellow
$node2 = Start-Process -FilePath $UDAYA -ArgumentList "--config", "config\node2.conf", "--datadir", "$BASE_DIR\node2", "start" -PassThru -NoNewWindow -RedirectStandardOutput "$BASE_DIR\node2.log" -RedirectStandardError "$BASE_DIR\node2.err"
Start-Sleep -Seconds 3

# Start Node 3
Write-Host "[3/3] Starting Node 3..." -ForegroundColor Yellow
$node3 = Start-Process -FilePath $UDAYA -ArgumentList "--config", "config\node3.conf", "--datadir", "$BASE_DIR\node3", "start" -PassThru -NoNewWindow -RedirectStandardOutput "$BASE_DIR\node3.log" -RedirectStandardError "$BASE_DIR\node3.err"
Start-Sleep -Seconds 5

Write-Host "`n=== Monitoring Synchronization ===" -ForegroundColor Cyan

$converged = $false
$maxWait = [TimeSpan]::FromMinutes($TestDurationMinutes)
$stopwatch = [System.Diagnostics.Stopwatch]::StartNew()

while (-not $converged -and $stopwatch.Elapsed -lt $maxWait) {
    $height1 = Get-Rpc 18332 "getblockcount"
    $height2 = Get-Rpc 18334 "getblockcount"
    $height3 = Get-Rpc 18336 "getblockcount"
    
    $tip1 = if ($height1 -gt 0) { Get-Rpc 18332 "getblockhash" @($height1) } else { $null }
    $tip2 = if ($height2 -gt 0) { Get-Rpc 18334 "getblockhash" @($height2) } else { $null }
    $tip3 = if ($height3 -gt 0) { Get-Rpc 18336 "getblockhash" @($height3) } else { $null }
    
    $peer1 = (Get-Rpc 18332 "getpeerinfo").Count
    $peer2 = (Get-Rpc 18334 "getpeerinfo").Count
    $peer3 = (Get-Rpc 18336 "getpeerinfo").Count
    
    Write-Host "[$(Get-Date -Format 'HH:mm:ss')] N1: h=$height1 peers=$peer1 | N2: h=$height2 peers=$peer2 | N3: h=$height3 peers=$peer3" -ForegroundColor Gray
    
    if ($height1 -gt 0 -and $height2 -eq $height1 -and $height3 -eq $height1 -and $tip1 -and $tip2 -and $tip3 -and $tip1 -eq $tip2 -and $tip2 -eq $tip3) {
        $converged = $true
        Write-Host "`n✅ ALL NODES CONVERGED!" -ForegroundColor Green
        Write-Host "   Height: $height1" -ForegroundColor Green
        Write-Host "   Best Block Hash: $tip1" -ForegroundColor Green
        Write-Host "   Node 1 peers: $peer1, Node 2 peers: $peer2, Node 3 peers: $peer3" -ForegroundColor Green
        break
    }
    
    Start-Sleep -Seconds 5
}

$stopwatch.Stop()

# Cleanup
Write-Host "`nCleaning up..." -ForegroundColor Yellow
Stop-Process -Id $node1.Id -Force -ErrorAction SilentlyContinue
Stop-Process -Id $node2.Id -Force -ErrorAction SilentlyContinue
Stop-Process -Id $node3.Id -Force -ErrorAction SilentlyContinue
Start-Sleep -Seconds 2

if ($converged) {
    Write-Host "`n=== TEST RESULT: PASS ===" -ForegroundColor Green
    exit 0
} else {
    Write-Host "`n=== TEST RESULT: FAIL ===" -ForegroundColor Red
    Write-Host "   Final state:" -ForegroundColor Red
    Write-Host "   N1: height=$height1 hash=$tip1 peers=$peer1" -ForegroundColor Red
    Write-Host "   N2: height=$height2 hash=$tip2 peers=$peer2" -ForegroundColor Red
    Write-Host "   N3: height=$height3 hash=$tip3 peers=$peer3" -ForegroundColor Red
    exit 1
}
