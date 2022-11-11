cargo b -r --target x86_64-pc-windows-gnu
cargo b -r --target i686-pc-windows-gnu

rm -r release

mkdir release
mkdir release/64bit
mkdir release/32bit

cp target/x86_64-pc-windows-gnu/release/*.exe release/64bit

cd release/64bit;

7z a 'all-x86_64' '*.exe'

$Hash = Get-FileHash all-x86_64.7z
$Hash | Out-File all-x86_64.7z.sha256

mv *.7z* ../

cd ../..

cp target/x86_64-pc-windows-gnu/release/*.exe release/32bit

cd release/32bit

7z a 'all-i686' '*.exe'

$Hash = Get-FileHash all-i686.7z
$Hash | Out-File all-i686.7z.sha256

mv *.7z* ../

cd ../..