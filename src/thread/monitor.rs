//! Monitor mode for progress bars.
//! 
//! In monitor mode progress bar is refreshed in specific intervals.
//! Default monitor modes may not fit in many cases.
//! So it is recommended to create a custom monitor mode.
//! The basic idea behind monitor mode is to create a separate thread for updating progress bar
//! which can be achieved by following code.
//! 
//! ```
//! use kdam::{Bar, BarExt};
//! use std::sync::{Arc, Mutex};
//! use std::thread;
//! 
//! fn custom_monitor(pb: Bar, maxinterval: f32) -> (Arc<Mutex<Bar>>, thread::JoinHandle<()>) {
//!     let pb_arc = Arc::new(Mutex::new(pb));
//!     let pb_arc_clone = pb_arc.clone();
//! 
//!     let handle = thread::spawn(move || loop {
//!         thread::sleep(std::time::Duration::from_secs_f32(maxinterval));
//!         let mut pb_monitor = pb_arc_clone.lock().unwrap();
//! 
//!         if pb_monitor.completed() {
//!             break;
//!         }
//! 
//!         pb_monitor.refresh();
//!     });
//! 
//!     (pb_arc, handle)
//! }
//! ```

use crate::progress::{Bar, BarExt, RichProgress};
use std::sync::{Arc, Mutex};
use std::thread;

/// Monitor mode for [Bar](crate::Bar)
///
/// # Example
///
/// ```no_run
/// use kdam::{tqdm, BarExt};
///
/// let pb = tqdm!(total = 100, force_refresh = true);
/// let (pb_arc, monitor_thread) = kdam::monitor::bar(pb, 1.0);
///
/// for _ in 0..100 {
///     pb_arc.lock().unwrap().update(1);
///     std::thread::sleep(std::time::Duration::from_secs_f32(3.0));
/// }
///
/// monitor_thread.join().unwrap();
/// eprint!("\n");
/// ```
pub fn bar(pb: Bar, maxinterval: f32) -> (Arc<Mutex<Bar>>, thread::JoinHandle<()>) {
    let pb_arc = Arc::new(Mutex::new(pb));
    let pb_arc_clone = pb_arc.clone();

    let handle = thread::spawn(move || loop {
        thread::sleep(std::time::Duration::from_secs_f32(maxinterval));
        let mut pb_monitor = pb_arc_clone.lock().unwrap();

        if pb_monitor.completed() {
            break;
        }

        pb_monitor.refresh();
    });

    (pb_arc, handle)
}

/// Monitor mode for [RichProgress](crate::RichProgress). See [monitor::bar](crate::monitor::bar) for example usecase.
pub fn rich(
    pb: RichProgress,
    maxinterval: f32,
) -> (Arc<Mutex<RichProgress>>, thread::JoinHandle<()>) {
    let pb_arc = Arc::new(Mutex::new(pb));
    let pb_arc_clone = pb_arc.clone();

    let handle = thread::spawn(move || loop {
        thread::sleep(std::time::Duration::from_secs_f32(maxinterval));
        let mut pb_monitor = pb_arc_clone.lock().unwrap();

        if pb_monitor.pb.completed() {
            break;
        }

        pb_monitor.refresh();
    });

    (pb_arc, handle)
}
