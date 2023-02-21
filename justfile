check:
    cargo c

install-targets:
    rustup target add x86_64-pc-windows-gnu
    rustup target add i686-pc-windows-gnu

build-all:
    just build x86_64-pc-windows-gnu
    just build i686-pc-windows-gnu

build TARGET:
    cargo b -r --target {{ TARGET }}

release: build-all
    mkdir -p "release"

    7z a "./release/dl-x86_64" "./target/x86_64-pc-windows-gnu/release/*.exe"
    just export-hash x86_64

    7z a "./release/dl-i686" "./target/i686-pc-windows-gnu/release/*.exe"
    just export-hash i686

export-hash TARGET:
    python scripts/hash.py './release/dl-{{ TARGET }}.7z'
