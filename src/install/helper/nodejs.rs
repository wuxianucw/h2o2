use std::time::{Duration, SystemTime};
use tokio::sync::mpsc;
use url::Url;

pub fn get_binary_info() -> (&'static str, &'static str) {
    #[cfg(all(windows, target_arch = "x86"))]
    return (
        "-x86.msi",
        "b5bea503f45058a6acd0900bfe7e52deba12dcc1769808eece93b42bce40c7d8",
    );

    #[cfg(all(windows, target_arch = "x86_64"))]
    return (
        "-x64.msi",
        "964e36aa518b17ab04c3a49a0f5641a6bd8a9dc2b57c18272b6f90edf026f5dc",
    );

    #[cfg(all(
        target_os = "linux",
        any(target_arch = "x86_64", target_arch = "aarch64")
    ))]
    return (
        "-linux-x64.tar.gz",
        "7ef1f7dae52a3ec99cda9cf29e655bc6e61c2c48e496532d83d9f17ea108d5d8",
    );

    #[cfg(all(target_os = "linux", target_arch = "arm"))]
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

#[derive(Default, PartialEq, Eq)]
struct TestResult {
    pub error: u32,
    pub total: Duration,
}

impl TestResult {
    const ATTEMPT_TIMES: u32 = 5;

    pub fn average(&self) -> Duration {
        self.total / (Self::ATTEMPT_TIMES - self.error)
    }

    pub fn is_failed(&self) -> bool {
        self.error == Self::ATTEMPT_TIMES
    }
}

impl PartialOrd for TestResult {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(match self.error.cmp(&other.error) {
            std::cmp::Ordering::Equal => {
                if self.is_failed() {
                    std::cmp::Ordering::Equal
                } else {
                    self.average().cmp(&other.average())
                }
            }
            ord => ord,
        })
    }
}

impl Ord for TestResult {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        match self.error.cmp(&other.error) {
            std::cmp::Ordering::Equal => {
                if self.is_failed() {
                    std::cmp::Ordering::Equal
                } else {
                    self.average().cmp(&other.average())
                }
            }
            ord => ord,
        }
    }
}

pub async fn determine_mirror() -> Option<String> {
    let dists = vec![
        "https://nodejs.org/dist",
        "https://mirrors.tuna.tsinghua.edu.cn/nodejs-release",
        "https://mirrors.cloud.tencent.com/nodejs-release",
    ];
    let testfile = "v14.17.3/SHASUMS256.txt";
    let (tx, mut rx) = mpsc::channel(16);

    for (i, dist) in dists.iter().enumerate() {
        let url = Url::parse(dist).unwrap();
        let url = url.join(testfile).unwrap();
        let tx = tx.clone();
        tokio::spawn(async move {
            for _ in 0..TestResult::ATTEMPT_TIMES {
                let now = SystemTime::now();

                tx.send((
                    i,
                    reqwest::get(url.as_str())
                        .await
                        .map_err(|_| ())
                        .and_then(|_| now.elapsed().map_err(|_| ())),
                ))
                .await
                .expect("mpsc send failed");
            }
        });
    }

    // must drop here, otherwise the receiver will block forever
    std::mem::drop(tx);

    let mut results = vec![
        TestResult::default(),
        TestResult::default(),
        TestResult::default(),
    ];

    while let Some((i, res)) = rx.recv().await {
        let result = &mut results[i];
        if let Ok(t) = res {
            result.total += t;
            log::info!("[Node.js] {} -- {}ms", dists[i], t.as_millis());
        } else {
            result.error += 1;
            log::info!("[Node.js] {} -- FAILED", dists[i]);
        }
    }

    results
        .iter()
        .enumerate()
        .min_by(|(_, a), (_, b)| a.cmp(b))
        .and_then(|(i, r)| {
            if r.is_failed() {
                None
            } else {
                Some(dists[i].to_owned())
            }
        })
}
