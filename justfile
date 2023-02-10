check:
    cargo c

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
    # set HASH=""
    # echo "$(sha256sum './release/dl-{{ TARGET }}.7z')"
    # echo "$HASH"
    echo "${$(sha256sum './release/dl-{{ TARGET }}.7z'):0:64-0}"