$ErrorActionPreference = 'Stop'

Set-Location -Path "$PSScriptRoot\.."

Write-Host "Building rust-eval (release mode)..."
cargo build --release
if ($LASTEXITCODE -ne 0) {
    Write-Error "Build failed."
    exit $LASTEXITCODE
}

$targetDir = "$env:LOCALAPPDATA\Rust-Eval\app"
if (-not (Test-Path -Path $targetDir)) {
    Write-Host "Creating directory `"$targetDir`"..."
    New-Item -ItemType Directory -Path $targetDir -Force | Out-Null
}

Write-Host "Moving binary to `"$targetDir`"..."
Move-Item -Path "target\release\rs-eval.exe" -Destination "$targetDir\rs-eval.exe" -Force

Write-Host "Adding `"$targetDir`" to user PATH..."
try {
    $currentPath = [Environment]::GetEnvironmentVariable('Path', 'User')
    $paths = if ($currentPath) { $currentPath -split ';' } else { @() }
    if ($paths -notcontains $targetDir) {
        $newPath = if ($currentPath) { $currentPath.TrimEnd(';') + ';' + $targetDir } else { $targetDir }
        [Environment]::SetEnvironmentVariable('Path', $newPath, 'User')
        Write-Host "Added to user PATH."
    } else {
        Write-Host "Already in user PATH."
    }
} catch {
    Write-Warning "Failed to add `"$targetDir`" to user PATH."
}

Write-Host "Done! Installed rs-eval to `"$targetDir\rs-eval.exe`"."
