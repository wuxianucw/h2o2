use data_encoding::HEXLOWER;
use ring::digest::{Context, Digest, SHA256};
use std::{
    fs::File,
    io::{self, BufReader, Read},
    path::Path,
    process::Output,
};

pub fn debug_output(output: &Output) {
    log::debug!("{}", &output.status);
    log::debug!("stdout:\n{}", String::from_utf8_lossy(&output.stdout));
    log::debug!("stderr:\n{}", String::from_utf8_lossy(&output.stderr));
}

#[macro_export]
macro_rules! version_req {
    (nodejs) => {
        ::semver::VersionReq::parse(">=14").expect("Incorrect version requirement pattern")
    };
    (mongodb) => {
        ::semver::VersionReq::parse(">=4").expect("Incorrect version requirement pattern")
    };
}

#[macro_export]
macro_rules! check_version {
    ($com:tt, $version:expr) => {{
        $crate::version_req!($com).matches($version)
    }};
    (nodejs, $version:expr, warn) => {{
        let req = $crate::version_req!(nodejs);
        if !req.matches($version) {
            ::log::warn!(
                "Hydro 需要 `Node.js {}`，当前版本可能无法正常工作。 \
                Hydro requires `Node.js {}`, the current version may not work properly.",
                &req,
                &req,
            );
            false
        } else {
            true
        }
    }};
    (mongodb, $version:expr, warn) => {{
        let req = $crate::version_req!(mongodb);
        if !req.matches($version) {
            ::log::warn!(
                "Hydro 需要 `MongoDB {}`，当前版本可能无法正常工作。 \
                Hydro requires `MongoDB {}`, the current version may not work properly.",
                &req,
                &req,
            );
            false
        } else {
            true
        }
    }};
}

#[macro_export]
macro_rules! maybe_cmd {
    ($cmd:expr) => {
        if cfg!(windows) {
            concat!($cmd, ".cmd")
        } else {
            $cmd
        }
    };
}

fn sha256_digest<R: Read>(mut reader: R) -> io::Result<Digest> {
    let mut context = Context::new(&SHA256);
    let mut buffer = [0; 1024];

    loop {
        let count = reader.read(&mut buffer)?;
        if count == 0 {
            break;
        }
        context.update(&buffer[..count]);
    }

    Ok(context.finish())
}

pub fn sha256_file(file: impl AsRef<Path>) -> io::Result<String> {
    let input = File::open(file)?;
    let reader = BufReader::new(input);
    let digest = sha256_digest(reader)?;
    Ok(HEXLOWER.encode(digest.as_ref()))
}
