use std::f64;
use std::rc::Rc;
use std::cell::RefCell;

use slog::Logger;

use gdk_sys;
use gdk::{EventButton, EventMotion};
use gtk::{Inhibit, DrawingArea, WidgetExt};
use cairo::Context;

use model::{Ring, Color};

pub type Point = (f64, f64);

pub type RGB = (f64, f64, f64);

pub struct RingColor {
    pub stroke: RGB,
    pub fill: RGB
}

impl From<Option<Color>> for RingColor {
    fn from(cell: Option<Color>) -> RingColor {
        match cell {
            Some(Color::Blue) => RingColor {
                stroke: (0., 0., 0.),
                fill: (0.118, 0.565, 1.000)
            },
            Some(Color::Green) => RingColor {
                stroke: (0., 0., 0.),
                fill: (0.196, 0.804, 0.196)
            },
            Some(Color::Red) => RingColor {
                stroke: (0., 0., 0.),
                fill: (0.698, 0.133, 0.133)
            },
            Some(Color::Yellow) => RingColor {
                stroke: (0., 0., 0.),
                fill: (0.957, 0.643, 0.376) 
            },
            None => RingColor {
                stroke: (0., 0., 0.),
                fill: (1., 1., 1.)
            }
        }
    }
}

// Sadly I had to wrap almost everything inside a RefCell
// to make Gtk and borrow checker happy
pub struct Cell {
    pub drawing_area: Rc<RefCell<DrawingArea>>,
    hover: RefCell<Option<Ring>>,
    top: RefCell<Option<Color>>,
    middle: RefCell<Option<Color>>,
    bottom: RefCell<Option<Color>>,
    pub callback: RefCell<Option<Box<Fn(Ring)>>>,
    log: Logger
}

impl Cell {
    pub fn new(log: &Logger) -> Rc<Cell> {
        let area = Rc::new(RefCell::new(DrawingArea::new()));
        let area1 = area.clone();

        let cell = Cell {
            drawing_area: area,
            hover: RefCell::new(None),
            top: RefCell::new(None),
            middle: RefCell::new(None),
            bottom: RefCell::new(None),
            callback: RefCell::new(None),
            log: log.new(None)
        };

        let cell = Rc::new(cell);

        let area1 = area1.borrow();

        // This can't possibly be the correct way to do this
        let mask = gdk_sys::GDK_POINTER_MOTION_MASK
            | gdk_sys::GDK_BUTTON_PRESS_MASK;
        area1.add_events(mask.bits() as i32);

        area1.set_hexpand(true);
        area1.set_vexpand(true);

        // These are kind of scary
        let cell1 = cell.clone();
        area1.connect_draw(
            move |da, ctx| Cell::draw(&cell1, da, ctx));
        let cell2 = cell.clone();
        area1.connect_button_press_event(
            move |da, ev| Cell::button_press_handler(&*cell2, da, ev));
        let cell3 = cell.clone();
        area1.connect_motion_notify_event(
            move |da, ev| Cell::motion_event_handler(&*cell3, da, ev));

        cell
    }

    pub fn draw(&self, area: &DrawingArea, ctx: &Context) -> Inhibit {
        debug!(self.log, "draw event");

        fn draw_ring(ctx: &Context,
                     width: f64,
                     height: f64,
                     ring: Ring,
                     color: RingColor) {
            let (x, y) = (width / 2.0, height / 2.0);
            let radius = ring.radius(width, height);
            ctx.save();
            ctx.set_line_width(2.0);
            let (r, g, b) = color.stroke;
            ctx.set_source_rgb(r, g, b);
            ctx.translate(x, y);
            ctx.arc(0., 0., radius, 0., 2. * f64::consts::PI);
            ctx.stroke_preserve();
            let (r, g, b) = color.fill;
            ctx.set_source_rgb(r, g, b);
            ctx.fill();
            ctx.restore();
        }

        ctx.set_source_rgb(1.0, 1.0, 1.0);
        ctx.fill();

        let width = area.get_allocated_width() as f64;
        let height = area.get_allocated_height() as f64;

        // TODO: Handle hover

        draw_ring(ctx, width, height, Ring::Bottom, (*self.bottom.borrow()).into());
        draw_ring(ctx, width, height, Ring::Middle, (*self.middle.borrow()).into());
        draw_ring(ctx, width, height, Ring::Top, (*self.top.borrow()).into());

        Inhibit(true)
    }

