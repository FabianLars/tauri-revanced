package app.tauri.plugin

import app.tauri.annotation.Command
import java.lang.reflect.Method

class CommandData(
  val method: Method, methodDecorator: Command
) {

  // The name of the method
  val name: String = method.name
}
