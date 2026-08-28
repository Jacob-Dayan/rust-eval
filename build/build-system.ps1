if (-not ([Security.Principal.WindowsPrincipal][Security.Principal.WindowsIdentity]::GetCurrent()).IsInRole([Security.Principal.WindowsBuiltInRole]::Administrator)) {
    $scriptDir = Split-Path -Parent $MyInvocation.MyCommand.Path
    Start-Process powershell.exe -ArgumentList "-NoProfile", "-ExecutionPolicy", "Bypass", "-File", "`"$PSCommandPath`"" -WorkingDirectory $scriptDir -Verb RunAs
    exit
}

Set-Location "$PSScriptRoot\.."
cargo build --release

$targetDir = "$env:ProgramFiles\R-touch\bin"
New-Item -ItemType Directory -Force -Path $targetDir
Move-Item -Force -Path ".\target\release\rs-eval.exe" "$targetDir\rs-eval.exe"
$oldSystemPath = [Environment]::GetEnvironmentVariable("Path", "Machine")
if ($oldSystemPath -notlike "*$targetDir*") {
    [Environment]::SetEnvironmentVariable("Path", $oldSystemPath + ";$targetDir", "Machine")
}
$env:Path += ";$targetDir"
