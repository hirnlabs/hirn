[CmdletBinding()]
param (
    [Parameter(Mandatory = $false)]
    [string]$Version = ""
)

$ErrorActionPreference = "Stop"

# Repository settings
$Repo = "hirnlabs/hirn"
$Target = "x86_64-pc-windows-msvc"

# Create directories
$HomeDir = [System.Environment]::GetFolderPath("UserProfile")
$HirnDir = Join-Path $HomeDir ".hirn"
$BinDir = Join-Path $HirnDir "bin"
$ConfigDir = Join-Path $HirnDir "config"

New-Item -ItemType Directory -Force -Path $BinDir | Out-Null
New-Item -ItemType Directory -Force -Path $ConfigDir | Out-Null

# Resolve Tag
if ([string]::IsNullOrEmpty($Version)) {
    Write-Host "Fetching latest release information..."
    try {
        $ReleaseInfo = Invoke-RestMethod -Uri "https://api.github.com/repos/$Repo/releases/latest"
        $Tag = $ReleaseInfo.tag_name
    } catch {
        Write-Error "Failed to fetch latest release. Is there a published release on GitHub?"
        return
    }
} else {
    $Tag = $Version
}

Write-Host "Installing Hirn Agent $Tag for $Target..."

# Construct download URL
$AssetName = "hirn-$Tag-$Target.zip"
$DownloadUrl = "https://github.com/$Repo/releases/download/$Tag/$AssetName"

$TempDir = Join-Path [System.IO.Path]::GetTempPath() ([System.IO.Path]::GetRandomFileName())
New-Item -ItemType Directory -Path $TempDir | Out-Null

$ZipPath = Join-Path $TempDir $AssetName

Write-Host "Downloading from $DownloadUrl..."
try {
    Invoke-WebRequest -Uri $DownloadUrl -OutFile $ZipPath -UseBasicParsing
} catch {
    Write-Error "Download failed. Please check your network connection or the version tag '$Tag'."
    Remove-Item -Recurse -Force $TempDir | Out-Null
    return
}

Write-Host "Extracting archive..."
try {
    Expand-Archive -Path $ZipPath -DestinationPath $TempDir -Force
} catch {
    Write-Error "Failed to extract zip archive."
    Remove-Item -Recurse -Force $TempDir | Out-Null
    return
}

$ExeName = "hirn.exe"
$ExtractedExe = Join-Path $TempDir $ExeName
$TargetExe = Join-Path $BinDir $ExeName

if (-not (Test-Path $ExtractedExe)) {
    # Check if inside folder
    $ExtractedExe = Get-ChildItem -Path $TempDir -Filter $ExeName -Recurse | Select-Object -First 1
    if (-not $ExtractedExe) {
        Write-Error "Could not find hirn.exe in the extracted archive."
        Remove-Item -Recurse -Force $TempDir | Out-Null
        return
    }
}

Copy-Item -Path $ExtractedExe -Destination $TargetExe -Force
Remove-Item -Recurse -Force $TempDir | Out-Null

Write-Host ""
Write-Host "===============================================" -ForegroundColor Green
Write-Host " Hirn Agent was successfully installed!" -ForegroundColor Green
Write-Host "===============================================" -ForegroundColor Green
Write-Host "Binary path: $TargetExe"
Write-Host "Config path: $ConfigDir"
Write-Host ""
Write-Host "To add it to your PATH for the current user, run the following PowerShell command:"
Write-Host ""
Write-Host "  [System.Environment]::SetEnvironmentVariable('Path', [System.Environment]::GetEnvironmentVariable('Path', 'User') + ';$BinDir', 'User')" -ForegroundColor Yellow
Write-Host ""
Write-Host "Then, open a new PowerShell window for the changes to take effect."
Write-Host "==============================================="
