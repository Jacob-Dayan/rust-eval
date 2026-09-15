$ErrorActionPreference = 'Stop'

Set-Location -Path "$PSScriptRoot\.."

Write-Host "Building rust-eval (release mode)..."
cargo build --release
if ($LASTEXITCODE -ne 0) {
    Write-Error "Build failed."
    exit $LASTEXITCODE
}

$targetDir = "$env:ProgramFiles\Rust-Eval\app"
if (-not (Test-Path -Path $targetDir)) {
    Write-Host "Creating directory `"$targetDir`"..."
    New-Item -ItemType Directory -Path $targetDir -Force | Out-Null
}

Write-Host "Moving binary to `"$targetDir`"..."
Move-Item -Path "target\release\rs-eval.exe" -Destination "$targetDir\rs-eval.exe" -Force

Write-Host "Adding `"$targetDir`" to system PATH..."
try {
    $currentPath = [Environment]::GetEnvironmentVariable('Path', 'Machine')
    $paths = if ($currentPath) { $currentPath -split ';' } else { @() }
    if ($paths -notcontains $targetDir) {
        $newPath = if ($currentPath) { $currentPath.TrimEnd(';') + ';' + $targetDir } else { $targetDir }
        [Environment]::SetEnvironmentVariable('Path', $newPath, 'Machine')
        Write-Host "Added to system PATH."
    } else {
        Write-Host "Already in system PATH."
    }
} catch {
    Write-Warning "Failed to add `"$targetDir`" to system PATH. Make sure you run this script as Administrator."
}

Write-Host "Done! Installed rs-eval to `"$targetDir\rs-eval.exe`"."
