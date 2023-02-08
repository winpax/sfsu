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
    hash="$(sha256sum './release/dl-x86_64.7z')"
    echo "${hash:0:64}" > "./release/dl-x86_64.7z.sha256"
    7z a "./release/dl-i686" "./target/i686-pc-windows-gnu/release/*.exe"
    sha256sum "./release/dl-i686.7z" > "./release/dl-i686.7z.sha256"
