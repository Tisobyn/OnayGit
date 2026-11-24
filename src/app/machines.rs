//! Other machines reached over SSH.
use crate::prelude::*;

#[derive(Clone, PartialEq, Debug)]
pub struct Machine {
    pub name: String,
    /// Anything `ssh` accepts, for example `me@server`.
    pub host: String,
}

fn machines_path() -> PathBuf {
    home_dir().join(".onaygit_machines")
}

pub fn load_machines() -> Vec<Machine> {
    std::fs::read_to_string(machines_path())
        .unwrap_or_default()
        .lines()
        .filter_map(|l| {
            let (name, host) = l.split_once('\t')?;
            Some(Machine { name: name.to_string(), host: host.to_string() })
        })
        .collect()
}

impl App {
    fn save_machines(&self) {
        let text: Vec<String> = self.machines.iter().map(|m| format!("{}\t{}", m.name, m.host)).collect();
        let _ = std::fs::write(machines_path(), text.join("\n"));
    }

    /// Test the connection first, then keep the machine.
    pub fn add_machine(&mut self, ctx: &egui::Context) {
        let host = self.machine_host.trim().to_string();
        if host.is_empty() {
            return;
        }
        let name = if self.machine_name.trim().is_empty() { host.clone() } else { self.machine_name.trim().to_string() };
        self.pending_machine = Some(Machine { name, host: host.clone() });
        self.spawn_task(ctx, &format!("{T_TEST}{host}"), move || git::test_machine(&host).map(|_| String::new()));
    }

    pub fn add_machine_confirmed(&mut self, m: Machine) {
        if !self.machines.iter().any(|x| x.host == m.host) {
            self.machines.push(m.clone());
            self.save_machines();
        }
        self.machine_name.clear();
        self.machine_host.clear();
        self.dialog = None;
        self.selected_machine = self.machines.iter().position(|x| x.host == m.host);
    }

    pub fn remove_machine(&mut self, i: usize) {
        if i < self.machines.len() {
            self.machines.remove(i);
            self.save_machines();
            self.selected_machine = None;
        }
    }

    pub fn scan_machine(&mut self, ctx: &egui::Context, host: &str) {
        let h = host.to_string();
        self.spawn_task(ctx, &format!("{T_SCAN}{host}"), move || {
            git::scan_machine(&h).map(|list| list.join("\n"))
        });
    }
}
