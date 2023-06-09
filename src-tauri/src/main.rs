#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

use beiboot_desktop::connection::{get_connector_context, PortMapping, TLSFiles};
use tauri_plugin_oauth::start;

mod util;

fn main() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![connect_beiboot_ghostunnel, disconnect_beiboot_ghostunnel, write_kubeconfig, cleanup, start_server])
        .plugin(tauri_plugin_store::Builder::default().build())
        .on_window_event(|event| {
            match event.event() {
                tauri::WindowEvent::CloseRequested { .. } => {
                    get_connector_context("", "GhostunnelDocker").disconnect().unwrap();
                }
                _ => {}
            }
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

#[tauri::command]
async fn start_server(window: tauri::Window) -> Result<u16, String> {
    let tauri_url = window.url();
    start(move |url| {
        let params = url.split("#").collect::<Vec<&str>>();
        window.eval(format!("window.location.replace('{}#{}')", tauri_url, params[1]).as_str()).unwrap();
    })
    .map_err(|e| e.to_string())
}

#[tauri::command]
fn connect_beiboot_ghostunnel(beiboot_name: String, ports: Vec<PortMapping>, ca: &str, cl_cert: &str, cl_key: &str) -> Result<String, String> {
    let connector = get_connector_context(&beiboot_name, "GhostunnelDocker");
    let ca_cert_path = util::write_conf_file(beiboot_name.clone(), ca, "ca.crt").unwrap();
    let client_cert_path = util::write_conf_file(beiboot_name.clone(), cl_cert, "client.crt").unwrap();
    let client_key_path = util::write_conf_file(beiboot_name.clone(), cl_key, "client.key").unwrap();

    let tls = TLSFiles {
        ca_cert_path: &ca_cert_path,
        client_cert_path: &client_cert_path,
        client_key_path: &client_key_path,
    };

    match connector.connect(&ports, &tls) {
        Ok(_) => Ok("Cluster connected successfully".into()),
        Err(why) => {
            println!("{}", why);
            Err(format!("{}", why))
        }
    }
}

#[tauri::command]
fn disconnect_beiboot_ghostunnel(beiboot_name: String) -> Result<String, String> {
    let connector = get_connector_context(&beiboot_name, "GhostunnelDocker");
    match connector.disconnect() {
        Ok(_) => Ok("Cluster disconnected successfully".into()),
        Err(why) => {
            println!("{}", why);
            Err(format!("{}", why))
        }
    }
}

#[tauri::command]
fn write_kubeconfig(beiboot_name: String, kubeconfig: String) -> Result<String, String> {
    match util::write_conf_file(beiboot_name, &kubeconfig, "kubeconfig.yaml") {
        Ok(path) => Ok(path),
        Err(why) => {
            println!("{}", why);
            Err(why)
        }
    }
}

#[tauri::command]
fn cleanup(beiboot_name: String) -> Result<(), String> {
    match util::cleanup(beiboot_name) {
        Ok(_) => Ok(()),
        Err(why) => {
            println!("{}", why);
            Err(why)
        }
    }
}
