# use PowerShell instead of sh:
set shell := ["pwsh.exe", "-NoProfile",  "-c"]

setup:
    just pre-commit

remove-pre-commit:
    pre-commit uninstall
    pre-commit uninstall --hook-type commit-msg
    pre-commit uninstall --hook-type pre-push

pre-commit:
    pre-commit install
    pre-commit install --hook-type commit-msg
    pre-commit install --hook-type pre-push
