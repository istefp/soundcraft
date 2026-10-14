use super::plugin_menu::{MenuSearch, menu_row};
use super::{Route, SoundApp, Track, TrackId, TrackKind, Ui, json};

pub(crate) fn route_name(app: &SoundApp, r: &Route) -> String {
    match r {
        Route::None => "no output".into(),
        Route::Main => app.engine.session().outputs.first().map_or_else(|| "Out 1-2".into(), |o| o.name.clone()),
        Route::Bus(b) => app.engine.session().bus(*b).map_or_else(|| "bus?".into(), |b| b.name.clone()),
        Route::Hardware(h) => h.clone(),
    }
}

pub(crate) fn input_name(app: &SoundApp, track: &Track) -> String {
    if track.kind.has_playlist() && track.mixer.input == Route::None {
        let channels = track.format.channels();
        if channels == 1 { "In 1".to_string() } else { format!("In 1-{channels}") }
    } else {
        route_name(app, &track.mixer.input)
    }
}

/// The input or output menu under `resp`. It closes only on a click outside, so its bus search
/// field can be typed in.
pub(crate) fn route_popup(app: &mut SoundApp, resp: &egui::Response, id: TrackId, input: bool) {
    egui::Popup::menu(resp).close_behavior(egui::PopupCloseBehavior::CloseOnClickOutside).show(|ui| route_menu(app, ui, id, input));
}

/// Input or output menu: no route and buses on both, plus the main output (outputs) or inputs 1-8 (inputs).
fn route_menu(app: &mut SoundApp, ui: &mut Ui, id: TrackId, input: bool) {
    if ui.button(if input { "no input" } else { "no output" }).clicked() {
        assign_route(app, ui, id, input, "none");
        ui.close();
    }
    let main_name = app.engine.session().outputs.first().map_or_else(|| "Out 1-2".to_string(), |o| o.name.clone());
    if !input && ui.button(main_name).clicked() {
        assign_route(app, ui, id, input, "main");
        ui.close();
    }
    if input {
        for i in 1..=8 {
            if ui.button(format!("In {i}")).clicked() {
                assign_route(app, ui, id, input, &format!("In {i}"));
                ui.close();
            }
        }
    }
    ui.separator();
    let busses: Vec<String> = app.engine.session().busses.iter().map(|b| b.name.clone()).collect();
    ui.strong("Bus");
    let search = MenuSearch::show(ui, egui::Id::unique(("route-bus-search", id.0, input)), "Search buses…");
    let visible: Vec<&String> = busses.iter().filter(|b| search.matches(b)).collect();
    if let Some(i) = search.rows(ui, visible.len(), 420.0, |ui, i, highlighted| menu_row(ui, highlighted, visible[i])) {
        assign_route(app, ui, id, input, visible[i]);
        ui.close();
    }
    if ui.button("new bus...").clicked() {
        let mut n = busses.len() + 1;
        while busses.contains(&format!("Bus {n}")) || app.engine.session().track_by_name(&format!("Bus {n}")).is_some() {
            n += 1;
        }
        assign_route(app, ui, id, input, &format!("Bus {n}"));
        ui.close();
    }
    ui.separator();
    let is_mac = ui.ctx().os() == egui::os::OperatingSystem::Mac;
    let name = |m: egui::Modifiers| egui::ModifierNames::NAMES.format(&m, is_mac);
    let (alt, shift, cmd) = (name(egui::Modifiers::ALT), name(egui::Modifiers::SHIFT), name(egui::Modifiers::COMMAND));
    let hint = if input {
        format!("{alt}-click: all tracks · {alt}+{shift}: selected tracks · with {cmd}: cascade the inputs")
    } else {
        format!("{alt}-click: all tracks · {alt}+{shift}: selected tracks")
    };
    ui.small(egui::RichText::new(hint).weak());
}

/// Sets the clicked track's input or output to `value`, or more tracks' by the modifiers held
/// ([`route_targets`]), as one command and so one undo step.
fn assign_route(app: &mut SoundApp, ui: &Ui, id: TrackId, input: bool, value: &str) {
    let key = if input { "input" } else { "output" };
    let cmd = if input { "track.input" } else { "track.output" };
    let mods = ui.input(|i| i.modifiers);
    let (targets, cascade) = {
        let s = app.engine.session();
        let tracks: Vec<(TrackId, bool)> = s.tracks.iter().map(|t| (t.id, takes_route(t.kind, input))).collect();
        let cascadable = input && value.trim().to_ascii_lowercase().starts_with("in ");
        route_targets(&tracks, &s.edit.selected_tracks, id, mods, cascadable)
    };
    let ids: Vec<u64> = targets.iter().map(|t| t.0).collect();
    let _ = app.run(cmd, json!({"tracks": ids, key: value, "cascade": cascade}));
}

