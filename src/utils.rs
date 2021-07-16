use std::process::Output;

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
        let nodejs_version_requirement = $crate::version_req!(nodejs);
        if !nodejs_version_requirement.matches($version) {
            ::log::warn!(
                "Hydro 需要 `Node.js {}`，当前版本可能无法正常工作。 \
                Hydro requires `Node.js {}`, the current version may not work properly.",
                &nodejs_version_requirement,
                &nodejs_version_requirement,
            );
            false
        } else {
            true
        }
    }};
    (mongodb, $version:expr, warn) => {{
        let mongodb_version_requirement = $crate::version_req!(mongodb);
        if !mongodb_version_requirement.matches($version) {
            ::log::warn!(
                "Hydro 需要 `MongoDB {}`，当前版本可能无法正常工作。 \
                Hydro requires `MongoDB {}`, the current version may not work properly.",
                &mongodb_version_requirement,
                &mongodb_version_requirement,
            );
            false
        } else {
            true
        }
    }};
}
