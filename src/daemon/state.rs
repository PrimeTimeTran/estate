// use std::{
//     fs,
//     path::PathBuf,
//     time::{SystemTime, UNIX_EPOCH},
// };

// use serde::{Deserialize, Serialize};

// #[derive(Default, Debug, Serialize, Deserialize)]
// pub struct DaemonState {
//     pub starts: u64,
//     pub status_checks: u64,
//     pub started_at: i64,
//     pub longest_run: u64,
// }

// impl DaemonState {
//     // pub fn load() -> DaemonState {
//     //     let p = path();

//     //     if !p.exists() {
//     //         return Default::default();
//     //     }

//     //     serde_json::from_str(&fs::read_to_string(p).unwrap()).unwrap()
//     // }

//     // pub fn save(state: &DaemonState) {
//     //     let p = path();
//     //     fs::create_dir_all(p.parent().unwrap()).unwrap();
//     //     fs::write(p, serde_json::to_string_pretty(state).unwrap()).unwrap();
//     // }

//     pub fn save_workspace(path: &PathBuf) {
//         println!("💾 save_workspace not implemented yet: {:?}", path);
//     }

//     pub fn now() -> u64 {
//         SystemTime::now()
//             .duration_since(UNIX_EPOCH)
//             .unwrap()
//             .as_secs()
//     }
// }

// impl DaemonState {
//     fn path() -> std::io::Result<PathBuf> {
//         Ok(crate::daemon::resolver::engine_data_dir()?.join("state.json"))
//     }

//     pub fn load() -> Self {
//         let path = Self::path().expect("could not resolve daemon state path");

//         if !path.exists() {
//             return Self {
//                 starts: 0,
//                 status_checks: 0,
//                 started_at: 0,
//                 longest_run: 0,
//             };
//         }

//         let raw = std::fs::read_to_string(path).expect("failed reading daemon state");

//         serde_json::from_str(&raw).expect("failed parsing daemon state")
//     }

//     pub fn save(state: &Self) {
//         let path = Self::path().expect("could not resolve daemon state path");

//         let json = serde_json::to_string_pretty(state).expect("failed serializing daemon state");

//         std::fs::write(path, json).expect("failed writing daemon state");
//     }

//     pub fn record_status_check() {
//         let mut state = Self::load();

//         state.status_checks += 1;

//         Self::save(&state);
//     }
// }

// fn path() -> PathBuf {
//     dirs::home_dir()
//         .unwrap()
//         .join(".leviticus")
//         .join("state.json")
// }