/// Whether a track of `kind` has the connection: audio and aux tracks take an input; those and
/// instrument tracks have an output.
fn takes_route(kind: TrackKind, input: bool) -> bool {
    match kind {
        TrackKind::Audio | TrackKind::Aux => true,
        TrackKind::Instrument => !input,
        TrackKind::Master | TrackKind::Midi | TrackKind::Vca | TrackKind::Folder | TrackKind::Video => false,
    }
}

/// Which tracks a routing choice applies to, from the modifiers held on the click, as in Pro
/// Tools: a plain click sets the clicked track; Alt (Option) every track that has the
/// connection; Alt+Shift the selected ones; and with Cmd (Ctrl) held as well a hardware input
/// cascades, the chosen one on the first track and the channels after it on the next ones.
/// `tracks` is the session's order with whether each takes the connection, and the result keeps
/// that order. A widening modifier with nothing to widen to falls back to the clicked track.
pub(crate) fn route_targets(
    tracks: &[(TrackId, bool)],
    selected: &[TrackId],
    clicked: TrackId,
    mods: egui::Modifiers,
    cascadable: bool,
) -> (Vec<TrackId>, bool) {
    if !mods.alt {
        return (vec![clicked], false);
    }
    let pool: Vec<TrackId> = tracks.iter().filter(|(id, takes)| *takes && (!mods.shift || selected.contains(id))).map(|(id, _)| *id).collect();
    let targets = if pool.is_empty() { vec![clicked] } else { pool };
    (targets, cascadable && mods.command)
}

#[cfg(test)]
mod tests {
    use super::input_name;
    use crate::{Services, SoundApp};
    use serde_json::json;

    #[test]
    fn stereo_track_defaults_to_a_stereo_input_pair() {
        let mut app = SoundApp::new(soundcraft_engine::Engine::default(), None, Services::default());
        app.run("track.new", json!({"format": "stereo"})).unwrap();
        let track = &app.engine.session().tracks[0];

        assert_eq!(input_name(&app, track), "In 1-2");
    }
}

#[cfg(test)]
mod route_target_tests {
    use super::route_targets;
    use egui::Modifiers;
    use soundcraft_model::TrackId;

    fn t(n: u64) -> TrackId {
        TrackId(n)
    }

    #[test]
    fn a_plain_click_sets_the_clicked_track_only() {
        let tracks = [(t(1), true), (t(2), true), (t(3), false)];
        assert_eq!(route_targets(&tracks, &[t(1), t(2)], t(2), Modifiers::NONE, true), (vec![t(2)], false));
        assert_eq!(route_targets(&tracks, &[t(1)], t(2), Modifiers::COMMAND, true), (vec![t(2)], false), "Cmd alone doesn't widen or cascade");
        assert_eq!(route_targets(&tracks, &[t(1)], t(2), Modifiers::SHIFT, true), (vec![t(2)], false));
    }

    #[test]
    fn alt_takes_every_track_with_the_connection_and_alt_shift_the_selected_ones() {
        let tracks = [(t(1), true), (t(2), true), (t(3), false), (t(4), true)];
        assert_eq!(route_targets(&tracks, &[], t(2), Modifiers::ALT, false), (vec![t(1), t(2), t(4)], false));
        let alt_shift = Modifiers::ALT | Modifiers::SHIFT;
        assert_eq!(
            route_targets(&tracks, &[t(4), t(1), t(3)], t(1), alt_shift, true),
            (vec![t(1), t(4)], false),
            "session order; track 3 can't take it"
        );
        assert_eq!(route_targets(&tracks, &[], t(2), alt_shift, true), (vec![t(2)], false), "nothing selected: the clicked track");
        assert_eq!(route_targets(&[(t(3), false)], &[], t(3), Modifiers::ALT, true), (vec![t(3)], false));
    }

    #[test]
    fn cmd_with_alt_cascades_a_hardware_input_but_never_a_bus() {
        let tracks = [(t(1), true), (t(2), true)];
        let m = Modifiers::ALT | Modifiers::COMMAND;
        assert_eq!(route_targets(&tracks, &[], t(1), m, true), (vec![t(1), t(2)], true));
        assert_eq!(route_targets(&tracks, &[t(2)], t(1), m | Modifiers::SHIFT, true), (vec![t(2)], true), "on the selected tracks");
        assert_eq!(route_targets(&tracks, &[], t(1), m, false), (vec![t(1), t(2)], false), "a bus or \"none\" has nothing to cascade");
    }
}
