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

function Get-PeerCount($port) {
    $body = '{"jsonrpc":"2.0","id":1,"method":"getpeerinfo","params":[]}'
    $r = Invoke-RestMethod -Uri "http://127.0.0.1:$port/" -Method Post -Body $body -ContentType "application/json" -TimeoutSec 5
    if ($r.result) { return $r.result.Count } else { return 0 }
}

Write-Host "=== Querying 3 Udaya Nodes ===" -ForegroundColor Cyan

$h1 = Get-BlockCount 18332
$h2 = Get-BlockCount 18334
$h3 = Get-BlockCount 18336

$p1 = Get-PeerCount 18332
$p2 = Get-PeerCount 18334
$p3 = Get-PeerCount 18336

Write-Host "N1: height=$h1 peers=$p1"
Write-Host "N2: height=$h2 peers=$p2"
Write-Host "N3: height=$h3 peers=$p3"

if ($h1 -gt 0 -and $h2 -gt 0 -and $h3 -gt 0) {
    $t1 = Get-BlockHash 18332 $h1
    $t2 = Get-BlockHash 18334 $h2
    $t3 = Get-BlockHash 18336 $h3
    
    Write-Host "N1 tip: $t1"
    Write-Host "N2 tip: $t2"
    Write-Host "N3 tip: $t3"
    
    if ($h1 -eq $h2 -and $h2 -eq $h3 -and $t1 -eq $t2 -and $t2 -eq $t3) {
        Write-Host "CONVERGED: All nodes at height $h1 with hash $t1" -ForegroundColor Green
        exit 0
    } else {
        Write-Host "DIVERGED: Heights or hashes differ" -ForegroundColor Red
        exit 1
    }
} else {
    Write-Host "Nodes not ready or not mining" -ForegroundColor Yellow
    exit 1
}
