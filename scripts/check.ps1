$ErrorActionPreference = 'Stop'
$failed = [System.Collections.Generic.List[string]]::new()

function Invoke-Step {
    param([string]$Name, [scriptblock]$Action)
    Write-Host "==> $Name" -ForegroundColor Cyan
    & $Action
    if ($LASTEXITCODE -ne 0) {
        Write-Host "FAILED: $Name (exit $LASTEXITCODE)" -ForegroundColor Red
        $script:failed.Add($Name)
    }
}

Invoke-Step 'formatting (cargo fmt --check)' { cargo fmt --all -- --check }
Invoke-Step 'linting (cargo clippy -D warnings)' {
    cargo clippy --all-targets --all-features -- -D warnings
}
Invoke-Step 'tests (cargo test)' { cargo test --all-features }
Invoke-Step 'docs (cargo doc)' {
    $env:RUSTDOCFLAGS = '-D warnings'
    cargo doc --no-deps --all-features
}

if ($failed.Count -gt 0) {
    Write-Host "`nFailed: $($failed -join ', ')" -ForegroundColor Red
    exit 1
}
Write-Host "`nAll fast checks passed." -ForegroundColor Green
