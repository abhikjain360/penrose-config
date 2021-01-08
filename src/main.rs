use penrose::{
    bindings::MouseEvent,
    contrib::extensions::Scratchpad,
    draw::{
        bar::{widgets::{Workspaces, CurrentLayout, ActiveWindowName, RootWindowName}, StatusBar},
        Position, TextStyle, XCBDraw,
    },
    gen_keybindings, gen_mousebindings,
    helpers::index_selectors,
    layout::{bottom_stack, side_stack, Layout, LayoutConf},
    run_external, run_internal, Backward, Config, Forward, Less, More, Result, WindowManager,
    XcbConnection,
};

mod bar_text;
mod colors;

use colors::*;

const FONT: &str = "Fira Code";

fn main() -> Result<()> {
    let mut config = Config::default();

    config.floating_classes = &["dmenu", "polybar"];
    config.gap_px = 0;
    config.border_px = 7;
    config.focused_border = GREEN;
    config.bar_height = 28;
    config.main_ratio_step = 0.05;
    config.layouts = vec![
        Layout::new("[side]", LayoutConf::default(), side_stack, 1, 0.5),
        Layout::new("[botm]", LayoutConf::default(), bottom_stack, 1, 0.5),
    ];

    let sp = Box::new(Scratchpad::new("st", 0.8, 0.8));
    let sp_toggle = sp.toggle();
    let style = &TextStyle {
        font: FONT.to_string(),
        point_size: 14,
        fg: WHITE.into(),
        bg: Some(BLACK.into()),
        padding: (2.0, 2.0),
    };
    let bar = Box::new(
        StatusBar::try_new(
            Box::new(XCBDraw::new()?),
            Position::Top,
            28,
            style.bg.unwrap_or_else(|| 0x000000.into()),
            &[&style.font],
            vec![
                Box::new(Workspaces::new(
                    &config.workspaces,
                    style,
                    BLUE, // highlight
                    GREY, // empty_ws
                )),
                Box::new(CurrentLayout::new(style)),
                Box::new(ActiveWindowName::new(
                    &TextStyle {
                        bg: Some(BLUE.into()),
                        padding: (6.0, 4.0),
                        ..style.clone()
                    },
                    35,
                    true,
                    false,
                )),
                Box::new(RootWindowName::new(
                &TextStyle {
                    padding: (4.0, 2.0),
                    ..style.clone()
                },
                false,
                true,
            )),
            ],
        )
        .unwrap(),
    );
    // let bar_toggle = bar.toggle();
    config.hooks = vec![sp, bar];

    let key_bindings = gen_keybindings! {
        // stack management
        "M-d" => run_external!("dmenu_run");
        "M-Return" => run_external!("st");
        "M-S-Return" => sp_toggle;
        "M-k" => run_internal!(cycle_client, Backward);
        "M-j" => run_internal!(cycle_client, Forward);
        "M-o" => run_internal!(update_max_main, More);
        "M-S-o" => run_internal!(update_max_main, Less);
        "M-l" => run_internal!(update_main_ratio, More);
        "M-h" => run_internal!(update_main_ratio, Less);
        "M-S-j" => run_internal!(drag_client, Forward);
        "M-S-k" => run_internal!(drag_client, Backward);
        "M-q" => run_internal!(kill_client);
        "M-Tab" => run_internal!(toggle_workspace);
        "M-bracketright" => run_internal!(cycle_screen, Forward);
        "M-bracketleft" => run_internal!(cycle_screen, Backward);
        "M-S-l" => run_internal!(drag_workspace, Forward);
        "M-S-j" => run_internal!(drag_workspace, Backward);
        "M-S-q" => run_internal!(exit);

        // layouts
        "M-t" => run_internal!(cycle_layout, Forward);
        "M-b" => Box::new(|wm: &mut WindowManager<'_>| {

        });

        // workspaces
        refmap [ config.ws_range() ] in {
            "M-{}" => focus_workspace [ index_selectors(config.workspaces.len()) ];
            "M-S-{}" => client_to_workspace [ index_selectors(config.workspaces.len()) ];
        };

        // applications
        "M-w" => run_external!("brave");
        "M-S-w" => run_external!("bookmarks2");
        "M-S-s" => run_external!("locker");
        "M-m" => run_external!("st -e ncmpcpp");
        "M-v" => run_external!("st -e alsamixer");
        "M-S-v" => run_external!("pavucontrol");
        "M-e" => run_external!("st -e ranger");
        "M-p" => run_external!("flameshot gui");
        "M-s" => run_external!("st -e stub");
    };

    let mouse_bindings = gen_mousebindings! {
        Press Right + [Meta] => |wm: &mut WindowManager, _: &MouseEvent| wm.cycle_workspace(Forward),
        Press Left + [Meta] => |wm: &mut WindowManager, _: &MouseEvent| wm.cycle_workspace(Backward)
    };

    let conn = XcbConnection::new().unwrap();
    let mut wm = WindowManager::init(config, &conn);
    wm.grab_keys_and_run(key_bindings, mouse_bindings);

    Ok(())
}
