//! Graphical user interface for the to-do list, built with GTK 3.
//!
//! Run it with `cargo run -- gui` or `cargo run -- --gui`.

use std::cell::RefCell;
use std::rc::Rc;

use gtk::gdk;
use gtk::gio::ListStore;
use gtk::glib::{self, Cast};
use gtk::pango;
use gtk::prelude::*;
use gtk::{
    Application, ApplicationWindow, Button, Entry, ListBox, ListBoxRow, Label, Orientation,
    ResponseType, Stack,
};

use crate::storage::{
    STORAGE_FILE, TaskStatus, TodoItem, descriptions_match, load_tasks, next_available_id,
    save_tasks,
};

const CSS: &str = r#"
.todo-row { padding: 8px 10px; }
.desc-pending { color: #2c3e50; font-size: 14px; }
.desc-done { color: #95a5a6; font-size: 14px; }
.desc-tbd { color: #b7791f; font-size: 14px; }
.badge {
    color: #ffffff;
    border-radius: 10px;
    padding: 1px 8px;
    margin-left: 8px;
    font-size: 11px;
}
.badge-pending { background-color: #2563eb; }
.badge-done { background-color: #16a34a; }
.badge-tbd { background-color: #d97706; }
.count-label { color: #7f8c8d; }
"#;

#[derive(Clone, Copy, PartialEq, Eq)]
enum Filter {
    All,
    Pending,
    Done,
}

impl Filter {
    fn passes(self, item: &TodoItem) -> bool {
        match self {
            Filter::All => true,
            Filter::Pending => matches!(item.status, TaskStatus::Pending),
            Filter::Done => matches!(item.status, TaskStatus::Done),
        }
    }
}

/// Shared mutable state used by every widget callback.
struct AppState {
    items: Vec<TodoItem>,
    filter: Filter,
    only_unfinished: bool,
}

/// Everything a row callback needs in order to mutate and refresh the UI.
#[derive(Clone)]
struct Handlers {
    state: Rc<RefCell<AppState>>,
    model: ListStore,
    stack: Stack,
    count_label: Label,
}

impl Handlers {
    fn persist(&self) {
        let items = self.state.borrow().items.clone();
        if let Err(error) = save_tasks(&items, STORAGE_FILE) {
            eprintln!("Error saving tasks: {}", error);
        }
    }

    fn refresh(&self) {
        rebuild_list(self);
    }
}

pub fn run() {
    let app = Application::builder()
        .application_id("com.example.todolist.gui")
        .build();

    app.connect_activate(build_ui);
    // `run` returns when the last window closes; ignore the exit code here.
    let _ = app.run();
}

fn build_ui(app: &Application) {
    let provider = gtk::CssProvider::new();
    provider
        .load_from_data(CSS.as_bytes())
        .expect("failed to load CSS");
    gtk::StyleContext::add_provider_for_screen(
        &gdk::Screen::default().expect("failed to get default GDK screen"),
        &provider,
        gtk::STYLE_PROVIDER_PRIORITY_APPLICATION,
    );

    let window = ApplicationWindow::builder()
        .application(app)
        .title("To-Do List")
        .default_width(540)
        .default_height(560)
        .build();

    // --- Top bar: entry + Add button ----------------------------------------
    let entry = Entry::builder()
        .placeholder_text("Add a new task...")
        .hexpand(true)
        .build();

    let add_button = Button::with_label("Add");

    let header = gtk::Box::new(Orientation::Horizontal, 6);
    header.set_margin_start(10);
    header.set_margin_end(10);
    header.set_margin_top(10);
    header.set_margin_bottom(6);
    header.pack_start(&entry, true, true, 0);
    header.pack_end(&add_button, false, false, 0);

    // --- Filter controls ------------------------------------------------------
    let filter_all = Button::with_label("All");
    let filter_pending = Button::with_label("Pending");
    let filter_done = Button::with_label("Done");
    for button in [&filter_all, &filter_pending, &filter_done] {
        button.set_relief(gtk::ReliefStyle::None);
    }

    let filter_bar = gtk::Box::new(Orientation::Horizontal, 4);
    filter_bar.set_margin_start(10);
    filter_bar.pack_start(&filter_all, false, false, 0);
    filter_bar.pack_start(&filter_pending, false, false, 0);
    filter_bar.pack_start(&filter_done, false, false, 0);

    let only_unfinished = gtk::CheckButton::with_label("Show unfinished tasks only");
    only_unfinished.set_margin_start(10);

    // --- Task list ------------------------------------------------------------
    let model = ListStore::new::<ListBoxRow>();
    let list_box = ListBox::new();
    list_box.bind_model(Some(&model), |obj| {
        obj.downcast_ref::<ListBoxRow>()
            .expect("model holds only ListBoxRow items")
            .clone()
            .upcast()
    });
    list_box.set_selection_mode(gtk::SelectionMode::None);

    let scrolled = gtk::ScrolledWindow::builder()
        .hscrollbar_policy(gtk::PolicyType::Never)
        .vscrollbar_policy(gtk::PolicyType::Automatic)
        .child(&list_box)
        .build();

    let empty_label = Label::builder()
        .label("No tasks yet.\nType a task above and press Add.")
        .justify(gtk::Justification::Center)
        .build();

    let stack = Stack::builder()
        .transition_type(gtk::StackTransitionType::Crossfade)
        .build();
    stack.add_named(&scrolled, "list");
    stack.add_named(&empty_label, "empty");
    stack.set_vexpand(true);

    // --- Footer -----------------------------------------------------------------
    let count_label = Label::new(None);
    count_label.style_context().add_class("count-label");
    count_label.set_halign(gtk::Align::Start);
    count_label.set_margin_start(10);
    count_label.set_margin_top(4);
    count_label.set_margin_bottom(6);

    let root = gtk::Box::new(Orientation::Vertical, 0);
    root.pack_start(&header, false, false, 0);
    root.pack_start(&filter_bar, false, false, 0);
    root.pack_start(&only_unfinished, false, false, 0);
    root.pack_start(&stack, true, true, 0);
    root.pack_start(&count_label, false, false, 0);
    window.add(&root);

    // --- Shared state -------------------------------------------------------------
    let handlers = Handlers {
        state: Rc::new(RefCell::new(AppState {
            items: load_tasks(STORAGE_FILE).unwrap_or_default(),
            filter: Filter::All,
            only_unfinished: false,
        })),
        model,
        count_label,
    };

    // Add a task (button click or Enter key).
    let on_add = Rc::new({
        let handlers = handlers.clone();
        let entry = entry.clone();
        move || {
            let text = entry.text().trim().to_string();
            if text.is_empty() {
                return;
            }
            {
                let mut state = handlers.state.borrow_mut();
                let duplicate_id = state
                    .items
                    .iter()
                    .filter(|item| descriptions_match(&item.description, &text))
                    .map(|item| item.id)
                    .last();
                let id = duplicate_id.unwrap_or_else(|| next_available_id(&state.items));
                state.items.push(TodoItem {
                    id,
                    description: text,
                    status: TaskStatus::Pending,
                    tag: None,
                });
            }
            entry.set_text("");
            handlers.persist();
            handlers.refresh();
        }
    });

    add_button.connect_clicked({
        let on_add = Rc::clone(&on_add);
        move |_| on_add()
    });
    entry.connect_activate({
        let on_add = Rc::clone(&on_add);
        move |_| on_add()
    });

    // Filter buttons.
    filter_all.connect_clicked({
        let handlers = handlers.clone();
        let only_unfinished = only_unfinished.clone();
        move |_| {
            handlers.state.borrow_mut().filter = Filter::All;
            only_unfinished.set_active(false);
            handlers.refresh();
        }
    });
    filter_pending.connect_clicked({
        let handlers = handlers.clone();
        let only_unfinished = only_unfinished.clone();
        move |_| {
            handlers.state.borrow_mut().filter = Filter::Pending;
            only_unfinished.set_active(false);
            handlers.refresh();
        }
    });
    filter_done.connect_clicked({
        let handlers = handlers.clone();
        let only_unfinished = only_unfinished.clone();
        move |_| {
            handlers.state.borrow_mut().filter = Filter::Done;
            only_unfinished.set_active(false);
            handlers.refresh();
        }
    });

    // "Unfinished only" shortcut checkbox.
    only_unfinished.connect_toggled({
        let handlers = handlers.clone();
        let (filter_all, filter_pending, filter_done) = (
            filter_all.clone(),
            filter_pending.clone(),
            filter_done.clone(),
        );
        move |check| {
            let active = check.is_active();
            {
                let mut state = handlers.state.borrow_mut();
                state.only_unfinished = active;
                if !active {
                    state.filter = Filter::All;
                }
            }
            for button in [&filter_all, &filter_pending, &filter_done] {
                button.set_sensitive(!active);
            }
            handlers.refresh();
        }
    });

    rebuild_list(&handlers);
    window.show_all();
}

fn rebuild_list(handlers: &Handlers) {
    let (visible_items, total, done, show_empty) = {
        let state = handlers.state.borrow();
        let visible: Vec<TodoItem> = state
            .items
            .iter()
            .filter(|item| {
                if state.only_unfinished {
                    !matches!(item.status, TaskStatus::Done)
                } else {
                    state.filter.passes(item)
                }
            })
            .cloned()
            .collect();
        let done = state
            .items
            .iter()
            .filter(|item| matches!(item.status, TaskStatus::Done))
            .count();
        let show_empty = visible.is_empty();
        (visible, state.items.len(), done, show_empty)
    };

    // Replace the model contents; the bound ListBox follows automatically.
    handlers.model.remove_all();
    for item in &visible_items {
        let row = make_row(item, handlers);
        handlers.model.append(&row);
    }

    handlers
        .count_label
        .set_text(&format!("{} of {} tasks completed", done, total));
    handlers
        .stack
        .set_visible_child_name(if show_empty { "empty" } else { "list" });
}

/// Build one list row for a task, wiring toggle / rename / delete callbacks.
fn make_row(item: &TodoItem, handlers: &Handlers) -> ListBoxRow {
    let container = gtk::Box::new(Orientation::Horizontal, 8);
    container.style_context().add_class("todo-row");

    let check = gtk::CheckButton::new();
    check.set_active(matches!(item.status, TaskStatus::Done));
    check.set_tooltip_text(Some("Toggle done / pending"));

    let desc_label = Label::new(Some(&item.description));
    desc_label.set_xalign(0.0);
    desc_label.set_ellipsize(pango::EllipsizeMode::End);
    let (desc_class, badge_class) = match item.status {
        TaskStatus::Done => ("desc-done", "badge-done"),
        TaskStatus::TBD => ("desc-tbd", "badge-tbd"),
        TaskStatus::Pending => ("desc-pending", "badge-pending"),
    };
    desc_label.style_context().add_class(desc_class);

    let badge = Label::new(Some(item.status.as_str()));
    badge.style_context().add_class("badge");
    badge.style_context().add_class(badge_class);

    let edit_area = gtk::EventBox::new();
    edit_area.add(&desc_label);

    let delete_button = Button::builder()
        .label("Delete")
        .relief(gtk::ReliefStyle::None)
        .build();
    delete_button.set_tooltip_text(Some("Delete this task"));

    container.pack_start(&check, false, false, 0);
    container.pack_start(&edit_area, true, true, 0);
    container.pack_start(&badge, false, false, 0);
    container.pack_start(&delete_button, false, false, 0);

    let row = ListBoxRow::new();
    row.add(&container);

    // Toggle done/pending.
    check.connect_toggled({
        let handlers = handlers.clone();
        let row_id = item.id;
        move |button| {
            let active = button.is_active();
            {
                let mut state = handlers.state.borrow_mut();
                if let Some(task) = state.items.iter_mut().find(|task| task.id == row_id) {
                    task.status = if active {
                        TaskStatus::Done
                    } else {
                        TaskStatus::Pending
                    };
                }
            }
            handlers.persist();
            handlers.refresh();
        }
    });

    // Double-click the description to rename it.
    edit_area.connect_button_press_event({
        let handlers = handlers.clone();
        let row_id = item.id;
        let description = item.description.clone();
        let parent_window = row.toplevel().and_downcast::<gtk::Window>();
        move |_widget, event| {
            if event.button() == 1 && event.event_type() == gdk::EventType::DoubleButtonPress {
                open_rename_dialog(row_id, &description, &handlers, parent_window.as_ref());
                return glib::Propagation::Stop;
            }
            glib::Propagation::Proceed
        }
    });

    // Delete with confirmation.
    delete_button.connect_clicked({
        let handlers = handlers.clone();
        let row_id = item.id;
        let description = item.description.clone();
        let parent_window = row.toplevel().and_downcast::<gtk::Window>();
        move |_| {
            let dialog = gtk::MessageDialog::new(
                parent_window.as_ref(),
                gtk::DialogFlags::MODAL,
                gtk::MessageType::Question,
                gtk::ButtonsType::OkCancel,
                &format!("Delete \"{}\"?", description),
            );
            dialog.set_secondary_text(Some("This action cannot be undone."));
            dialog.set_title("To-Do List");
            if dialog.run() == ResponseType::Ok {
                handlers
                    .state
                    .borrow_mut()
                    .items
                    .retain(|task| task.id != row_id);
                handlers.persist();
                handlers.refresh();
            }
            unsafe { dialog.destroy() };
        }
    });

    row
}

fn open_rename_dialog(
    row_id: usize,
    current: &str,
    handlers: &Handlers,
    parent: Option<&gtk::Window>,
) {
    let fallback = gtk::Window::new(gtk::WindowType::Toplevel);
    let dialog = gtk::Dialog::builder()
        .title("Rename task")
        .transient_for(parent.unwrap_or(&fallback))
        .modal(true)
        .build();

    let area = dialog.content_area();
    area.set_margin_start(10);
    area.set_margin_end(10);
    area.set_margin_top(10);
    area.pack_start(&Label::new(Some("New description:")), false, false, 0);
    let input = Entry::builder().text(current).build();
    area.pack_start(&input, false, false, 0);
    dialog.add_button("Cancel", ResponseType::Cancel);
    dialog.add_button("Save", ResponseType::Ok);
    dialog.set_default_size(340, -1);
    area.show_all();

    if dialog.run() == ResponseType::Ok {
        let text = input.text().trim().to_string();
        if !text.is_empty() {
            if let Some(task) = handlers
                .state
                .borrow_mut()
                .items
                .iter_mut()
                .find(|task| task.id == row_id)
            {
                task.description = text;
            }
            handlers.persist();
            handlers.refresh();
        }
    }
    unsafe { dialog.destroy() };
}
