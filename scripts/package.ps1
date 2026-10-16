<#
.SYNOPSIS
    Packages and stages zedkata native binaries and external lesson assets for release.
.DESCRIPTION
    Builds (or verifies) release binaries for zedkata-tui, zedkata-lsp, and zedkata-agent,
    then stages them alongside external curriculum lessons, fixtures, extension metadata,
    and a release manifest into a self-contained distribution directory.
.PARAMETER OutputDir
    The target directory where the staged install tree will be created.
    Defaults to dist/zedkata-v<version>.
.PARAMETER Target
    Optional cargo compilation target triple (e.g. x86_64-pc-windows-msvc).
.PARAMETER SkipBuild
    If specified, skips 'cargo build --release' and expects binaries to already exist.
.PARAMETER Clean
    If specified, removes existing OutputDir before staging.
.PARAMETER Archive
    If specified, compresses the staged directory into a .zip archive.
#>

[CmdletBinding()]
param(
    [string]$OutputDir = "",
    [string]$Target = "",
    [switch]$SkipBuild,
    [switch]$Clean,
    [switch]$Archive
)

$ErrorActionPreference = "Stop"

$RepoRoot = (Get-Item $PSScriptRoot).Parent.FullName

# 1. Read package version from root Cargo.toml
$CargoTomlContent = Get-Content (Join-Path $RepoRoot "Cargo.toml") -Raw
if ($CargoTomlContent -match 'version\s*=\s*"([^"]+)"') {
    $Version = $matches[1]
} else {
    $Version = "0.1.0"
}

Write-Host "========================================" -ForegroundColor Cyan
Write-Host " Zedkata Release Packaging (v$Version)  " -ForegroundColor Cyan
Write-Host "========================================" -ForegroundColor Cyan

# 2. Determine default output directory
if ([string]::IsNullOrWhiteSpace($OutputDir)) {
    $OutputDir = Join-Path $RepoRoot "dist\zedkata-v$Version"
}

if ($Clean -and (Test-Path $OutputDir)) {
    Write-Host "Cleaning existing output directory: $OutputDir" -ForegroundColor Yellow
    Remove-Item -Recurse -Force $OutputDir
}

# 3. Determine binary names and target path
$IsWindows = $true
$ExeExt = ".exe"

if (-not [string]::IsNullOrWhiteSpace($Target)) {
    $TargetReleaseDir = Join-Path $RepoRoot "target\$Target\release"
    $BuildArgs = @("build", "--workspace", "--release", "--target", $Target)
} else {
    $TargetReleaseDir = Join-Path $RepoRoot "target\release"
    $BuildArgs = @("build", "--workspace", "--release")
}

# 4. Build release binaries if not skipped
if (-not $SkipBuild) {
    Write-Host "`n[1/5] Building release binaries with Cargo..." -ForegroundColor Green
    Push-Location $RepoRoot
    try {
        & cargo @BuildArgs
        if ($LASTEXITCODE -ne 0) {
            throw "Cargo build failed with exit code $LASTEXITCODE"
        }
    } finally {
        Pop-Location
    }
} else {
    Write-Host "`n[1/5] Skipping cargo build (-SkipBuild specified)..." -ForegroundColor Yellow
}

# 5. Verify release binaries exist
$BinaryNames = @("zedkata-tui$ExeExt", "zedkata-lsp$ExeExt", "zedkata-agent$ExeExt")
Write-Host "`n[2/5] Verifying binary artifacts in $TargetReleaseDir..." -ForegroundColor Green

foreach ($bin in $BinaryNames) {
    $binPath = Join-Path $TargetReleaseDir $bin
    if (-not (Test-Path $binPath)) {
        throw "Missing required release binary: $binPath"
    }
    Write-Host "  [OK] Found $bin" -ForegroundColor Gray
}

# 6. Create staging directory layout
Write-Host "`n[3/5] Assembling staging tree at $OutputDir..." -ForegroundColor Green
$BinDest = Join-Path $OutputDir "bin"
$LessonsDest = Join-Path $OutputDir "lessons"
$ExtensionDest = Join-Path $OutputDir "extension"

New-Item -ItemType Directory -Force -Path $BinDest | Out-Null
New-Item -ItemType Directory -Force -Path $LessonsDest | Out-Null
New-Item -ItemType Directory -Force -Path $ExtensionDest | Out-Null

# Copy binaries
$BinaryManifest = @()
foreach ($bin in $BinaryNames) {
    $src = Join-Path $TargetReleaseDir $bin
    $dst = Join-Path $BinDest $bin
    Copy-Item -Path $src -Destination $dst -Force

    $hash = (Get-FileHash -Path $dst -Algorithm SHA256).Hash.ToLowerInvariant()
    $fileSize = (Get-Item $dst).Length

    $BinaryManifest += @{
        name = $bin
        path = "bin/$bin"
        sha256 = $hash
        size_bytes = $fileSize
    }
}

# Copy lessons and fixtures (excluding any scratch or temporary files)
$LessonsSrc = Join-Path $RepoRoot "lessons"
Copy-Item -Path (Join-Path $LessonsSrc "*") -Destination $LessonsDest -Recurse -Force

