/**
 * The Tauri API allows you to interface with the backend layer.
 *
 * This module exposes all other modules as an object where the key is the module name, and the value is the module exports.
 * @example
 * ```typescript
 * import { event, window, path } from '@tauri-apps/api'
 * ```
 * @module
 */

import * as app from './app';
import * as event from './event';
import * as core from './core';
import * as window from './window';
import * as webview from './webview';
import * as webviewWindow from './webviewWindow';
import * as path from './path';
import * as dpi from './dpi';
import * as tray from './tray';
import * as menu from './menu';
import * as image from './image';

export { app, dpi, event, path, core, window, webview, webviewWindow, tray, menu, image };
