# GitMentor MVP Environment Check Script (PowerShell)
# Usage: .\Check-Environment-Simple.ps1

param(
    [switch]$Fix
)

$ErrorActionPreference = "Continue"

Write-Host "GitMentor MVP Environment Check" -ForegroundColor Green
Write-Host "================================" -ForegroundColor Gray

$allGood = $true

# Check PowerShell version
Write-Host "Checking PowerShell version..." -ForegroundColor Yellow
$psVersion = $PSVersionTable.PSVersion
if ($psVersion.Major -ge 5) {
    Write-Host "   OK: PowerShell $($psVersion.ToString())" -ForegroundColor Green
} else {
    Write-Host "   ERROR: PowerShell $($psVersion.ToString()) - Need 5.0+" -ForegroundColor Red
    $allGood = $false
}

# Check Node.js
Write-Host "Checking Node.js..." -ForegroundColor Yellow
try {
    $nodeVersion = node --version 2>$null
    if ($nodeVersion) {
        $versionNumber = [version]($nodeVersion -replace 'v', '')
        if ($versionNumber.Major -ge 16) {
            Write-Host "   OK: Node.js $nodeVersion" -ForegroundColor Green
        } else {
            Write-Host "   WARNING: Node.js $nodeVersion - Recommend 16+" -ForegroundColor Yellow
        }
    } else {
        throw "Node.js not found"
    }
}
catch {
    Write-Host "   ERROR: Node.js not installed" -ForegroundColor Red
    if ($Fix) {
        Write-Host "   Trying to install Node.js..." -ForegroundColor Cyan
        try {
            winget install OpenJS.NodeJS --accept-package-agreements --accept-source-agreements
            Write-Host "   OK: Node.js installed, please restart terminal" -ForegroundColor Green
        }
        catch {
            Write-Host "   ERROR: Auto-install failed, please install manually: https://nodejs.org/" -ForegroundColor Red
        }
    } else {
        Write-Host "   TIP: winget install OpenJS.NodeJS" -ForegroundColor Cyan
        Write-Host "   TIP: or visit https://nodejs.org/" -ForegroundColor Cyan
    }
    $allGood = $false
}

# Check npm
Write-Host "Checking npm..." -ForegroundColor Yellow
try {
    $npmVersion = npm --version 2>$null
    if ($npmVersion) {
        Write-Host "   OK: npm $npmVersion" -ForegroundColor Green
    } else {
        throw "npm not found"
    }
}
catch {
    Write-Host "   ERROR: npm not installed (usually comes with Node.js)" -ForegroundColor Red
    $allGood = $false
}

# Check Rust
Write-Host "Checking Rust..." -ForegroundColor Yellow
try {
    $rustVersion = rustc --version 2>$null
    if ($rustVersion) {
        $versionMatch = $rustVersion -match "rustc (\d+\.\d+)"
        if ($versionMatch) {
            $rustVer = [version]$matches[1]
            if ($rustVer -ge [version]"1.70") {
                Write-Host "   OK: $rustVersion" -ForegroundColor Green
            } else {
                Write-Host "   WARNING: $rustVersion - Recommend 1.70+" -ForegroundColor Yellow
                if ($Fix) {
                    Write-Host "   Updating Rust..." -ForegroundColor Cyan
                    rustup update
                }
            }
        }
    } else {
        throw "Rust not found"
    }
}
catch {
    Write-Host "   ERROR: Rust not installed" -ForegroundColor Red
    if ($Fix) {
        Write-Host "   Trying to install Rust..." -ForegroundColor Cyan
        try {
            winget install Rustlang.Rustup --accept-package-agreements --accept-source-agreements
            Write-Host "   OK: Rust installed, please restart terminal" -ForegroundColor Green
        }
        catch {
            Write-Host "   ERROR: Auto-install failed, please install manually" -ForegroundColor Red
        }
    } else {
        Write-Host "   TIP: winget install Rustlang.Rustup" -ForegroundColor Cyan
        Write-Host "   TIP: or visit https://rustup.rs/" -ForegroundColor Cyan
    }
    $allGood = $false
}

# Check Cargo
Write-Host "Checking Cargo..." -ForegroundColor Yellow
try {
    $cargoVersion = cargo --version 2>$null
    if ($cargoVersion) {
        Write-Host "   OK: $cargoVersion" -ForegroundColor Green
    } else {
        throw "Cargo not found"
    }
}
catch {
    Write-Host "   ERROR: Cargo not installed (usually comes with Rust)" -ForegroundColor Red
    $allGood = $false
}

# Check Git
Write-Host "Checking Git..." -ForegroundColor Yellow
try {
    $gitVersion = git --version 2>$null
    if ($gitVersion) {
        Write-Host "   OK: $gitVersion" -ForegroundColor Green
    } else {
        throw "Git not found"
    }
}
catch {
    Write-Host "   ERROR: Git not installed" -ForegroundColor Red
    if ($Fix) {
        Write-Host "   Trying to install Git..." -ForegroundColor Cyan
        try {
            winget install Git.Git --accept-package-agreements --accept-source-agreements
            Write-Host "   OK: Git installed, please restart terminal" -ForegroundColor Green
        }
        catch {
            Write-Host "   ERROR: Auto-install failed, please install manually" -ForegroundColor Red
        }
    } else {
        Write-Host "   TIP: winget install Git.Git" -ForegroundColor Cyan
        Write-Host "   TIP: or visit https://git-scm.com/" -ForegroundColor Cyan
    }
    $allGood = $false
}

# Summary
Write-Host ""
Write-Host "================================" -ForegroundColor Gray
if ($allGood) {
    Write-Host "SUCCESS: Environment check completed - All required tools are installed!" -ForegroundColor Green
    Write-Host ""
    Write-Host "Next steps:" -ForegroundColor Cyan
    Write-Host "   1. Run .\Setup-MVP.ps1 to create project" -ForegroundColor White
    Write-Host "   2. Follow documentation to implement code" -ForegroundColor White
    Write-Host "   3. Run .\Build-MVP.ps1 -Mode dev to start dev server" -ForegroundColor White
} else {
    Write-Host "FAILED: Environment check failed - Please install missing tools" -ForegroundColor Red
    Write-Host ""
    Write-Host "Tips:" -ForegroundColor Cyan
    Write-Host "   - Run .\Check-Environment-Simple.ps1 -Fix to auto-fix" -ForegroundColor White
    Write-Host "   - Restart PowerShell terminal after installation" -ForegroundColor White
}

Write-Host ""
