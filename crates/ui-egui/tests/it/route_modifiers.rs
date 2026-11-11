//! The I/O menus read the modifiers held on the click: Alt sets every track, and with Cmd (Ctrl)
//! as well a hardware input cascades over them (#570).

use egui::{Modifiers, vec2};
use egui_kittest::Harness;
use egui_kittest::kittest::Queryable;
use serde_json::json;
use soundcraft_model::Route;
use soundcraft_ui_egui::{MainWindow, Services, SoundApp};

fn harness() -> Harness<'static, SoundApp> {
    let mut engine = soundcraft_engine::Engine::default();
    for (name, format) in [("Vox", "mono"), ("Gtr", "stereo"), ("Bass", "mono")] {
        engine.execute("track.new", &json!({"name": name, "format": format})).unwrap();
    }
    let mut app = SoundApp::new(engine, None, Services::default());
    app.ui.window = MainWindow::Mix;
    let mut h = Harness::builder().with_size(vec2(1400.0, 1000.0)).build_ui_state(
        |ui, app: &mut SoundApp| {
            app.logic(&ui.ctx().clone());
            app.ui(ui);
        },
        app,
    );
    h.run_steps(4);
    h
}

fn pick(h: &mut Harness<SoundApp>, menu: &str, choice: &str, modifiers: Modifiers) {
    h.get_by_label(menu).click();
    h.run_steps(3);
    h.get_by_label(choice).click_modifiers(modifiers);
    h.run_steps(3);
}

fn inputs(h: &Harness<SoundApp>) -> Vec<Route> {
    h.state().engine.session().tracks.iter().map(|t| t.mixer.input.clone()).collect()
}

#[test]
fn alt_click_routes_every_track_and_cmd_alt_cascades_the_inputs() {
    let mut h = harness();
    pick(&mut h, "Input for Vox", "In 2", Modifiers::NONE);
    assert_eq!(inputs(&h)[0], Route::Hardware("In 2".into()));
    assert_ne!(inputs(&h)[1], Route::Hardware("In 2".into()), "a plain click sets the clicked track only");

    pick(&mut h, "Input for Gtr", "In 3", Modifiers::ALT | Modifiers::COMMAND);
    assert_eq!(inputs(&h), ["In 3", "In 4-5", "In 6"].map(|n| Route::Hardware(n.into())), "a stereo track takes two channels");

    pick(&mut h, "Output for Bass", "no output", Modifiers::ALT);
    assert!(h.state().engine.session().tracks.iter().all(|t| t.mixer.output == Route::None), "Alt sets every track");
}
