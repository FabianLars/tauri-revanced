// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use json_patch::merge;
use serde_json::Value as JsonValue;

pub use tauri_utils::{config::*, platform::Target};

use std::{
    collections::HashMap,
    env::{current_dir, set_current_dir, set_var},
    ffi::OsStr,
    process::exit,
    sync::{Arc, Mutex, OnceLock},
};

pub const MERGE_CONFIG_EXTENSION_NAME: &str = "--config";

pub struct ConfigMetadata {
    /// The current target.
    target: Target,
    /// The actual configuration, merged with any extension.
    inner: Config,
    /// The config extensions (platform-specific config files or the config CLI argument).
    /// Maps the extension name to its value.
    extensions: HashMap<String, JsonValue>,
}

impl std::ops::Deref for ConfigMetadata {
    type Target = Config;

    #[inline(always)]
    fn deref(&self) -> &Config {
        &self.inner
    }
}

impl ConfigMetadata {
    /// Checks which config is overwriting the bundle identifier.
    pub fn find_bundle_identifier_overwriter(&self) -> Option<String> {
        for (ext, config) in &self.extensions {
            if let Some(identifier) = config
                .as_object()
                .and_then(|config| config.get("tauri"))
                .and_then(|tauri_config| tauri_config.as_object())
                .and_then(|tauri_config| tauri_config.get("bundle"))
                .and_then(|bundle_config| bundle_config.as_object())
                .and_then(|bundle_config| bundle_config.get("identifier"))
                .and_then(|id| id.as_str())
            {
                if identifier == self.inner.identifier {
                    return Some(ext.clone());
                }
            }
        }
        None
    }
}

pub type ConfigHandle = Arc<Mutex<Option<ConfigMetadata>>>;

fn config_handle() -> &'static ConfigHandle {
    static CONFIG_HANDLE: OnceLock<ConfigHandle> = OnceLock::new();
    CONFIG_HANDLE.get_or_init(Default::default)
}

/// Gets the static parsed config from `tauri.conf.json`.
fn get_internal(
    merge_config: Option<&serde_json::Value>,
    reload: bool,
    target: Target,
) -> crate::Result<ConfigHandle> {
    if !reload && config_handle().lock().unwrap().is_some() {
        return Ok(config_handle().clone());
    }

    let tauri_dir = super::app_paths::tauri_dir();
    let (mut config, config_path) =
        tauri_utils::config::parse::parse_value(target, tauri_dir.join("tauri.conf.json"))?;
    let config_file_name = config_path.file_name().unwrap().to_string_lossy();
    let mut extensions = HashMap::new();

    if let Some((platform_config, config_path)) =
        tauri_utils::config::parse::read_platform(target, tauri_dir)?
    {
        merge(&mut config, &platform_config);
        extensions.insert(
            config_path.file_name().unwrap().to_str().unwrap().into(),
            platform_config,
        );
    }

    if let Some(merge_config) = merge_config {
        let merge_config_str = serde_json::to_string(&merge_config).unwrap();
        set_var("TAURI_CONFIG", merge_config_str);
        merge(&mut config, merge_config);
        extensions.insert(MERGE_CONFIG_EXTENSION_NAME.into(), merge_config.clone());
    };

    if config_path.extension() == Some(OsStr::new("json"))
        || config_path.extension() == Some(OsStr::new("json5"))
    {
        let schema: JsonValue = serde_json::from_str(include_str!("../../schema.json"))?;
        let schema = jsonschema::JSONSchema::compile(&schema).unwrap();
        let result = schema.validate(&config);
        if let Err(errors) = result {
            for error in errors {
                let path = error.instance_path.clone().into_vec().join(" > ");
                if path.is_empty() {
                    log::error!("`{}` error: {}", config_file_name, error);
                } else {
                    log::error!("`{}` error on `{}`: {}", config_file_name, path, error);
                }
            }
            if !reload {
                exit(1);
            }
        }
    }

    // the `Config` deserializer for `package > version` can resolve the version from a path relative to the config path
    // so we actually need to change the current working directory here
    let current_dir = current_dir()?;
    set_current_dir(config_path.parent().unwrap())?;
    let config: Config = serde_json::from_value(config)?;
    // revert to previous working directory
    set_current_dir(current_dir)?;

    for (plugin, conf) in &config.plugins.0 {
        set_var(
            format!(
                "TAURI_{}_PLUGIN_CONFIG",
                plugin.to_uppercase().replace('-', "_")
            ),
            serde_json::to_string(&conf)?,
        );
    }

    *config_handle().lock().unwrap() = Some(ConfigMetadata {
        target,
        inner: config,
        extensions,
    });

    Ok(config_handle().clone())
}

pub fn get(
    target: Target,
    merge_config: Option<&serde_json::Value>,
) -> crate::Result<ConfigHandle> {
    get_internal(merge_config, false, target)
}

pub fn reload(merge_config: Option<&serde_json::Value>) -> crate::Result<ConfigHandle> {
    let target = config_handle()
        .lock()
        .unwrap()
        .as_ref()
        .map(|conf| conf.target);
    if let Some(target) = target {
        get_internal(merge_config, true, target)
    } else {
        Err(anyhow::anyhow!("config not loaded"))
    }
}