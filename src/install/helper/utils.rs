use std::time::{Duration, SystemTime};
use tokio::sync::mpsc;
use url::Url;

use crate::Com;

#[derive(Clone, Default, PartialEq, Eq)]
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

pub async fn determine_mirror(
    com: Com,
    mirrors: Vec<&str>,
    testfile: Option<&str>,
) -> Option<String> {
    let (tx, mut rx) = mpsc::channel(16);

    for (i, mirror) in mirrors.iter().enumerate() {
        let url = match testfile {
            Some(file) => Url::parse(mirror).unwrap().join(file),
            None => Url::parse(mirror),
        }
        .unwrap();
        let tx = tx.clone();
        tokio::spawn(async move {
            for _ in 0..TestResult::ATTEMPT_TIMES {
                let now = SystemTime::now();

                tx.send((
                    i,
                    reqwest::get(url.clone())
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

    let mut results = vec![TestResult::default(); mirrors.len()];

    while let Some((i, res)) = rx.recv().await {
        let result = &mut results[i];
        if let Ok(t) = res {
            result.total += t;
            log::info!("[{}] {} -- {}ms", com, mirrors[i], t.as_millis());
        } else {
            result.error += 1;
            log::info!("[{}] {} -- FAILED", com, mirrors[i]);
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
                Some(mirrors[i].to_owned())
            }
        })
}
