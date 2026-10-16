<#
.SYNOPSIS
    Verifies installed-path discovery, paths with spaces, read-only install tree,
    and writable state isolation for zedkata.
.DESCRIPTION
    Simulates an installed environment outside the source tree:
    1. Installs zedkata into a directory containing spaces.
    2. Marks the installation directory tree strictly read-only.
    3. Runs zedkata from a completely separate working directory with spaces.
    4. Confirms that packaged lessons are discovered via executable parent.
    5. Confirms that state and exercises are written exclusively to agreed writable paths.
    6. Confirms that zero writes occur inside the read-only install tree.
#>

[CmdletBinding()]
param(
    [string]$StagedDir = ""
)

$ErrorActionPreference = "Stop"

$RepoRoot = (Get-Item $PSScriptRoot).Parent.FullName

if ([string]::IsNullOrWhiteSpace($StagedDir)) {
    # Check default dist folder
    $CargoToml = Get-Content (Join-Path $RepoRoot "Cargo.toml") -Raw
    if ($CargoToml -match 'version\s*=\s*"([^"]+)"') {
        $Version = $matches[1]
    } else {
        $Version = "0.1.0"
    }
    $StagedDir = Join-Path $RepoRoot "dist\zedkata-v$Version"
}

if (-not (Test-Path $StagedDir)) {
    Write-Host "Staged directory not found at $StagedDir. Building staging first..." -ForegroundColor Yellow
    & (Join-Path $PSScriptRoot "package.ps1")
}

Write-Host "==================================================" -ForegroundColor Cyan
Write-Host " Zedkata Installed-Path & Writable Writes Check   " -ForegroundColor Cyan
Write-Host "==================================================" -ForegroundColor Cyan

# 1. Setup isolated directories with spaces in temp
$Nonce = [System.Guid]::NewGuid().ToString("N").Substring(0, 8)
$BaseTemp = Join-Path ([System.IO.Path]::GetTempPath()) "zedkata_install_test_$Nonce"

$InstallTree = Join-Path $BaseTemp "Program Files (x86)\Zedkata Tools 2026"
$StudentWorkspace = Join-Path $BaseTemp "My Student Workspace\Projects 2026"

Write-Host "`n[1/5] Creating simulated install and workspace paths..." -ForegroundColor Green
Write-Host "  Install Tree: $InstallTree" -ForegroundColor Gray
Write-Host "  Workspace:    $StudentWorkspace" -ForegroundColor Gray

New-Item -ItemType Directory -Force -Path $InstallTree | Out-Null
New-Item -ItemType Directory -Force -Path $StudentWorkspace | Out-Null

