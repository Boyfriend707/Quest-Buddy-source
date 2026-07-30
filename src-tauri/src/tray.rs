use tauri::{
    menu::{Menu, MenuItem},
    tray::TrayIconBuilder,
    App, Error, Manager,
};

pub fn create_tray(app: &App) -> Result<(), Error> {
    let show = MenuItem::with_id(app, "show", "Show", true, None::<&str>)?;
    let quit = MenuItem::with_id(app, "quit", "Quit", true, None::<&str>)?;
    let menu = Menu::with_items(app, &[&show, &quit])?;

    let icon = tauri::include_image!("icons/icon.png");

    TrayIconBuilder::new()
        .icon(icon)
        .tooltip("QuestBuddy")
        .menu(&menu)
        .on_menu_event(|app, event| match event.id().as_ref() {
            "show" => {
                if let Some(window) = app.get_webview_window("main") {
                    let _ = window.show();
                    let _ = window.set_focus();
                }
            }
            "quit" => {
                app.exit(0);
            }
            _ => {}
        })
        .build(app)?;

    Ok(())
}
