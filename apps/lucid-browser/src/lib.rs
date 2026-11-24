pub mod app;

#[cfg(target_os = "windows")]
#[unsafe(no_mangle)]
pub unsafe extern "C" fn RunWinMain(
    instance: cef::sys::HINSTANCE,
    _command_line: *const u8,
    _command_show: i32,
    sandbox_info: *mut u8,
) -> i32 {
    let _ = cef::api_hash(cef::sys::CEF_API_VERSION_LAST, 0);
    let main_args = cef::MainArgs { instance };
    let args = cef::args::Args::from(main_args);
    let main_args = args.as_main_args();

    if let Some(exit_code) = app::cef_host::execute_subprocess(main_args, sandbox_info) {
        return exit_code;
    }

    match app::run_lucid(instance, sandbox_info) {
        Ok(()) => 0,
        Err(error) => {
            eprintln!("Lucid exited with an error: {error}");
            1
        }
    }
}
