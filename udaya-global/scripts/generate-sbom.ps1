# Generate CycloneDX SBOMs for every workspace crate.
$ErrorActionPreference = "Stop"
$env:PATH = "C:\Users\Admin\.cargo\bin;C:\msys64\mingw64\bin;" + $env:PATH
Get-ChildItem -Path crates -Directory | ForEach-Object {
    Push-Location $_.FullName
    cargo cyclonedx --format json
    Pop-Location
}
Write-Host "SBOM generation complete"