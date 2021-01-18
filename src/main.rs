#![allow(dead_code, unused_imports)]

#[macro_use]
extern crate penrose;

use penrose::{
    contrib::extensions::Scratchpad,
    core::{
        bindings::{KeyEventHandler, MouseEvent},
        config::Config,
        data_types::PropVal,
        helpers::index_selectors,
        layout::{bottom_stack, side_stack, Layout, LayoutConf},
        manager::WindowManager,
        Client, Selector,
    },
    draw::{dwm_bar, TextStyle},
    logging_error_handler,
    xcb::{new_xcb_backed_window_manager, XcbDraw, XcbHooks},
    Backward, Forward, Less, More, PenroseError,
};

use std::cell::RefCell;
use std::rc::Rc;

// applications
const TERMINAL: &str = "st";
const LAUNCHER: &str = "dmenu_run";

// colors
const WHITE: u32 = 0xebdbb2ff;
const GREY: u32 = 0x3c3836ff;
const BLUE: u32 = 0x458588ff;
const GREEN: u32 = 0x0055ff33;
const BLACK: u32 = 0x282828ff;

// bar aesthettics
const FONT: &str = "Fira Code";
const HEIGHT: usize = 28;

fn main() {
    // generate default configs
    let config = Config::default()
        .builder()
        .floating_classes(vec!["dmenu", "polybar"])
        .gap_px(0)
        .border_px(5)
        .focused_border(GREEN)
        .bar_height(28)
        .main_ratio_step(0.05)
        .layouts(vec![
            Layout::new("[side]", LayoutConf::default(), side_stack, 1, 0.5),
            Layout::new("[botm]", LayoutConf::default(), bottom_stack, 1, 0.5),
            Layout::floating("[----]"),
        ])
        .build()
        .expect("unable to build config");

    let sp = Scratchpad::new(TERMINAL, 0.8, 0.8);
    let bar = dwm_bar(
        XcbDraw::new().expect("unable to start xcb draw"),
        HEIGHT,
        &TextStyle {
            font: FONT.to_string(),
            point_size: 11,
            fg: WHITE.into(),
            bg: Some(BLACK.into()),
            padding: (2.0, 2.0),
        },
        BLUE, // highlight
        GREY, // empty_ws
        config.workspaces().clone(),
    )
    .expect("unable to set the top bar hook");

    thread_local! {
        static BAR_IS_ENABLED: Rc<RefCell<bool>> = Rc::new(RefCell::new(true));
    }

    // let id = bar.id;

    let hooks: XcbHooks = vec![sp.get_hook(), Box::new(bar)];

    let key_bindings = gen_keybindings! {
        // stack management
        "M-d" => run_external!("dmenu_run");
        "M-Return" => run_external!("st");
        "M-S-Return" => sp.toggle();
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
        "M-S-q" => run_internal!(exit);

        // layouts
        "M-t" => run_internal!(cycle_layout, Forward);

        // toggle status bar
        "M-b" => Box::new(|wm: &mut WindowManager<_>| {
            let id = match wm.client(&Selector::Condition(&|client: &Client| client.wm_name() == "penrose-statusbar")) {
                Some(client) => client.id(),
                None => return Ok(())
            };
            if BAR_IS_ENABLED.with(|v| *v.borrow()) {
                BAR_IS_ENABLED.with(|v| *v.borrow_mut() = false);
                wm.hide_client(id)
            } else {
                BAR_IS_ENABLED.with(|v| *v.borrow_mut() = true);
                match wm.focus_client(&Selector::WinId(id)) {
                    Err(_) => Err(PenroseError::MissingClientIds(vec![id])),
                    Ok(_) => Ok(())
                }
            }
        });

        // workspaces
        refmap [ config.ws_range() ] in {
            "M-{}" => focus_workspace [ index_selectors(config.workspaces().len()) ];
            "M-S-{}" => client_to_workspace [ index_selectors(config.workspaces().len()) ];
        };

        // applications
        "M-S-l" => run_external!("locker");
        "M-w" => run_external!("brave");
        "M-S-w" => run_external!("bookmarks2");
        "M-S-d" => run_external!("discord");
        "M-m" => run_external!("st -e ncmpcpp");
        "M-v" => run_external!("st -e alsamixer");
        "M-S-v" => run_external!("pavucontrol");
        "M-e" => run_external!("st -e ranger");
        "M-s" => run_external!("st -e stub");
        "M-S-p" => run_external!("flameshot gui");
        "M-p" => run_external!("stup");
    };

    let mouse_bindings = gen_mousebindings! {
        Press Right + [Meta] => |wm: &mut WindowManager<_>, _: &MouseEvent| wm.cycle_workspace(Forward),
        Press Left + [Meta] => |wm: &mut WindowManager<_>, _: &MouseEvent| wm.cycle_workspace(Backward)
    };

    let mut wm = new_xcb_backed_window_manager(config, hooks, logging_error_handler())
        .expect("unable to create window manager");
    wm.grab_keys_and_run(key_bindings, mouse_bindings)
        .expect("unable to start window manager");
}
