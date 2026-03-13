use std::fs;
use std::path::PathBuf;

const DB_FILE_NAME: &str = "mdcraft.sqlite3";

fn resolve_app_data_dir() -> Result<PathBuf, String> {
    #[cfg(target_os = "linux")]
    {
        if let Some(path) = std::env::var_os("XDG_DATA_HOME") {
            return Ok(PathBuf::from(path).join("mdcraft"));
        }

        if let Some(home) = std::env::var_os("HOME") {
            return Ok(PathBuf::from(home).join(".local/share/mdcraft"));
        }

        return Err("Nao foi possivel resolver o diretorio de dados no Linux.".to_string());
    }

    #[cfg(target_os = "windows")]
    {
        if let Some(path) = std::env::var_os("APPDATA") {
            return Ok(PathBuf::from(path).join("mdcraft"));
        }

        if let Some(profile) = std::env::var_os("USERPROFILE") {
            return Ok(PathBuf::from(profile).join("AppData/Roaming/mdcraft"));
        }

        return Err("Nao foi possivel resolver o diretorio de dados no Windows.".to_string());
    }

    #[cfg(not(any(target_os = "linux", target_os = "windows")))]
    {
        std::env::current_dir()
            .map(|dir| dir.join("mdcraft-data"))
            .map_err(|err| format!("Nao foi possivel resolver diretorio de dados: {err}"))
    }
}

pub(super) fn sqlite_db_path() -> Result<PathBuf, String> {
    let app_dir = resolve_app_data_dir()?;
    fs::create_dir_all(&app_dir)
        .map_err(|err| format!("Nao foi possivel criar diretorio de dados: {err}"))?;
    Ok(app_dir.join(DB_FILE_NAME))
}
