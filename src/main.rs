use glib::clone;

use gtk::{
    self, Application, ApplicationWindow, Box, Button, Entry, FileDialog, Label, ListBox,
    Orientation, Paned, Picture, PolicyType, glib,
};
use gtk::{ScrolledWindow, prelude::*};

use std::cell::RefCell;
use std::fs;
use std::path::PathBuf;
use std::rc::Rc;

use gio::Cancellable;

const APP_ID: &str = "spiceboy.MediaOrganizer";

// Define a vector of accepted image file extensions
// TODO: This should be configurable, let's let the user decide
const ACCEPTED_IMAGE_EXTENSIONS: &[&str] =
    &["jpg", "jpeg", "png", "gif", "bmp", "tiff", "webp", "svg"];

// Structure to hold file information
#[derive(Clone)]
struct FileInfo {
    path: PathBuf,
    name: String,
    extension: String,
    size: u64,
}

// Application state structure
#[derive(Clone)]
struct AppState {
    input_directory: Option<PathBuf>,
    output_directory: Option<PathBuf>,
    selected_files: Vec<FileInfo>,
}

impl AppState {
    fn new() -> Self {
        Self {
            input_directory: None,
            output_directory: None,
            selected_files: Vec::new(),
        }
    }

    fn set_input_directory(&mut self, path: PathBuf) {
        self.input_directory = Some(path);
    }

    fn get_input_directory(&self) -> Option<&PathBuf> {
        self.input_directory.as_ref()
    }

    fn set_output_directory(&mut self, path: PathBuf) {
        self.output_directory = Some(path);
    }

    fn get_output_directory(&self) -> Option<&PathBuf> {
        self.output_directory.as_ref()
    }

    fn add_selected_file(&mut self, file: FileInfo) {
        self.selected_files.push(file);
    }

    fn clear_selected_files(&mut self) {
        self.selected_files.clear();
    }

    fn get_selected_files(&self) -> &[FileInfo] {
        &self.selected_files
    }
}

fn main() -> glib::ExitCode {
    // Create the Application
    let app = Application::builder().application_id(APP_ID).build();

    // Create main application state
    let app_state = Rc::new(RefCell::new(AppState::new()));

    // Connect to "activate" signal of "app"
    let app_state_clone = Rc::clone(&app_state);
    app.connect_activate(move |app| {
        build_ui(app, Rc::clone(&app_state_clone));
    });

    // Run the application
    app.run()
}

fn build_ui(app: &Application, app_state: Rc<RefCell<AppState>>) {
    // Build UI components
    let button = build_ingestion_button();
    let gtk_box = build_entry_box(&button);
    let window = build_main_window(app, &gtk_box);

    // Set up event handlers
    if let Err(e) = setup_ingestion_file_upload_click(&button, &window, Rc::clone(&app_state)) {
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
    app_state: Rc<RefCell<AppState>>,
) -> Result<(), String> {
    button.connect_clicked(clone!(
        #[weak]
        window,
        #[strong]
        app_state,
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
                    #[strong]
                    app_state,
                    move |result| {
                        if let Ok(file) = result {
                            if let Some(path) = file.path() {
                                app_state.borrow_mut().set_input_directory(path.clone());

                                handle_folder_selection(&window, Rc::clone(&app_state));
                            }
                        }
                    }
                ),
            );
        }
    ));

    Ok(())
}

fn handle_folder_selection(window: &ApplicationWindow, app_state: Rc<RefCell<AppState>>) {
    println!(
        "Selected folder: {:?}",
        app_state.borrow_mut().get_input_directory()
    );

    // Build list of files to process
    match build_file_list_to_process(Rc::clone(&app_state)) {
        Ok(()) => {
            // Build and display the files UI
            let files_box = build_files_display_box(&window, Rc::clone(&app_state));
            window.set_child(Some(&files_box));
        }
        Err(e) => {
            eprintln!("Error processing directory: {}", e);
            let error_label = Label::builder()
                .label(&format!("Error processing directory: {}", e))
                .build();
            window.set_child(Some(&error_label));
        }
    }
}

