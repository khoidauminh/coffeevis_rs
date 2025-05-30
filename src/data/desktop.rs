const DESKTOP_FILE_ICON: &'static [u8] = include_bytes!("../../assets/coffeevis_icon.svg");

use super::log::{error, info};
use std::{
    fs::File,
    io::Write,
    path::{Path, PathBuf},
    process::Command,
};

use xdg;

pub fn create_tmp_desktop_file() {
    let xdg_dirs = xdg::BaseDirectories::new();

    let args = {
        let args = std::env::args().collect::<Vec<_>>();
        let Some(exec) = args
            .first()
            .map(|x| Path::new(x).canonicalize().unwrap_or(PathBuf::from(x)))
        else {
            error!("Failed to determine the executable path.");
            return;
        };

        let Some(exec) = exec.to_str() else {
            error!("Failed to process executable path.\n");
            return;
        };

        exec.to_owned() + &args.iter().skip(1).fold(String::new(), |a, x| a + " " + &x)
    };

    let Some(share_path) = xdg_dirs.get_data_home() else {
        error!("Local share path not found.");
        return;
    };

    let mut icon_path = share_path.clone();
    let mut desktop_path = share_path.clone();

    icon_path.push("icons/coffeevis_icon.svg");
    desktop_path.push("applications/coffeevis.desktop");

    let Ok(mut icon) = File::create(&icon_path) else {
        error!("Faile to create icon file.\n");
        return;
    };

    let Ok(mut desktop) = File::create(desktop_path) else {
        error!("Faile to create desktop entry file.\n");
        return;
    };

    let Ok(_) = icon.write_all(DESKTOP_FILE_ICON) else {
        error!("Faile to write to icon file.\n");
        return;
    };

    let desktop_content = format!(
        "[Desktop Entry]\nName=Coffeevis\nType=Application\nExec={}\nIcon={}\n",
        args,
        icon_path.display()
    );

    let Ok(_) = desktop.write_all(desktop_content.as_bytes()) else {
        error!("Faile to write to desktop entry file.\n");
        return;
    };

    info!("Now running update-desktop-database...");

    let update_command_ret = Command::new("update-desktop-database").output();

    if let Ok(s) = update_command_ret {
        info!("Status of command: {}", s.status);
    }
}

#[allow(dead_code)]
pub fn __create_tmp_desktop_file() {
    let base_path = "/tmp/coffeevis/";
    let icon_path = base_path.to_owned() + "coffeevis_icon.svg";
    let desktop_path = base_path.to_owned() + "coffeevis.desktop";

    let args = {
        let args = std::env::args().collect::<Vec<_>>();
        let Some(exec) = args
            .first()
            .map(|x| Path::new(x).canonicalize().unwrap_or(PathBuf::from(x)))
        else {
            error!("Failed to determine the executable path.");
            return;
        };

        let Some(exec) = exec.to_str() else {
            error!("Failed to process executable path.\n");
            return;
        };

        exec.to_owned() + &args.iter().skip(1).fold(String::new(), |a, x| a + " " + &x)
    };

    if std::fs::create_dir(base_path).is_err() {
        error!("Failed to create {} or it already exists", base_path);
        return;
    }

    let Ok(mut icon) = File::create(&icon_path) else {
        error!("Failed to create icon file.\n");
        return;
    };

    let Ok(mut desktop) = File::create(desktop_path) else {
        error!("Failed to create desktop entry file.\n");
        return;
    };

    let Ok(_) = icon.write_all(DESKTOP_FILE_ICON) else {
        error!("Failed to write to icon file.\n");
        return;
    };

    let desktop_content = format!(
        "[Desktop Entry]\nName=Coffeevis\nType=Application\nExec={}\nIcon={}\n",
        args, icon_path
    );

    let Ok(_) = desktop.write_all(desktop_content.as_bytes()) else {
        error!("Faile to write to desktop entry file.\n");
        return;
    };

    info!("Now running update-desktop-database...");

    let update_command_ret = Command::new("update-desktop-database")
        .arg(base_path)
        .output();

    if let Ok(s) = update_command_ret {
        info!("Status of command: {}", s.status);
    }
}