    pub fn button_press_handler(&self,
                            area: &DrawingArea,
                            event: &EventButton) -> Inhibit {
        debug!(self.log, "button press event");

        let point = event.get_position();

        let width = area.get_allocated_width() as f64;
        let height = area.get_allocated_height() as f64;
        let center = (width / 2., height / 2.);

        let callback = &*self.callback.borrow();
        let callback = callback.as_ref().unwrap();

        // point_inside_ring(center, width, height, point)

        match point_inside_ring(center, width, height, point) {
            Some(Ring::Top) => {
                debug!(self.log, "clicked on a ring"; "ring" => "top");
                callback(Ring::Top);
            },
            Some(Ring::Middle) => {
                debug!(self.log, "clicked on a ring"; "ring" => "middle");
                callback(Ring::Middle);
            },
            Some(Ring::Bottom) => {
                debug!(self.log, "clicked on a ring"; "ring" => "bottom");
                callback(Ring::Bottom);
            },
            _ => {}
        }

        area.queue_draw();

        Inhibit(true)
    }

    pub fn motion_event_handler(&self,
                            area: &DrawingArea,
                            event: &EventMotion) -> Inhibit {
        let point = event.get_position();

        let width = area.get_allocated_width() as f64;
        let height = area.get_allocated_height() as f64;
        let center = (width / 2., height / 2.);

        match &point_inside_ring(center, width, height, point) {
            r@&Some(Ring::Top) if *self.hover.borrow() != *r => {
                debug!(self.log, "moved to a new ring"; "ring" => "top");
                *self.hover.borrow_mut() = Some(Ring::Top);
                area.queue_draw();
            },
            r@&Some(Ring::Middle) if *self.hover.borrow() != *r => {
                debug!(self.log, "moved to a new ring"; "ring" => "middle");
                *self.hover.borrow_mut() = Some(Ring::Middle);
                area.queue_draw();
            },
            r@&Some(Ring::Bottom) if *self.hover.borrow() != *r => {
                debug!(self.log, "moved to a new ring"; "ring" => "bottom");
                *self.hover.borrow_mut() = Some(Ring::Bottom);
                area.queue_draw();
            },
            r@&None if *self.hover.borrow() != *r => {
                debug!(self.log, "moved outside the ring stack");
                *self.hover.borrow_mut() = None;
                area.queue_draw();
            },
            _ => {}
        }

        Inhibit(true)
    }

    pub fn set_ring(&self, ring: Ring, color: Option<Color>) {
        let mut ring = match ring {
            Ring::Top => self.top.borrow_mut(),
            Ring::Middle => self.middle.borrow_mut(),
            Ring::Bottom => self.bottom.borrow_mut()
        };
        let mut ring = &mut *ring;
        *ring = color;
    }

}

fn point_inside_circle((circle_x, circle_y): Point, radius: f64, (x, y): Point) -> bool {
    ((x - circle_x).abs().powi(2) + (y - circle_y).abs().powi(2)).sqrt() < radius
}

fn point_inside_ring(center: Point,
                     width: f64,
                     height: f64,
                     point: Point) -> Option<Ring> {
    if point_inside_circle(center, Ring::Top.radius(width, height), point) {
        Some(Ring::Top)
    } else if point_inside_circle(center, Ring::Middle.radius(width, height), point) {
        Some(Ring::Middle)
    } else if point_inside_circle(center, Ring::Bottom.radius(width, height), point) {
        Some(Ring::Bottom)
    } else {
        None
    }
}
