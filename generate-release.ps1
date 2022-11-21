function Out-Hashes($Path) {
    $Hash = Get-FileHash $Path -Algorithm SHA256
    $Hash.Hash | Out-File "$Path.sha256"
}

cargo b -r --target x86_64-pc-windows-msvc
cargo b -r --target i686-pc-windows-msvc

if (Test-Path release) {
    rm -r release
}

mkdir release
mkdir release/64bit
mkdir release/32bit

cp target/x86_64-pc-windows-msvc/release/*.exe release/64bit

cd release/64bit;

7z a 'dl-x86_64' '*.exe'

Out-Hashes dl-x86_64.7z

mv *.7z* ../

cd ../..

cp target/i686-pc-windows-msvc/release/*.exe release/32bit

cd release/32bit

7z a 'dl-i686' '*.exe'

Out-Hashes dl-i686.7z

mv *.7z* ../

cd ../..
