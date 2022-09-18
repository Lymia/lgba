/// Not public API.
#[doc(hidden)]
#[macro_export]
macro_rules! __lgba_macro_export__marker {
    ($name:ident, $str:expr) => {
        #[no_mangle]
        pub static $name: &'static str = concat!($str, "\0");
    };
}

__lgba_macro_export__marker!(__lgba_exh_lib_cname, env!("CARGO_PKG_NAME"));
__lgba_macro_export__marker!(__lgba_exh_lib_cver, env!("CARGO_PKG_VERSION"));

#[no_mangle]
pub unsafe extern "C" fn __lgba_init_rust() {}

#[no_mangle]
pub unsafe extern "C" fn __lgba_main_func_returned() -> ! {
    panic!("Internal error: Main function returned?")
}
