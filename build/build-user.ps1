Set-Location "$PSScriptRoot\.."

cargo build --release

$targetDir = "$env:LOCALAPPDATA\R-touch\bin"
New-Item -ItemType Directory -Force -Path $targetDir

Move-Item -Force -Path ".\target\release\rs-eval.exe" "$targetDir\rs-eval.exe"

$userPath = [Environment]::GetEnvironmentVariable("Path", "User")
if ($userPath -notlike "*$targetDir*") {
    [Environment]::SetEnvironmentVariable("Path", $userPath + ";$targetDir", "User")
}
$env:Path += ";$targetDir"