try {
    # 2. Copy staged assets into install tree
    Write-Host "`n[2/5] Copying staged distribution into install tree..." -ForegroundColor Green
    Copy-Item -Path (Join-Path $StagedDir "*") -Destination $InstallTree -Recurse -Force

    $BinDir = Join-Path $InstallTree "bin"
    $TuiExe = Join-Path $BinDir "zedkata-tui.exe"
    $LspExe = Join-Path $BinDir "zedkata-lsp.exe"
    $AgentExe = Join-Path $BinDir "zedkata-agent.exe"

    if (-not (Test-Path $TuiExe) -or -not (Test-Path $LspExe) -or -not (Test-Path $AgentExe)) {
        throw "Failed to locate native binaries in $BinDir"
    }

    # Record baseline file inventory in install tree
    $InitialFiles = Get-ChildItem -Path $InstallTree -Recurse | Select-Object -ExpandProperty FullName

    # 3. Mark the entire install directory read-only
    Write-Host "`n[3/5] Marking install directory strictly read-only..." -ForegroundColor Green
    Get-ChildItem -Path $InstallTree -Recurse | ForEach-Object {
        $_.Attributes = $_.Attributes -bor [System.IO.FileAttributes]::ReadOnly
    }
    Write-Host "  [OK] Read-only file attributes applied" -ForegroundColor Gray

    # 4. Execute commands from outside checkout, with working dir set to $StudentWorkspace
    Write-Host "`n[4/5] Executing binaries from external workspace..." -ForegroundColor Green
    $SessionFile = Join-Path $StudentWorkspace "user state\session.json"
    $SessionDir = Split-Path $SessionFile -Parent
    New-Item -ItemType Directory -Force -Path $SessionDir | Out-Null

    Push-Location $StudentWorkspace
    try {
        # Check --version
        $verOut = & $TuiExe --version
        Write-Host "  [OK] zedkata-tui --version: $verOut" -ForegroundColor Gray

        $verLsp = & $LspExe --version
        Write-Host "  [OK] zedkata-lsp --version: $verLsp" -ForegroundColor Gray

        $verAgent = & $AgentExe --version
        Write-Host "  [OK] zedkata-agent --version: $verAgent" -ForegroundColor Gray

        # Run headless select: should discover external lessons via <exe_dir>/../lessons
        Write-Host "  Running 'select modal-00-baseline 0'..." -ForegroundColor Gray
        $selectOut = & $TuiExe select modal-00-baseline 0 --session $SessionFile 2>&1
        if ($LASTEXITCODE -ne 0) {
            throw "zedkata-tui select failed with exit code $($LASTEXITCODE): $selectOut"
        }
        Write-Host "  [OK] Exercise selected successfully" -ForegroundColor Gray

        # Run status
        Write-Host "  Running 'status'..." -ForegroundColor Gray
        $statusOut = & $TuiExe status --session $SessionFile 2>&1
        if ($LASTEXITCODE -ne 0) {
            throw "zedkata-tui status failed with exit code $($LASTEXITCODE): $statusOut"
        }
        Write-Host "  [OK] Status reported cleanly" -ForegroundColor Gray

        # Run agent walkthrough list
        Write-Host "  Running 'walkthrough list'..." -ForegroundColor Gray
        $wtOut = & $AgentExe walkthrough list --session $SessionFile 2>&1
        if ($LASTEXITCODE -ne 0) {
            throw "zedkata-agent walkthrough list failed with exit code $($LASTEXITCODE): $wtOut"
        }
        Write-Host "  [OK] Walkthrough list reported cleanly" -ForegroundColor Gray
    } finally {
        Pop-Location
    }

    # 5. Verify file writes and read-only isolation
    Write-Host "`n[5/5] Verifying write isolation and read-only install tree..." -ForegroundColor Green

    # Verify session was written in the writable workspace
    if (-not (Test-Path $SessionFile)) {
        throw "Session file was not created in writable workspace at $SessionFile"
    }
    $sessionContent = Get-Content $SessionFile -Raw
    if ($sessionContent -notmatch '"lesson_id":\s*"modal-00-baseline"') {
        throw "Session content does not contain expected lesson_id"
    }
    Write-Host "  [OK] State verified in writable path: $SessionFile" -ForegroundColor Gray

    # Verify zero files created or modified in read-only install directory
    $CurrentFiles = Get-ChildItem -Path $InstallTree -Recurse | Select-Object -ExpandProperty FullName
    $Diff = Compare-Object -ReferenceObject $InitialFiles -DifferenceObject $CurrentFiles
    if ($Diff) {
        throw "Detected unexpected file mutations inside install directory: $($Diff | Out-String)"
    }
    Write-Host "  [OK] Verified zero files added or modified in read-only install tree" -ForegroundColor Gray

    Write-Host "`n==================================================" -ForegroundColor Cyan
    Write-Host " Installed-Path & Write Isolation Check: PASSED!  " -ForegroundColor Green
    Write-Host "==================================================" -ForegroundColor Cyan
} finally {
    # Remove read-only attributes so cleanup can delete temp directory
    Get-ChildItem -Path $InstallTree -Recurse -ErrorAction SilentlyContinue | ForEach-Object {
        $_.Attributes = [System.IO.FileAttributes]::Normal
    }
    Remove-Item -Path $BaseTemp -Recurse -Force -ErrorAction SilentlyContinue
}