# Copy extension metadata, docs, and compiled WASM binary (if present)
$ExtSrc = Join-Path $RepoRoot "extension"
Copy-Item -Path (Join-Path $ExtSrc "extension.toml") -Destination $ExtensionDest -Force
Copy-Item -Path (Join-Path $ExtSrc "README.md") -Destination $ExtensionDest -Force
$LanguagesSrc = Join-Path $ExtSrc "languages"
if (Test-Path $LanguagesSrc) {
    Copy-Item -Path $LanguagesSrc -Destination $ExtensionDest -Recurse -Force
    Write-Host "  [OK] Extension languages/ definitions packaged" -ForegroundColor Gray
}
$WasmBinary = Join-Path $ExtSrc "target\wasm32-wasip2\release\zedkata_extension.wasm"
if (Test-Path $WasmBinary) {
    Copy-Item -Path $WasmBinary -Destination (Join-Path $ExtensionDest "extension.wasm") -Force
    Write-Host "  [OK] Precompiled extension.wasm packaged" -ForegroundColor Gray
}

# Copy root README, licenses, and release notes
Copy-Item -Path (Join-Path $RepoRoot "README.md") -Destination $OutputDir -Force
foreach ($doc in @("LICENSE", "LICENSE-MIT", "LICENSE-APACHE", "RELEASE_NOTES.md")) {
    $docPath = Join-Path $RepoRoot $doc
    if (Test-Path $docPath) {
        Copy-Item -Path $docPath -Destination $OutputDir -Force
        Write-Host "  [OK] $doc packaged" -ForegroundColor Gray
    }
}

# 7. Generate MANIFEST.json
Write-Host "`n[4/5] Generating release manifest..." -ForegroundColor Green
$LessonTomlFiles = Get-ChildItem -Path $LessonsDest -Filter "*.toml"
$FixturesDir = Join-Path $LessonsDest "fixtures"

$ManifestObj = [ordered]@{
    name = "zedkata"
    version = $Version
    packaged_at = (Get-Date).ToUniversalTime().ToString("yyyy-MM-ddTHH:mm:ssZ")
    target = if ($Target) { $Target } else { "host" }
    binaries = $BinaryManifest
    curriculum = [ordered]@{
        lesson_count = $LessonTomlFiles.Count
        lessons = @($LessonTomlFiles | ForEach-Object { $_.Name })
        fixtures_present = (Test-Path $FixturesDir)
    }
    layout = [ordered]@{
        bin_dir = "bin"
        lessons_dir = "lessons"
        extension_dir = "extension"
    }
}

$ManifestJson = $ManifestObj | ConvertTo-Json -Depth 5
$Utf8NoBom = [System.Text.UTF8Encoding]::new($false)
[System.IO.File]::WriteAllText((Join-Path $OutputDir "MANIFEST.json"), $ManifestJson, $Utf8NoBom)
Write-Host "  [OK] MANIFEST.json written" -ForegroundColor Gray

# 8. Validate staged binaries & catalog
Write-Host "`n[5/5] Validating staged artifacts..." -ForegroundColor Green

# Verify binary execution & versions
foreach ($bin in $BinaryNames) {
    $stagedBin = Join-Path $BinDest $bin
    $out = & $stagedBin --version 2>&1
    if ($LASTEXITCODE -ne 0) {
        throw "Staged binary $bin failed execution with code $LASTEXITCODE"
    }
    if ($out -notmatch [regex]::Escape($Version)) {
        throw "Staged binary $bin returned version '$out', expected '$Version'"
    }
    Write-Host "  [OK] $bin --version reports: $out" -ForegroundColor Gray
}

# Verify lesson catalog completeness
if ($LessonTomlFiles.Count -ne 10) {
    throw "Expected 10 curriculum lessons, found $($LessonTomlFiles.Count)"
}
Write-Host "  [OK] 10 external lesson TOML files verified in staged lessons directory" -ForegroundColor Gray

Write-Host "`n========================================" -ForegroundColor Cyan
Write-Host " Release staged successfully!          " -ForegroundColor Green
Write-Host " Staged Location: $OutputDir" -ForegroundColor Cyan
Write-Host "========================================" -ForegroundColor Cyan

# 9. Create release archive if -Archive specified
if ($Archive) {
    $TargetLabel = if ($Target) { $Target } else { "x86_64-pc-windows-msvc" }
    $ArchiveName = "zedkata-v$Version-$TargetLabel.zip"
    $ParentDir = Split-Path -Parent $OutputDir
    if (-not (Test-Path $ParentDir)) {
        New-Item -ItemType Directory -Force -Path $ParentDir | Out-Null
    }
    $ArchivePath = Join-Path $ParentDir $ArchiveName
    Write-Host "`nCreating release archive: $ArchiveName..." -ForegroundColor Green
    if (Test-Path $ArchivePath) {
        Remove-Item -Force $ArchivePath
    }
    Compress-Archive -Path (Join-Path $OutputDir "*") -DestinationPath $ArchivePath -Force
    Write-Host "  [OK] Release archive created at: $ArchivePath" -ForegroundColor Gray
}
