# Udaya Multi-Node Synchronization Test
# Starts 3 nodes and verifies they converge on the same chain height and best block hash
param(
    [int]$TestDurationMinutes = 30,
    [int]$MineBlocks = 10
)

$ErrorActionPreference = "Stop"
$UDAYA = "C:\Projects\Udaya\target\release\Udayad.exe"
$BASE_DIR = "C:\Projects\Udaya\temp_testnet_data"

# Clean up previous test data
if (Test-Path $BASE_DIR) {
    Remove-Item -Recurse -Force $BASE_DIR
}
New-Item -ItemType Directory -Force -Path "$BASE_DIR\node1" | Out-Null
New-Item -ItemType Directory -Force -Path "$BASE_DIR\node2" | Out-Null
New-Item -ItemType Directory -Force -Path "$BASE_DIR\node3" | Out-Null

Write-Host "=== Udaya Multi-Node Sync Test ===" -ForegroundColor Cyan
Write-Host "Node 1: Mining node (port 19798, RPC 18332)"
Write-Host "Node 2: Full node (port 19799, RPC 18334)"
Write-Host "Node 3: Full node (port 19800, RPC 18336)"

function Get-Rpc($port, $method, $params = @()) {
    $body = @{
        jsonrpc = "2.0"
        id = "1"
        method = $method
        params = $params
    } | ConvertTo-Json -Depth 10
    
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
    
    $tip1 = Get-Rpc 18332 "getblockhash" @($height1)
    $tip2 = Get-Rpc 18334 "getblockhash" @($height2)
    $tip3 = Get-Rpc 18336 "getblockhash" @($height3)
    
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

if (-not $converged) {
    Write-Host "`n❌ Nodes did not converge within $TestDurationMinutes minutes" -ForegroundColor Red
    Write-Host "   Final state:" -ForegroundColor Red
    Write-Host "   N1: height=$($height1) hash=$tip1" -ForegroundColor Red
    Write-Host "   N2: height=$($height2) hash=$tip2" -ForegroundColor Red
    Write-Host "   N3: height=$($height3) hash=$tip3" -ForegroundColor Red
} else {
    Write-Host "`n=== Testing Restart Recovery ===" -ForegroundColor Cyan
    
    # Stop all nodes
    Write-Host "Stopping all nodes..." -ForegroundColor Yellow
    Stop-Process -Id $node1.Id -Force -ErrorAction SilentlyContinue
    Stop-Process -Id $node2.Id -Force -ErrorAction SilentlyContinue
    Stop-Process -Id $node3.Id -Force -ErrorAction SilentlyContinue
    Start-Sleep -Seconds 3
    
    # Restart all nodes
    Write-Host "Restarting all nodes..." -ForegroundColor Yellow
    $node1 = Start-Process -FilePath $UDAYA -ArgumentList "--config", "config\node1.conf", "--datadir", "$BASE_DIR\node1", "start" -PassThru -NoNewWindow -RedirectStandardOutput "$BASE_DIR\node1.log" -RedirectStandardError "$BASE_DIR\node1.err"
    $node2 = Start-Process -FilePath $UDAYA -ArgumentList "--config", "config\node2.conf", "--datadir", "$BASE_DIR\node2", "start" -PassThru -NoNewWindow -RedirectStandardOutput "$BASE_DIR\node2.log" -RedirectStandardError "$BASE_DIR\node2.err"
    $node3 = Start-Process -FilePath $UDAYA -ArgumentList "--config", "config\node3.conf", "--datadir", "$BASE_DIR\node3", "start" -PassThru -NoNewWindow -RedirectStandardOutput "$BASE_DIR\node3.log" -RedirectStandardError "$BASE_DIR\node3.err"
    Start-Sleep -Seconds 10
    
    $recovered = $false
    $recoveryWatch = [System.Diagnostics.Stopwatch]::StartNew()
    
    while ($recoveryWatch.Elapsed -lt [TimeSpan]::FromMinutes(5)) {
        $h1 = Get-Rpc 18332 "getblockcount"
        $h2 = Get-Rpc 18334 "getblockcount"
        $h3 = Get-Rpc 18336 "getblockcount"
        $t1 = Get-Rpc 18332 "getblockhash" @($h1)
        $t2 = Get-Rpc 18334 "getblockhash" @($h2)
        $t3 = Get-Rpc 18336 "getblockhash" @($h3)
        
        Write-Host "[$(Get-Date -Format 'HH:mm:ss')] Recovery: N1=$h1 N2=$h2 N3=$h3" -ForegroundColor Gray
        
        if ($h1 -gt 0 -and $h2 -eq $h1 -and $h3 -eq $h1 -and $t1 -and $t2 -and $t3 -and $t1 -eq $t2 -and $t2 -eq $t3) {
            $recovered = $true
            Write-Host "`n✅ RESTART RECOVERY SUCCESSFUL!" -ForegroundColor Green
            Write-Host "   Height: $h1" -ForegroundColor Green
            Write-Host "   Best Block Hash: $t1" -ForegroundColor Green
            break
        }
        Start-Sleep -Seconds 5
    }
    
    if (-not $recovered) {
        Write-Host "`n❌ Restart recovery FAILED" -ForegroundColor Red
    }
}

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
    exit 1
}
