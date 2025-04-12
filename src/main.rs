use glib::clone;

use gtk::{
    self, Application, ApplicationWindow, Box, Button, FileDialog, Label, ListBox, Orientation,
    Paned, PolicyType, glib,
};
use gtk::{ScrolledWindow, prelude::*};

use std::fs;
use std::path::Path;

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
    let files_box = build_files_display_box(file_count, path);
    window.set_child(Some(&files_box));
}

fn count_files_in_directory(path: &std::path::Path) -> Result<usize, String> {
    let mut file_count = 0;

    match fs::read_dir(path) {
        Ok(entries) => {
            for entry in entries {
                if let Ok(entry) = entry {
                    if let Ok(metadata) = entry.metadata() {
                        if metadata.is_file() {
                            file_count += 1;
                        }
                    }
                }
            }
            Ok(file_count)
        }
        Err(e) => Err(format!("Error reading directory: {}", e)),
    }
}

fn build_files_display_box(file_count: usize, path: &std::path::Path) -> Box {
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

    // Create the paned structure and add it to the main box
    let paned_box = build_paned_result_structure(file_count, path);
    files_box.append(&paned_box);

    files_box
}

fn build_paned_result_structure(file_count: usize, path: &std::path::Path) -> Paned {
    // Create Paned widget
    let paned_box = Paned::builder()
        .orientation(Orientation::Vertical)
        .hexpand(true)
        .vexpand(true)
        .build();

    // Add overview label
    let files_overview_label = Label::builder()
        .label(&format!("{} file(s) found", file_count))
        .build();
    paned_box.set_start_child(Some(&files_overview_label));

    // Split bottom pane into two horizontal panes - display pane and processing pane
    let file_overview_pane = build_file_processing_display(path);
    paned_box.set_end_child(Some(&file_overview_pane));

    paned_box
}

fn build_file_processing_display(path: &std::path::Path) -> Paned {
    // Split bottom pane into two horizontal panes - display pane and processing pane
    let file_overview_pane = Paned::builder()
        .orientation(Orientation::Horizontal)
        .hexpand(true)
        .vexpand(true)
        .build();

    // Add files display - attaching every file in the path
    let files_window = build_file_processing_list(path);
    file_overview_pane.set_start_child(Some(&files_window));

    // Add file processing box
    let file_processing_box = build_file_processor_display();
    file_overview_pane.set_end_child(Some(&file_processing_box));

    file_overview_pane
}

fn build_file_processing_list(path: &std::path::Path) -> ScrolledWindow {
    let files_list_box = ListBox::new();

    let files_window = ScrolledWindow::builder()
        .hscrollbar_policy(PolicyType::Never)
        .hexpand(true)
        .vexpand(true)
        .child(&files_list_box)
        .build();

    // Add all files
    match fs::read_dir(path) {
        Ok(entries) => {
            for entry in entries {
                if let Ok(entry) = entry {
                    if let Ok(metadata) = entry.metadata() {
                        if metadata.is_file() {
                            let new_file_label = Label::builder()
                                .label(&format!(
                                    "{:?} {:?} {:?} bytes",
                                    entry.file_name(),
                                    Path::new(&entry.file_name())
                                        .extension()
                                        .and_then(|ext| ext.to_str())
                                        .unwrap_or("Unknown Extension"),
                                    metadata.len()
                                ))
                                .build();

                            files_list_box.append(&new_file_label);
                        }
                    }
                }
            }
        }
        Err(e) => println!("Error reading directory: {}", e),
    }

    files_window
}

fn build_file_processor_display() -> Box {
    let file_processing_file_name_label =
        Label::builder().label(&format!("No file selected")).build();
    let file_processing_box = Box::builder()
        .orientation(Orientation::Horizontal)
        .hexpand(true)
        .vexpand(true)
        .build();
    file_processing_box.append(&file_processing_file_name_label);
    
    file_processing_box
}
