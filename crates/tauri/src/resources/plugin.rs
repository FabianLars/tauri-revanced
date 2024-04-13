use crate::{
    command,
    plugin::{Builder, TauriPlugin},
    Manager, Runtime, Webview,
};

use super::ResourceId;

#[command(root = "crate")]
fn close<R: Runtime>(webview: Webview<R>, rid: ResourceId) -> crate::Result<()> {
    webview.resources_table().close(rid)
}

pub(crate) fn init<R: Runtime>() -> TauriPlugin<R> {
    Builder::new("resources")
        .invoke_handler(crate::generate_handler![close])
        .build()
}
