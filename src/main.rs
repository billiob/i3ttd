#[macro_use]
extern crate serde_derive;

extern crate serde_json;

#[macro_use]
extern crate debug_macros;
extern crate docopt;
extern crate i3ipc;


use docopt::Docopt;

use std::fs::File;

use i3ipc::I3EventListener;
use i3ipc::Subscription;
use i3ipc::event::{Event, WorkspaceEventInfo};
use i3ipc::event::inner::WorkspaceChange;

#[derive(Deserialize, Debug)]
pub struct ConfigWorkspace {
    pub name: String,
    pub category: String,
}

#[derive(Deserialize, Debug)]
pub struct Config {
    pub workspaces: Vec<ConfigWorkspace>,
}


const USAGE: &'static str = "
Time Tracking server for i3.

Usage: i3ttd [options] <config_file>
       i3ttd -v | --version

Options:
    -h, --help           Show this message
    -v, --version        Show the version
    <config_file>        A JSON configuration file
";

struct Ctx {
    cfg: Config,
    current_category: Option<String>,
}

fn handle_workspace_event(ctx: &mut Ctx, e: WorkspaceEventInfo) {

    if let WorkspaceChange::Focus = e.change {
    } else {
        return;
    }

    if e.current.is_none() {
        return;
    }
    let n = e.current.unwrap();
    if n.name.is_none() {
        if ctx.current_category.is_none() {
            return;
        }
        ctx.current_category = None;
        /* TODO: finish old category */
        return;
    }
    let name = n.name.unwrap();
    for w in &ctx.cfg.workspaces {
        if w.name == name {
            if let Some(cat) = ctx.current_category.take() {
                if cat != w.category {
                    /* TODO: finish old category */
                    ctx.current_category = Some(w.category.clone());
                    /* TODO: new category */
                }
            } else {
                ctx.current_category = Some(w.category.clone());
                /* TODO: new category */
            }
            return;
        }
    }
    if let Some(_) = ctx.current_category.take() {
        /* TODO: finish old category */
        ctx.current_category = None;
    }
}

fn load_config(file_path: &str) -> Ctx {
    let contents = File::open(file_path).unwrap();
    let cfg = serde_json::from_reader(contents).unwrap();
    Ctx {
        cfg: cfg,
        current_category: None,
    }
}

fn main() {
    let version = env!("CARGO_PKG_VERSION").to_owned();
    let args = Docopt::new(USAGE)
                      .and_then(|dopt| dopt.version(Some(version)).parse())
                      .unwrap_or_else(|e| e.exit());

    let config_file = args.get_str("<config_file>");
    let mut ctx = load_config(config_file);
    // establish connection.
    let mut listener = I3EventListener::connect().unwrap();

    // subscribe to a couple events.
    let subs = [Subscription::Workspace];
    listener.subscribe(&subs).unwrap();

    // handle them
    for event in listener.listen() {
        match event.unwrap() {
            Event::WorkspaceEvent(e) => handle_workspace_event(&mut ctx, e),
            _ => unreachable!()
        }
    }
}
