# use PowerShell instead of sh:
set shell := ["pwsh.exe", "-c"]

check:
    cargo c

build-all:
    just build x86_64-pc-windows-msvc
    just build i686-pc-windows-msvc

build TARGET:
    cargo auditable b -r --target {{ TARGET }}

release: build-all
    if (Test-Path "release") { rm -r "release" -Force -ErrorAction Ignore }
    mkdir "release"

    cp "./target/x86_64-pc-windows-msvc/release/sfsu.exe" "./release/sfsu.exe"
    7z a "./release/dl-x86_64" "./release/sfsu.exe"
    mv "./release/sfsu.exe" "./release/sfsu-x86_64.exe"
    just export-hash x86_64

    cp "./target/i686-pc-windows-msvc/release/sfsu.exe" "./release/sfsu.exe"
    7z a "./release/dl-i686" "./release/sfsu.exe"
    mv "./release/sfsu.exe" "./release/sfsu-i686.exe"
    just export-hash i686

export-hash TARGET:
    python scripts/hash.py './release/dl-{{ TARGET }}.7z'
    python scripts/hash.py './release/sfsu-{{ TARGET }}.exe'