fn build_file_list_to_process(app_state: Rc<RefCell<AppState>>) -> Result<(), String> {
    match fs::read_dir(
        app_state
            .borrow_mut()
            .get_input_directory()
            .unwrap()
            .as_path(),
    ) {
        Ok(entries) => {
            for entry in entries {
                if let Ok(entry) = entry {
                    if let Ok(metadata) = entry.metadata() {
                        if metadata.is_file() {
                            // Check if the file has an accepted image extension
                            if let Some(extension) =
                                entry.path().extension().and_then(|ext| ext.to_str())
                            {
                                if ACCEPTED_IMAGE_EXTENSIONS
                                    .iter()
                                    .any(|&accepted| accepted.eq_ignore_ascii_case(extension))
                                {
                                    let name = entry.file_name().to_string_lossy().to_string();

                                    app_state.borrow_mut().add_selected_file(FileInfo {
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
            Ok(())
        }
        Err(e) => Err(format!("Error reading directory: {}", e)),
    }
}

fn build_files_display_box(window: &ApplicationWindow, app_state: Rc<RefCell<AppState>>) -> Box {
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
    let paned_box = build_paned_result_structure(&window, Rc::clone(&app_state));
    files_box.append(&paned_box);

    files_box
}

fn build_paned_result_structure(
    window: &ApplicationWindow,
    app_state: Rc<RefCell<AppState>>,
) -> Paned {
    // Create Paned widget
    let paned_box = Paned::builder()
        .orientation(Orientation::Vertical)
        .hexpand(true)
        .vexpand(true)
        .build();

    // Add overview box
    let file_overview_box = Box::builder()
        .orientation(Orientation::Vertical)
        .hexpand(true)
        .vexpand(true)
        .build();

    let files_overview_label = Label::builder()
        .label(&format!(
            "{} file(s) found",
            app_state.borrow().get_selected_files().len()
        ))
        .build();
    file_overview_box.append(&files_overview_label);

    // We need an output handler for the output directory
    let output_selection_button = build_output_selection_button();

    if let Err(e) = setup_output_selection_button_click(
        &output_selection_button,
        &window,
        Rc::clone(&app_state),
    ) {
        eprintln!("Error setting up output selection handler: {}", e);
    }

    file_overview_box.append(&output_selection_button);

    paned_box.set_start_child(Some(&file_overview_box));

    // Split bottom pane into two horizontal panes - display pane and processing pane
    let file_overview_pane = build_file_processing_display(Rc::clone(&app_state));
    paned_box.set_end_child(Some(&file_overview_pane));

    paned_box
}

fn build_file_processing_display(app_state: Rc<RefCell<AppState>>) -> Paned {
    // Split bottom pane into two horizontal panes - display pane and processing pane
    let file_overview_pane = Paned::builder()
        .orientation(Orientation::Horizontal)
        .hexpand(true)
        .vexpand(true)
        .build();

    // Add file processing box
    let file_processing_box = build_file_processor_display();
    let file_processing_box_clone = file_processing_box.clone();

    // Add files display - attaching every file in the path
    // Pass the processing box clone so that we can attach it via callback
    let files_window = build_file_processing_list(
        move |file| {
            // Update file processor when a file is selected
            update_file_processor(&file_processing_box_clone, file);
        },
        Rc::clone(&app_state),
    );

    update_file_processor(
        &file_processing_box,
        &app_state.borrow_mut().get_selected_files()[0],
    );

    file_overview_pane.set_start_child(Some(&files_window));
    file_overview_pane.set_end_child(Some(&file_processing_box));

    file_overview_pane
}

fn build_file_processing_list(
    on_file_selected: impl Fn(&FileInfo) + 'static,
    app_state: Rc<RefCell<AppState>>,
) -> ScrolledWindow {
    let files_list_box = ListBox::new();

    // Clone the file_list for use in the closure
    let file_list_clone = app_state.borrow().get_selected_files().to_vec();

    // Setup row handler
    files_list_box.connect_row_activated(move |_, row| {
        let index = row.index() as usize;
        if index < file_list_clone.len() {
            on_file_selected(&file_list_clone[index]);
        }
    });

    let files_window = ScrolledWindow::builder()
        .hscrollbar_policy(PolicyType::Never)
        .hexpand(true)
        .vexpand(true)
        .child(&files_list_box)
        .build();

    // Add all files from our file_list
    for file in app_state.borrow().get_selected_files() {
        let new_file_label = Label::builder()
            .label(&format!(
                "{} {} {} bytes",
                file.name, file.extension, file.size
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

fn update_file_processor(box_widget: &Box, file: &FileInfo) {
    // Clear out the previous file information
    while let Some(child) = box_widget.first_child() {
        box_widget.remove(&child);
    }

    // Create summary box for image and info
    let summary_box = Box::builder()
        .orientation(Orientation::Vertical)
        .spacing(10)
        .build();

    // add the image thumbnail
    let thumbnail = create_image_thumbnail(&file.path, 200, 200);

    summary_box.append(&thumbnail);

    // Add in the new file info
    let file_info_label = Label::builder()
        .label(&format!(
            "Selected file: {}\nSize: {} bytes\nType: {}",
            file.name, file.size, file.extension
        ))
        .halign(gtk::Align::Start)
        .build();

    // Add the output section
    let output_box: Box = Box::builder()
        .orientation(Orientation::Horizontal)
        .spacing(10)
        .build();

    let output_label: Label = Label::builder()
        .label(&format!("Output File Name"))
        .halign(gtk::Align::Start)
        .build();
    output_box.append(&output_label);

    let output_name_edit = Entry::builder().build();
    output_box.append(&output_name_edit);

    summary_box.append(&file_info_label);
    summary_box.append(&output_box);
    box_widget.append(&summary_box);
}

fn create_image_thumbnail(file_path: &std::path::Path, width: i32, height: i32) -> Picture {
    let picture: Picture = Picture::new();

    // Set the size
    picture.set_content_fit(gtk::ContentFit::Contain);
    picture.set_size_request(width, height);

    // Load image from file
    picture.set_filename(Some(file_path.to_str().unwrap_or("")));

    picture
}

fn build_output_selection_button() -> Button {
    Button::builder()
        .label("Set Output Folder")
        .margin_top(12)
        .margin_bottom(12)
        .margin_start(12)
        .valign(gtk::Align::Center)
        .vexpand(true)
        .halign(gtk::Align::Center)
        .hexpand(true)
        .build()
}

fn setup_output_selection_button_click(
    button: &Button,
    window: &ApplicationWindow,
    app_state: Rc<RefCell<AppState>>,
) -> Result<(), String> {
    button.connect_clicked(clone!(
        #[weak]
        window,
        #[strong]
        app_state,
        move |_| {
            let file_dialog: FileDialog = FileDialog::builder()
                .title("Select Output Folder")
                .modal(true)
                .build();

            file_dialog.select_folder(
                Some(&window),
                Option::<&Cancellable>::None,
                clone!(
                    #[strong]
                    app_state,
                    move |result| {
                        if let Ok(file) = result {
                            if let Ok(file_info) = file.query_info(
                                "standard::type",
                                gio::FileQueryInfoFlags::NONE,
                                None::<&gio::Cancellable>,
                            ) {
                                if file_info.file_type() == gio::FileType::Directory {
                                    if let Some(path) = file.path() {
                                        app_state.borrow_mut().set_output_directory(path.clone());
                                        println!("Output directory set to: {:?}", path);
                                    }
                                } else {
                                    eprintln!("Selected improper file; not a directory!");
                                }
                            }
                        }
                    }
                ),
            );
        }
    ));

    Ok(())
}
