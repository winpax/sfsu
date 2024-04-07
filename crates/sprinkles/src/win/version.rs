// mod internal {
//     extern "C" {
//         pub fn is_windows_10_or_later() -> std::ffi::c_int;
//     }
// }

// #[must_use]
// pub fn is_windows_10_or_later() -> bool {
//     let internal_result = unsafe { internal::is_windows_10_or_later() };

//     internal_result == 1
// }
