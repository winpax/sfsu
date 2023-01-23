# Uses GNU rather than MSVC because for some reason the MSVC build is picked up as a trojan

function Out-Hashes($Path)
{
    $Hash = Get-FileHash $Path -Algorithm SHA256
    $Hash.Hash | Out-File "$Path.sha256"
}

cargo b -r --target x86_64-pc-windows-gnu
cargo b -r --target i686-pc-windows-gnu

if (Test-Path release)
{
    Remove-Item -r release
}

mkdir release
mkdir release/64bit
mkdir release/32bit

Copy-Item target/x86_64-pc-windows-gnu/release/*.exe release/64bit

Set-Location release/64bit;

7z a 'dl-x86_64' '*.exe'

Out-Hashes dl-x86_64.7z

Move-Item *.7z* ../

Set-Location ../..

Copy-Item target/i686-pc-windows-gnu/release/*.exe release/32bit

Set-Location release/32bit

7z a 'dl-i686' '*.exe'

Out-Hashes dl-i686.7z

Move-Item *.7z* ../

Set-Location ../..
