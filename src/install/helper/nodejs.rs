pub fn get_binary_info() -> (&'static str, &'static str) {
    #[cfg(windows)]
    #[cfg(target_arch = "x86")]
    return (
        "-x86.msi",
        "b5bea503f45058a6acd0900bfe7e52deba12dcc1769808eece93b42bce40c7d8",
    );

    #[cfg(windows)]
    #[cfg(target_arch = "x86_64")]
    return (
        "-x64.msi",
        "964e36aa518b17ab04c3a49a0f5641a6bd8a9dc2b57c18272b6f90edf026f5dc",
    );

    #[cfg(target_os = "linux")]
    #[cfg(target_arch = "x86_64")]
    return (
        "-linux-x64.tar.gz",
        "7ef1f7dae52a3ec99cda9cf29e655bc6e61c2c48e496532d83d9f17ea108d5d8",
    );

    #[cfg(target_os = "linux")]
    #[cfg(target_arch = "aarch64")]
    return (
        "-linux-x64.tar.gz",
        "7ef1f7dae52a3ec99cda9cf29e655bc6e61c2c48e496532d83d9f17ea108d5d8",
    );

    #[cfg(target_os = "linux")]
    #[cfg(target_arch = "arm")]
    return (
        "-linux-arm64.tar.gz",
        "784ede0c9faa4a71d77659918052cca39981138edde2c799ffdf2b4695c08544",
    );

    #[cfg(target_os = "macos")]
    return (
        "-darwin-x64.tar.gz",
        "522f85db1d1fe798cba5f601d1bba7b5203ca8797b2bc934ff6f24263f0b7fb2",
    );
}
