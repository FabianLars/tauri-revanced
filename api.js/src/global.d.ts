/** @ignore */

import type { invoke, transformCallback, convertFileSrc } from './core';

/** @ignore */
declare global {
    interface Window {
        __TAURI_INTERNALS__: {
            invoke: typeof invoke;
            transformCallback: typeof transformCallback;
            convertFileSrc: typeof convertFileSrc;
            ipc: (message: {
                cmd: string;
                callback: number;
                error: number;
                payload: unknown;
                options?: InvokeOptions;
            }) => void;
            metadata: {
                windows: WindowDef[];
                currentWindow: WindowDef;
                webviews: WebviewDef[];
                currentWebview: WebviewDef;
            };
            plugins: {
                path: {
                    sep: string;
                    delimiter: string;
                };
            };
        };
    }
}

/** @ignore */
interface WebviewDef {
    windowLabel: string;
    label: string;
}

/** @ignore */
interface WindowDef {
    label: string;
}
