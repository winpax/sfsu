cargo b -r --target x86_64-pc-windows-gnu
cargo b -r --target i686-pc-windows-gnu

rm -r release

mkdir release
mkdir release/64bit
mkdir release/32bit

cp target/x86_64-pc-windows-gnu/release/*.exe release/64bit

cd release/64bit;

$items = Get-ChildItem;
foreach ($i in $items) {
    $NewName = $i.Name.Replace(".exe", "-x86_64.exe");
    mv $i.Name "../$NewName";
}

cd ../..

cp target/x86_64-pc-windows-gnu/release/*.exe release/32bit

cd release/32bit

$items = Get-ChildItem;
foreach ($i in $items) {
    $NewName = $i.Name.Replace(".exe", "-i686.exe");
    mv $i.Name "../$NewName";
}

cd ../

rm -r 64bit
rm -r 32bit

$items = Get-ChildItem;

foreach ($i in $items) {
    $ExeName = $i.Name;
    $HashFile = "$ExeName.sha256"
    touch $HashFile;

    $Hash = Get-FileHash $ExeName;

    $Hash | Out-File $HashFile;
}

cd ..