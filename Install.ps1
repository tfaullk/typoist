[CmdletBinding()]
param(
    [string]$InstallDir = "$env:USERPROFILE\bin",
    [string]$Repo       = "tfaullk/typoist",
    [string]$Target     = "x86_64-pc-windows-msvc.exe"
)

$ErrorActionPreference = "Stop"

function Write-Step([string]$msg) {
    Write-Host "==> " -ForegroundColor Cyan -NoNewline
    Write-Host $msg
}

function Write-Ok([string]$msg) {
    Write-Host "  + " -ForegroundColor Green -NoNewline
    Write-Host $msg
}

function Write-Warn([string]$msg) {
    Write-Host "  ! " -ForegroundColor Yellow -NoNewline
    Write-Host $msg
}

function Write-Fail([string]$msg) {
    Write-Host "  x " -ForegroundColor Red -NoNewline
    Write-Host $msg
}
$arch = [System.Runtime.InteropServices.RuntimeInformation]::OSArchitecture
switch ($arch) {
    "X64"   { $Target = "x86_64-pc-windows-msvc" }
    "Arm64" { $Target = "aarch64-pc-windows-msvc" }
    default {
        Write-Fail "Unsupported architecture: $arch"
        Write-Host "  This installer only supports x64 and arm64 Windows."
        exit 1
    }
}

$assetName = "typoist-$Target.exe"
$url = "https://github.com/tfaullk/typoist/releases/download/v1.3.0/typoist-1.3.0-x86_64-pc-windows-msvc.exe"

Write-Host ""
Write-Step "typoist installer"
Write-Host "  platform: $arch ($Target)"
Write-Host "  install:  $InstallDir"
Write-Host ""
Write-Step "Preparing install directory"

if (-not (Test-Path $InstallDir)) {
    New-Item -ItemType Directory -Force -Path $InstallDir | Out-Null
    Write-Ok "Created $InstallDir"
} else {
    Write-Ok "$InstallDir already exists"
}
Write-Step "Downloading $assetName"
Write-Host "  from: $url"

$tmp = Join-Path $env:TEMP "typoist-install-$([guid]::NewGuid()).exe"
try {
    $oldProgress = $ProgressPreference
    $ProgressPreference = "SilentlyContinue"
    try {
        Invoke-WebRequest -Uri $url -OutFile $tmp -UseBasicParsing
    } finally {
        $ProgressPreference = $oldProgress
    }
} catch {
    Write-Fail "Download failed: $_"
    Write-Host "  Check your internet connection and that the release exists:"
    Write-Host "  https://github.com/$Repo/releases/latest"
    exit 1
}

$size = (Get-Item $tmp).Length
if ($size -lt 100000) {
    Write-Fail "Downloaded file is suspiciously small ($size bytes)."
    Write-Host "  The URL probably returned an HTML error page instead of the binary."
    Remove-Item $tmp -Force -ErrorAction SilentlyContinue
    exit 1
}

Write-Ok ("Downloaded {0:N1} MiB" -f ($size / 1MB))


Write-Step "Unblocking binary (clears SmartScreen source tag)"
Unblock-File -Path $tmp -ErrorAction SilentlyContinue
Write-Ok "Unblocked"


Write-Step "Installing to $InstallDir"

$dest = Join-Path $InstallDir "typoist.exe"

$running = Get-Process -Name "typoist" -ErrorAction SilentlyContinue
if ($running) {
    Write-Warn "typoist is currently running. Close it and re-run this installer."
    Remove-Item $tmp -Force -ErrorAction SilentlyContinue
    exit 1
}

Move-Item -Force -Path $tmp -Destination $dest
Write-Ok "Installed to $dest"


Write-Step "Checking PATH"

$userPath = [Environment]::GetEnvironmentVariable("Path", "User")
if ($null -eq $userPath) { $userPath = "" }

$onPath = $userPath -split ';' | Where-Object { $_ -and ($_.TrimEnd('\') -ieq $InstallDir.TrimEnd('\')) }

if ($onPath) {
    Write-Ok "$InstallDir is already on PATH"
} else {
    $newPath = if ($userPath.TrimEnd(';')) { "$userPath;$InstallDir" } else { $InstallDir }
    [Environment]::SetEnvironmentVariable("Path", $newPath, "User")
    Write-Ok "Added $InstallDir to user PATH"

    $env:Path = "$env:Path;$InstallDir"

    Write-Warn "Restart your terminal for PATH changes to apply to other sessions."
}


Write-Step "Verifying installation"

try {
    $version = & $dest --version 2>&1
    if ($LASTEXITCODE -eq 0) {
        Write-Ok $version
    } else {
        Write-Warn "typoist --version returned exit code $LASTEXITCODE"
        Write-Host "  Output: $version"
    }
} catch {
    Write-Fail "Could not run typoist: $_"
    Write-Host "  Windows Defender may have quarantined the file."
    Write-Host "  Check: Windows Security -> Virus & threat protection -> Protection history"
    exit 1
}


Write-Host ""
Write-Host "typoist installed successfully." -ForegroundColor Green
Write-Host ""
Write-Host "  Run it:      typoist"
Write-Host "  Update it:   typoist --update"
Write-Host "  Help:        typoist --help"
Write-Host ""

if (-not $onPath) {
    Write-Host "Note: you may need to close and reopen your terminal." -ForegroundColor Yellow
    Write-Host "      (The current session should already work.)"
    Write-Host ""
}
