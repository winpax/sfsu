check:
    cargo c

build-all:
    just build x86_64-pc-windows-gnu
    just build i686-pc-windows-gnu

build TARGET:
    cargo b -r --target {{ TARGET }}

release: build-all
    mkdir release
    mkdir release/64bit
    mkdir release/32bit

    cp target/x86_64-pc-windows-gnu/release/*.exe release/64bit
    cp target/i686-pc-windows-gnu/release/*.exe release/32bit

    7z a './release/dl-x86_64.zip' './release/64bit/*.exe'
    7z a './release/dl-i686.zip' './release/32bit/*.exe'
