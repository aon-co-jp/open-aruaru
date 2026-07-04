$ErrorActionPreference = 'Stop'
Write-Host 'Building aruaru-desktop thin UI and Rust GraphQL gateway...'
Push-Location $PSScriptRoot\..\..
cargo build --release -p aruaru-gql-gateway
Pop-Location
Push-Location $PSScriptRoot\..\..\apps\aruaru-desktop
npm install
npm run build
Pop-Location
Write-Host 'Next: package target/release/aruaru-gql-gateway.exe and apps/aruaru-desktop/dist with Inno Setup or MSIX.'
