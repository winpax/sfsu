check:
    cargo c

build-all:
    just build x86_64-pc-windows-gnu
    just build i686-pc-windows-gnu

build TARGET:
    cargo b -r --target {{ TARGET }}

release: build-all
    mkdir release

    7z a './release/dl-x86_64.zip' './target/x86_64-pc-windows-gnu/release/*.exe'
    7z a './release/dl-i686.zip' './target/i686-pc-windows-gnu/release/*.exe'
