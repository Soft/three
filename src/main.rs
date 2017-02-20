#[macro_use]
extern crate itertools;

#[macro_use]
extern crate slog;
extern crate slog_term;

extern crate gio;
extern crate gdk;
extern crate gdk_sys;
extern crate gtk;
extern crate cairo;

use slog::Logger;
use slog::DrainExt;

mod model;
mod window;
mod cell;

use window::MainWindow;

fn main() {
    gtk::init().unwrap();

    let drain = slog_term::streamer().compact().build().fuse();
    let log = Logger::root(drain, None);

    let app = MainWindow::new(log);
    app.run();
}


