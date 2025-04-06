use gtk::prelude::*;
use gtk::{self, Application, ApplicationWindow, Button, Orientation, glib};

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
        .build();

    // Add buttons to `gtk_box`
    let gtk_box = gtk::Box::builder()
        .orientation(Orientation::Vertical)
        .build();
    gtk_box.append(&button_select_ingestion);

    // Create a window and set the title
    let window = ApplicationWindow::builder()
        .application(app)
        .title("Media Organizer")
        .child(&gtk_box)
        .fullscreened(true)
        .build();

    // Present window
    window.present();
}
