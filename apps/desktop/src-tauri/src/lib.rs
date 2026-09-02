mod app;
mod commands;
mod floating_card;
mod inbox;
mod temporary;

pub fn run() {
    app::build()
        .run(tauri::generate_context!())
        .expect("error while running Download Inbox");
}
