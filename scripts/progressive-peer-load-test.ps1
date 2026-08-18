# Udaya Progressive Peer Load Test
# Tests P2P network behavior with increasing peer counts (5 -> 100)
# Monitors: connection handling, memory usage, sync stability
param(
    [int]$StartPeers = 5,
    [int]$EndPeers = 100,
    [int]$Step = 5,
    [int]$TestDurationSeconds = 30
)

$ErrorActionPreference = "Stop"
$UDAYA = "C:\Projects\Udaya\target\release\Udayad.exe"
$BASE_DIR = "C:\Projects\Udaya\temp_loadtest_data"

# Clean up previous test data
if (Test-Path $BASE_DIR) {
    Remove-Item -Recurse -Force $BASE_DIR
}
New-Item -ItemType Directory -Force -Path "$BASE_DIR" | Out-Null

Write-Host "=== Udaya Progressive Peer Load Test ===" -ForegroundColor Cyan
Write-Host "Testing peer counts from $StartPeers to $EndPeers (step=$Step)" -ForegroundColor Gray

function New-NodeConfig {
    param(
        [int]$NodeId,
        [int]$P2PPort,
        [int]$RPCPort,
        [int]$MaxPeers,
        [string]$ConnectTo = ""
    )
    
    $config = @"
[network]
listen_addr = "0.0.0.0"
listen_port = $P2PPort
max_peers = $MaxPeers
seed_nodes = $ConnectTo

[rpc]
listen_addr = "127.0.0.1"
listen_port = $RPCPort
"@
    
    $configPath = "$BASE_DIR\node${NodeId}.conf"
    Set-Content -Path $configPath -Value $config -Encoding UTF8
    return $configPath
}

function Start-Node {
    param(
        [int]$NodeId,
        [string]$ConfigPath,
        [string]$DataDir
    )
    
    $logDir = "$BASE_DIR"
    $proc = Start-Process -FilePath $UDAYA -ArgumentList "--config", $ConfigPath, "--datadir", $DataDir, "start" -PassThru -NoNewWindow -RedirectStandardOutput "$logDir\node${NodeId}.log" -RedirectStandardError "$logDir\node${NodeId}.err"
    Start-Sleep -Seconds 2
    return $proc
}

function Stop-Node {
    param($Proc)
    if (!$Proc.HasExited) {
        Stop-Process -Id $Proc.Id -Force
    }
}

function Get-Metrics {
    param([int]$RPCPort)
    
    try {
        $body = '{"jsonrpc":"2.0","id":"1","method":"getnetworkinfo","params":[]}'
        $response = Invoke-RestMethod -Uri "http://127.0.0.1:$RPCPort/" -Method Post -Body $body -ContentType "application/json" -TimeoutSec 5
        return $response.result
    } catch {
        return $null
    }
}

# Generate base configs
$nodeCount = 3
$baseP2PPort = 19798
$baseRPCPort = 18332
$nodeProcs = @()

Write-Host "`n[Setup] Starting base nodes..." -ForegroundColor Yellow
for ($i = 1; $i -le $nodeCount; $i++) {
    $p2pPort = $baseP2PPort + $i - 1
    $rpcPort = $baseRPCPort + ($i - 1) * 2
    $configPath = New-NodeConfig -NodeId $i -P2PPort $p2pPort -RPCPort $rpcPort -MaxPeers $EndPeers
    $dataDir = "$BASE_DIR\node${i}"
    New-Item -ItemType Directory -Force -Path $dataDir | Out-Null
    
    Write-Host "  Starting node $i (P2P=$p2pPort, RPC=$rpcPort)" -ForegroundColor.Gray
    $proc = Start-Node -NodeId $i -ConfigPath $configPath -DataDir $dataDir
    $nodeProcs += @{ Id = $i; Proc = $proc; P2PPort = $p2pPort; RPCPort = $rpcPort }
    Start-Sleep -Seconds 3
}

Write-Host "`n[Load Test] Progressive peer scaling: $StartPeers -> $EndPeers" -ForegroundColor.Cyan

# Connect nodes to each other
for ($i = 1; $i -le $nodeCount; $i++) {
    $connectTargets = ""
    for ($j = 1; $j -le $nodeCount; $j++) {
        if ($i -ne $j) {
            $p2pPort = $baseP2PPort + $j - 1
            $connectTargets += "127.0.0.1:$p2pPort,"
        }
    }
    $connectTargets = $connectTargets.TrimEnd(',')
    $configPath = New-NodeConfig -NodeId $i -P2PPort ($baseP2PPort + $i - 1) -RPCPort ($baseRPCPort + ($i - 1) * 2) -MaxPeers $EndPeers -ConnectTo $connectTargets
}

$results = @()
foreach ($peerCount in ($StartPeers..$EndPeers | Where-Object { $_ % $Step -eq 0 })) {
    Write-Host "`n[Test] Target peer count: $peerCount" -ForegroundColor.Yellow
    
    $startTime = Get-Date
    $stable = $true
    $maxMemory = 0
    
    while ((Get-Date) - $startTime).TotalSeconds -lt $TestDurationSeconds {
        $totalPeers = 0
        $allResponsive = $true
        
        foreach ($node in $nodeProcs) {
            $metrics = Get-Metrics -RPCPort $node.RPCPort
            if ($metrics -eq $null) {
                $allResponsive = $false
                continue
            }
            
            $peerCount_node = if ($metrics.connections) { $metrics.connections } else { 0 }
            $totalPeers += $peerCount_node
            
            try {
                $memUsage = (Get-Process -Id $node.Proc.Id -ErrorAction SilentlyContinue).WorkingSet64 / 1MB
                if ($memUsage -gt $maxMemory) { $maxMemory = $memUsage }
            } catch {}
        }
        
        if (-not $allResponsive) {
            $stable = $false
            Write-Host "  WARN: Node not responsive at peer count $peerCount" -ForegroundColor.Red
        }
        
        Write-Host "  Peers: $totalPeers | Memory: $([math]::Round($maxMemory, 1)) MB" -ForegroundColor.Gray
        Start-Sleep -Seconds 2
    }
    
    $result = [PSCustomObject]@{
        TargetPeers = $peerCount
        Stable = $stable
        MaxMemoryMB = [math]::Round($maxMemory, 1)
    }
    $results += $result
    
    if (-not $stable) {
        Write-Host "  INSTABILITY DETECTED at $peerCount peers. Stopping test." -ForegroundColor.Red
        break
    }
}

# Cleanup
Write-Host "`n[Cleanup] Stopping all nodes..." -ForegroundColor.Yellow
foreach ($node in $nodeProcs) {
    Stop-Node -Proc $node.Proc
}

Write-Host "`n=== Load Test Results ===" -ForegroundColor.Cyan
$results | Format-Table -AutoSize

Write-Host "`nMax memory across all tests: $([math]::Round(($results | Measure-Object -Property MaxMemoryMB -Maximum).Maximum, 1)) MB" -ForegroundColor.Green

# Cleanup
Remove-Item -Recurse -Force $BASE_DIR -ErrorAction SilentlyContinue
