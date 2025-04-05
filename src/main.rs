use gtk::prelude::*;
use gtk::{Application, ApplicationWindow, Button, glib};

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
    // Create a button with label and margins
    let button = Button::builder()
        .label("Organize!")
        .margin_top(12)
        .margin_bottom(12)
        .margin_start(12)
        .margin_end(12)
        .build();

    // Connect to clicked signal of button
    button.connect_clicked(|button| {
        // Set the label to "Organized" after the button has been clicked on
        button.set_label("Organized");
    });

    // Create a window and set the title
    let window = ApplicationWindow::builder()
        .application(app)
        .title("Media Organizer")
        .child(&button)
        .build();

    // Present window
    window.present();
}
