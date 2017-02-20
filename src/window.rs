use std::rc::Rc;
use std::cell::RefCell;

use slog::Logger;

use gio;
use gtk;
use gio::ApplicationExt;
use gtk::{Application, Window, WindowType, HeaderBar, Grid,
          MessageDialog, DialogExt, WindowExt, WidgetExt,
          ContainerExt};

use model::{Board, Color, Ring, Location, location_to_index, index_to_location};
use cell::Cell;

const APPLICATION_TITLE: &'static str = "Three";
const APPLICATION_ID: &'static str = "org.three";

pub struct MainWindow {
    application: Application,
    window: Rc<Window>,
    header_bar: HeaderBar,
    cells: Vec<Rc<Cell>>,
    board: RefCell<Board>,
    current_color: RefCell<Color>,
    log: Logger
}

const STARTING_COLOR: Color = Color::Blue;

impl MainWindow {
    pub fn new(log: Logger) -> Rc<MainWindow> {
        let app = Application::new(Some(APPLICATION_ID),
                                   gio::APPLICATION_FLAGS_NONE)
            .unwrap();
        let win = Rc::new(Window::new(WindowType::Toplevel));

        win.set_title(APPLICATION_TITLE);

        let header = HeaderBar::new();
        header.set_title(Some(APPLICATION_TITLE));
        header.set_show_close_button(true);
        win.set_titlebar(Some(&header));
        win.set_default_size(500, 600);

        let grid = Grid::new();
        let mut cells = vec![];

        for y in 0..3 {
            for x in 0..3 {
                let cell = Cell::new(&log);
                grid.attach(&*cell.drawing_area.borrow(), x, y, 1, 1);
                cells.push(cell);
            }
        }

        win.add(&grid);

        let win1 = win.clone();
        app.connect_activate(move |app| {
            app.add_window(&*win1);
            win1.show_all();
        });

        let main_win = MainWindow {
            application: app,
            window: win,
            header_bar: header,
            cells: cells,
            board: RefCell::new(Board::empty()),
            current_color: RefCell::new(STARTING_COLOR),
            log: log
        };

        let main_win = Rc::new(main_win);

        for (i, cell) in main_win.cells.iter().enumerate() {
            // I think this creates a reference cycle
            // not good but in this case it doesn't really matter
            let main_win1 = main_win.clone();
            let mut callback = cell.callback.borrow_mut();
            let pos = index_to_location(3, i);
            *callback = Some(Box::new(
                move |ring| MainWindow::ring_pressed_handler(&*main_win1, pos, ring)));
        }

        MainWindow::update_turn_indicator(&*main_win);

        main_win
    }

    pub fn run(&self) {
        debug!(self.log, "run");
        self.application.run(0, &[]);
    }

    fn ring_pressed_handler(&self, (x, y): Location, ring: Ring) {
        debug!(self.log, "ring pressed event"; "x" => x, "y" => y, "ring" => format!("{:?}", ring));
        if self.board.borrow().get_ring((x as usize, y as usize), ring).is_none() {
            let current_color = *self.current_color.borrow();
            self.board.borrow_mut().set_ring((x as usize, y as usize), ring, current_color);
            let cell = self.cell_at((x, y));
            cell.set_ring(ring, Some(current_color));
            self.change_turn();
            self.check_state();
        }
    }

    fn change_turn(&self) {
        let current = *self.current_color.borrow();
        *self.current_color.borrow_mut() = next_color(current);
        self.update_turn_indicator();
    }

    fn update_turn_indicator(&self) {
        self.header_bar.set_subtitle(Some(
            &format!("{:?}'s turn", self.current_color.borrow())));
    }

    fn check_state(&self) {
        let winner = self.board.borrow().winner();
        if let Some(color) = winner {
            debug!(self.log, "winner"; "color" => format!("{:?}", winner));
            let mut flags = gtk::DIALOG_MODAL;
            flags.insert(gtk::DIALOG_DESTROY_WITH_PARENT);
            flags.insert(gtk::DIALOG_USE_HEADER_BAR);
            let type_ = gtk::MessageType::Info;
            let buttons = gtk::ButtonsType::Ok;
            let dialog = MessageDialog::new::<Window>(Some(&*self.window),
                                            flags,
                                            type_,
                                            buttons,
                                            &format!("{:?} wins!", color));
            dialog.connect_response(move |dialog, _| {
                dialog.destroy();
            });
            dialog.show();
        }
    }

    fn cell_at(&self, (x, y): Location) -> Rc<Cell> {
        self.cells[location_to_index(3, (x as usize, y as usize))].clone()
    }

}

fn next_color(color: Color) -> Color {
    match color {
        Color::Blue => Color::Green,
        Color::Green => Color::Red,
        Color::Red => Color::Yellow,
        Color::Yellow => Color::Blue
    }
}


