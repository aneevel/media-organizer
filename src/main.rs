use glib::clone;

use gtk::{
    self, Application, ApplicationWindow, Box, Button, FileDialog, Label, ListBox, Orientation,
    Paned, PolicyType, glib,
};
use gtk::{ScrolledWindow, prelude::*};

use std::fs;
use std::path::{Path, PathBuf};

use gio::Cancellable;

const APP_ID: &str = "spiceboy.MediaOrganizer";

// Define a vector of accepted image file extensions
// TODO: This should be configurable, let's let the user decide
const ACCEPTED_IMAGE_EXTENSIONS: &[&str] = &["jpg", "jpeg", "png", "gif", "bmp", "tiff", "webp", "svg"];

// Structure to hold file information
struct FileInfo {
    path: PathBuf,
    name: String,
    extension: String,
    size: u64,
}

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

    // Build list of files to process
    match build_file_list_to_process(path) {
        Ok(file_list) => {
            // Build and display the files UI
            let files_box = build_files_display_box(&file_list);
            window.set_child(Some(&files_box));
        },
        Err(e) => {
            eprintln!("Error processing directory: {}", e);
            let error_label = Label::builder().label(&format!("Error processing directory: {}", e)).build();
            window.set_child(Some(&error_label));
        }
    }
}

fn build_file_list_to_process(path: &Path) -> Result<Vec<FileInfo>, String> {
    let mut file_list = Vec::new();

    match fs::read_dir(path) {
        Ok(entries) => {
            for entry in entries {
                if let Ok(entry) = entry {
                    if let Ok(metadata) = entry.metadata() {
                        if metadata.is_file() {
                            // Check if the file has an accepted image extension
                            if let Some(extension) = entry.path().extension().and_then(|ext| ext.to_str()) {
                                if ACCEPTED_IMAGE_EXTENSIONS.iter().any(|&accepted| accepted.eq_ignore_ascii_case(extension)) {
                                    let name = entry.file_name().to_string_lossy().to_string();
                                    
                                    // Add file to our list
                                    file_list.push(FileInfo {
                                        path: entry.path(),
                                        name,
                                        extension: extension.to_string(),
                                        size: metadata.len(),
                                    });
                                }
                            }
                        }
                    }
                }
            }
            Ok(file_list)
        }
        Err(e) => Err(format!("Error reading directory: {}", e)),
    }
}

fn build_files_display_box(file_list: &[FileInfo]) -> Box {
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
    let paned_box = build_paned_result_structure(file_list);
    files_box.append(&paned_box);

    files_box
}

fn build_paned_result_structure(file_list: &[FileInfo]) -> Paned {
    // Create Paned widget
    let paned_box = Paned::builder()
        .orientation(Orientation::Vertical)
        .hexpand(true)
        .vexpand(true)
        .build();

    // Add overview label
    let files_overview_label = Label::builder()
        .label(&format!("{} file(s) found", file_list.len()))
        .build();
    paned_box.set_start_child(Some(&files_overview_label));

    // Split bottom pane into two horizontal panes - display pane and processing pane
    let file_overview_pane = build_file_processing_display(file_list);
    paned_box.set_end_child(Some(&file_overview_pane));

    paned_box
}

fn build_file_processing_display(file_list: &[FileInfo]) -> Paned {
    // Split bottom pane into two horizontal panes - display pane and processing pane
    let file_overview_pane = Paned::builder()
        .orientation(Orientation::Horizontal)
        .hexpand(true)
        .vexpand(true)
        .build();

    // Add files display - attaching every file in the path
    let files_window = build_file_processing_list(file_list);
    file_overview_pane.set_start_child(Some(&files_window));

    // Add file processing box
    let file_processing_box = build_file_processor_display();
    file_overview_pane.set_end_child(Some(&file_processing_box));

    file_overview_pane
}

fn build_file_processing_list(file_list: &[FileInfo]) -> ScrolledWindow {
    let files_list_box = ListBox::new();

    let files_window = ScrolledWindow::builder()
        .hscrollbar_policy(PolicyType::Never)
        .hexpand(true)
        .vexpand(true)
        .child(&files_list_box)
        .build();

    // Add all files from our file_list
    for file in file_list {
        let new_file_label = Label::builder()
            .label(&format!(
                "{} {} {} bytes",
                file.name,
                file.extension,
                file.size
            ))
            .build();

        files_list_box.append(&new_file_label);
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
