use glib::clone;

use gtk::prelude::*;
use gtk::{self, Application, ApplicationWindow, Button, FileDialog, Orientation, glib};

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
    // Create the ingestion selection button
    let button_select_ingestion = Button::builder()
        .label("Choose Ingestion Folder")
        .margin_top(12)
        .margin_bottom(12)
        .margin_start(12)
        .margin_end(12)
        .valign(gtk::Align::Center)
        .vexpand(true)
        .halign(gtk::Align::Center)
        .hexpand(true)
        .build();

    // Add buttons to `gtk_box`
    let gtk_box = gtk::Box::builder()
        .orientation(Orientation::Vertical)
        .hexpand(true)
        .vexpand(true)
        .build();
    gtk_box.append(&button_select_ingestion);

    // Create a window and set the title
    let window = ApplicationWindow::builder()
        .application(app)
        .title("Media Organizer")
        .child(&gtk_box)
        .maximized(true)
        .deletable(true)
        .build();

    // Call the file select dialog when clicking button
    button_select_ingestion.connect_clicked(clone!(
        #[weak]
        window,
        move |_| {
            let file_dialog = FileDialog::builder()
                .title("Select Ingestion Folder")
                .modal(true)
                .build();

            file_dialog.select_folder(Some(&window), Option::<&Cancellable>::None, |result| {
                if let Ok(file) = result {
                    if let Some(path) = file.path() {
                        println!("Selected folder: {:?}", path);

                        match fs::read_dir(&path) {
                            Ok(entries) => {
                                println!("Files in the selected folder: ");
                                for entry in entries {
                                    if let Ok(entry) = entry {
                                        if let Ok(metadata) = entry.metadata() {
                                            if metadata.is_file() {
                                                println!(" - {:?}", entry.file_name());
                                            }
                                        }
                                    }
                                }
                            }
                            Err(e) => println!("Error reading directory: {}", e),
                        }
                    }
                }
            });
        }
    ));

    // Present window
    window.present();
}
