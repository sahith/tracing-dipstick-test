use std::thread;
use std::time::Duration;

use dipstick::{AtomicBucket, ScheduleFlush, Stream};
use log::LevelFilter;
use tracing::{debug, info, info_span, subscriber};
use tracing_dipstick::DipstickLayer;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::Registry;

fn main() {
    /*
     * We use the log-always integration of tracing here and route that to the env logger, that has
     * INFO enabled by default and can override by RUST_LOG to something else.
     *
     * We could use tracing_subscriber::fmt, *but* the EnvFilter there unfortunately disables
     * events/spans for the whole stack, not for logging only. And we want all the metrics while we
     * want only certain level of events.
     */
    env_logger::builder()
        .filter_level(LevelFilter::Info)
        .parse_default_env()
        .init();

    let root = AtomicBucket::new();
    root.stats(dipstick::stats_all);
    root.drain(Stream::write_to_stdout());
    let _flush = root.flush_every(Duration::from_secs(5));

    let bridge = DipstickLayer::new(root);
    let subscriber = Registry::default().with(bridge);

    subscriber::set_global_default(subscriber).unwrap();

    const CNT: usize = 10;
    let _yaks = info_span!("Shaving yaks", cnt = CNT, metrics.scope = "shaving").entered();
    for i in 0..CNT {
        let _this_yak = info_span!(
            "Yak",
            metrics.gauge.order = i,
            metrics.scope = "yak",
            metrics.timer = "time",
            metrics.level = "active"
        )
        .entered();
        debug!(metrics.counter = "started", "Starting shaving");
        thread::sleep(Duration::from_millis(60));
        debug!(metrics.counter = "done", metrics.counter.legs = 4, "Shaving done");
    }

    info!("Sleep for 10 seconds");
    thread::sleep(Duration::from_secs(10));
}