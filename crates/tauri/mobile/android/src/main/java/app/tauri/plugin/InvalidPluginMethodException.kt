package app.tauri.plugin

internal class InvalidCommandException : Exception {
  constructor(s: String?) : super(s) {}
  constructor(t: Throwable?) : super(t) {}
  constructor(s: String?, t: Throwable?) : super(s, t) {}
}
