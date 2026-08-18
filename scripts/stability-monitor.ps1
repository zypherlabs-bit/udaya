$ErrorActionPreference = "Stop"
$LOG_FILE = "C:\Projects\Udaya\stability_test_log.csv"
"timestamp,height,n1_hash,n2_hash,n3_hash,n1_peers,n2_peers,n3_peers,converged" | Out-File -FilePath $LOG_FILE -Encoding utf8

function Get-BlockCount($port) {
    $body = '{"jsonrpc":"2.0","id":1,"method":"getblockcount","params":[]}'
    try {
        $r = Invoke-RestMethod -Uri "http://127.0.0.1:$port/" -Method Post -Body $body -ContentType "application/json" -TimeoutSec 5
        return $r.result
    } catch { return -1 }
}

function Get-BlockHash($port, $height) {
    $body = "{`"jsonrpc`":`"2.0`",`"id`":1,`"method`":`"getblockhash`",`"params`":[$height]}"
    try {
        $r = Invoke-RestMethod -Uri "http://127.0.0.1:$port/" -Method Post -Body $body -ContentType "application/json" -TimeoutSec 5
        return $r.result
    } catch { return "error" }
}

function Get-PeerCount($port) {
    $body = '{"jsonrpc":"2.0","id":1,"method":"getpeerinfo","params":[]}'
    try {
        $r = Invoke-RestMethod -Uri "http://127.0.0.1:$port/" -Method Post -Body $body -ContentType "application/json" -TimeoutSec 5
        if ($r.result) { return $r.result.Count } else { return 0 }
    } catch { return -1 }
}

Write-Host "=== Long-Running Stability Test ===" -ForegroundColor Cyan
Write-Host "Monitoring 3 nodes for 30 minutes..."
Write-Host "Log file: $LOG_FILE"
Write-Host "Press Ctrl+C to stop early`n"

$iterations = 0
$max_iterations = 360  # 30 minutes at 5-second intervals

while ($iterations -lt $max_iterations) {
    $ts = Get-Date -Format "yyyy-MM-dd HH:mm:ss"
    $h1 = Get-BlockCount 18332
    $h2 = Get-BlockCount 18334
    $h3 = Get-BlockCount 18336
    $t1 = Get-BlockHash 18332 $h1
    $t2 = Get-BlockHash 18334 $h2
    $t3 = Get-BlockHash 18336 $h3
    $p1 = Get-PeerCount 18332
    $p2 = Get-PeerCount 18334
    $p3 = Get-PeerCount 18336
    
    $converged = "false"
    if ($h1 -gt 0 -and $h2 -eq $h1 -and $h3 -eq $h1 -and $t1 -eq $t2 -and $t2 -eq $t3) {
        $converged = "true"
    }
    
    "$ts,$h1,$t1,$t2,$t3,$p1,$p2,$p3,$converged" | Out-File -FilePath $LOG_FILE -Append -Encoding utf8
    
    if ($iterations % 12 -eq 0) {  # Every minute
        Write-Host "[$ts] H=$h1 P1=$p1 P2=$p2 P3=$p3 Converged=$converged" -ForegroundColor $(if ($converged -eq "true") { "Green" } else { "Red" })
    }
    
    $iterations++
    Start-Sleep -Seconds 5
}

Write-Host "`n=== Stability Test Complete ===" -ForegroundColor Cyan
Write-Host "Log saved to: $LOG_FILE"
