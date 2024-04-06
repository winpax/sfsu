#include <stdbool.h>
#include <windows.h>
#include <VersionHelpers.h>

BOOL is_windows_10_or_later() {
    return IsWindows10OrGreater();
}