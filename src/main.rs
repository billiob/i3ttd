#[macro_use]
extern crate serde_derive;

extern crate serde_json;

#[macro_use]
extern crate debug_macros;
extern crate docopt;
extern crate i3ipc;
extern crate time;


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
Time Tracking Deamon for i3.

Usage: i3ttd [options] <config_file>
       i3ttd -v | --version

Options:
    -h, --help           Show this message
    -v, --version        Show the version
    <config_file>        A JSON configuration file
";

struct State {
    current_category: Option<String>,
    last_time: time::Timespec,
}
struct Ctx {
    cfg: Config,
    state: State,
}

fn finish_old_category(state: &mut State, cat: String)
{
    let now = time::now();
    let now_tm = now.to_timespec();
    state.current_category = None;
    let diff = now_tm - state.last_time;
    let num_seconds = diff.num_seconds();
    if num_seconds > 0 {
        println!("{},{},{}", now.rfc3339(),num_seconds, cat);
    }
}

fn new_category(state: &mut State, cat: &String)
{
    let now = time::now().to_timespec();
    state.current_category = Some(cat.clone());
    state.last_time = now;
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
        if let Some(cat) = ctx.state.current_category.take() {
            finish_old_category(&mut ctx.state, cat);
        }
        return;
    }
    let name = n.name.unwrap();
    for w in &ctx.cfg.workspaces {
        if w.name == name {
            if let Some(cat) = ctx.state.current_category.take() {
                if cat != w.category {
                    finish_old_category(&mut ctx.state, cat);
                    new_category(&mut ctx.state, &w.category);
                }
            } else {
                new_category(&mut ctx.state, &w.category);
            }
            return;
        }
    }
    if let Some(cat) = ctx.state.current_category.take() {
        finish_old_category(&mut ctx.state, cat);
    }
}

fn load_config(file_path: &str) -> Config {
    let contents = File::open(file_path).unwrap();
    serde_json::from_reader(contents).unwrap()
}

fn main() {
    let version = env!("CARGO_PKG_VERSION").to_owned();
    let args = Docopt::new(USAGE)
                      .and_then(|dopt| dopt.version(Some(version)).parse())
                      .unwrap_or_else(|e| e.exit());

    let config_file = args.get_str("<config_file>");
    let mut ctx = Ctx {
        cfg: load_config(config_file),
        state: State {
            current_category: None,
            last_time: time::now().to_timespec(),
        },
    };
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
