use glib::clone;

use gtk::prelude::*;
use gtk::{
    self, Application, ApplicationWindow, Box, Button, FileDialog, Label, Orientation, glib,
};

use std::fs;

use gio::Cancellable;

const APP_ID: &str = "spiceboy.MediaOrganizer";

fn main() -> glib::ExitCode {
    // Create the Application
    let app = Application::builder().application_id(APP_ID).build();

    // Connect to "activate" signal of "app"
    app.connect_activate(build_ui);

    // Run the application
    app.run()
}

fn build_ui(app: &Application) {
    // Build UI components
    let button = build_ingestion_button();
    let gtk_box = build_entry_box(&button);
    let window = build_main_window(app, &gtk_box);

    // Set up event handlers
    if let Err(e) = setup_ingestion_file_upload_click(&button, &window) {
        eprintln!("Error setting up button handler: {}", e);
    }

    // Present window
    window.present();
}

fn build_ingestion_button() -> Button {
    Button::builder()
        .label("Choose Ingestion Folder")
        .margin_top(12)
        .margin_bottom(12)
        .margin_start(12)
        .margin_end(12)
        .valign(gtk::Align::Center)
        .vexpand(true)
        .halign(gtk::Align::Center)
        .hexpand(true)
        .build()
}

fn build_entry_box(button: &Button) -> Box {
    let gtk_box = Box::builder()
        .orientation(Orientation::Vertical)
        .hexpand(true)
        .vexpand(true)
        .build();
    gtk_box.append(button);
    gtk_box
}

fn build_main_window(app: &Application, gtk_box: &Box) -> ApplicationWindow {
    ApplicationWindow::builder()
        .application(app)
        .title("Media Organizer")
        .child(gtk_box)
        .maximized(true)
        .deletable(true)
        .build()
}

fn setup_ingestion_file_upload_click(
    button: &Button,
    window: &ApplicationWindow,
) -> Result<(), String> {
    button.connect_clicked(clone!(
        #[weak]
        window,
        move |_| {
            let file_dialog = FileDialog::builder()
                .title("Select Ingestion Folder")
                .modal(true)
                .build();

            file_dialog.select_folder(
                Some(&window),
                Option::<&Cancellable>::None,
                clone!(
                    #[weak]
                    window,
                    move |result| {
                        if let Ok(file) = result {
                            if let Some(path) = file.path() {
                                handle_folder_selection(&window, &path);
                            }
                        }
                    }
                ),
            );
        }
    ));

    Ok(())
}

fn handle_folder_selection(window: &ApplicationWindow, path: &std::path::Path) {
    println!("Selected folder: {:?}", path);

    // Count files in the directory
    let file_count = match count_files_in_directory(path) {
        Ok(count) => count,
        Err(e) => {
            eprintln!("Error counting files: {}", e);
            0
        }
    };

    // Build and display the files UI
    let files_box = build_files_display_box(file_count);
    window.set_child(Some(&files_box));
}

fn count_files_in_directory(path: &std::path::Path) -> Result<usize, String> {
    let mut file_count = 0;

    match fs::read_dir(path) {
        Ok(entries) => {
            println!("Files in the selected folder: ");
            for entry in entries {
                if let Ok(entry) = entry {
                    if let Ok(metadata) = entry.metadata() {
                        if metadata.is_file() {
                            file_count += 1;
                            println!(" - {:?}", entry.file_name());
                        }
                    }
                }
            }
            Ok(file_count)
        }
        Err(e) => Err(format!("Error reading directory: {}", e)),
    }
}

fn build_files_display_box(file_count: usize) -> Box {
    let files_box = Box::builder()
        .orientation(Orientation::Vertical)
        .spacing(10)
        .margin_top(12)
        .margin_bottom(12)
        .margin_start(12)
        .margin_end(12)
        .hexpand(true)
        .vexpand(true)
        .build();

    // Add label
    let files_label = Label::builder()
        .label(&format!("{} file(s) found", file_count))
        .build();
    files_box.append(&files_label);

    files_box
}
