#[cfg(feature = "bevy-integration")]
use bevy::prelude::*;

#[cfg(feature = "bevy-integration")]
use crate::ast::Value;
#[cfg(feature = "bevy-integration")]
use crate::environment::Environment;
#[cfg(feature = "bevy-integration")]
use crate::error::Result;
#[cfg(feature = "bevy-integration")]
use crate::ast::{Function, FunctionBody, NativeFunction};

#[cfg(feature = "bevy-integration")]
/// Load Bevy-specific functions into the environment
pub fn load_bevy_stdlib(env: &mut Environment) {
    register_bevy_native(env, "ui_spawn_button", 2, ui_spawn_button);
    register_bevy_native(env, "ui_spawn_text", 1, ui_spawn_text);
    register_bevy_native(env, "ui_spawn_container", 0, ui_spawn_container);
    register_bevy_native(env, "ui_set_position", 4, ui_set_position);
    register_bevy_native(env, "ui_set_size", 3, ui_set_size);
    register_bevy_native(env, "ui_set_color", 5, ui_set_color);
}

#[cfg(feature = "bevy-integration")]
fn register_bevy_native(
    env: &mut Environment,
    name: &str,
    arity: usize,
    func: fn(&[Value]) -> Result<Value>,
) {
    let native = NativeFunction {
        name: name.to_string(),
        arity,
        func,
    };
    env.define(
        name.to_string(),
        Value::Function(Function {
            name: name.to_string(),
            params: (0..arity).map(|i| format!("arg{}", i)).collect(),
            body: FunctionBody::Native(native),
        }),
    );
}

// UI Functions (these would integrate with the actual Bevy world)
// For now, they return placeholder entities

#[cfg(feature = "bevy-integration")]
fn ui_spawn_button(_args: &[Value]) -> Result<Value> {
    // In a real implementation, this would:
    // 1. Get the text from args[0]
    // 2. Get the callback from args[1]
    // 3. Spawn a button entity in the Bevy world
    // 4. Return the entity ID

    // Placeholder implementation
    use crate::error::FusabiError;
    Err(FusabiError::runtime_error(
        "ui_spawn_button requires integration with Bevy World (not available in this context)",
    ))
}

#[cfg(feature = "bevy-integration")]
fn ui_spawn_text(_args: &[Value]) -> Result<Value> {
    use crate::error::FusabiError;
    Err(FusabiError::runtime_error(
        "ui_spawn_text requires integration with Bevy World (not available in this context)",
    ))
}

#[cfg(feature = "bevy-integration")]
fn ui_spawn_container(_args: &[Value]) -> Result<Value> {
    use crate::error::FusabiError;
    Err(FusabiError::runtime_error(
        "ui_spawn_container requires integration with Bevy World (not available in this context)",
    ))
}

#[cfg(feature = "bevy-integration")]
fn ui_set_position(_args: &[Value]) -> Result<Value> {
    use crate::error::FusabiError;
    Err(FusabiError::runtime_error(
        "ui_set_position requires integration with Bevy World (not available in this context)",
    ))
}

#[cfg(feature = "bevy-integration")]
fn ui_set_size(_args: &[Value]) -> Result<Value> {
    use crate::error::FusabiError;
    Err(FusabiError::runtime_error(
        "ui_set_size requires integration with Bevy World (not available in this context)",
    ))
}

#[cfg(feature = "bevy-integration")]
fn ui_set_color(_args: &[Value]) -> Result<Value> {
    use crate::error::FusabiError;
    Err(FusabiError::runtime_error(
        "ui_set_color requires integration with Bevy World (not available in this context)",
    ))
}

#[cfg(feature = "bevy-integration")]
/// Resource to hold the Fusabi interpreter
#[derive(Resource)]
pub struct FusabiInterpreter {
    interpreter: crate::interpreter::Interpreter,
}

#[cfg(feature = "bevy-integration")]
impl FusabiInterpreter {
    pub fn new() -> Self {
        let mut interpreter = crate::interpreter::Interpreter::new();
        // Load Bevy-specific standard library
        load_bevy_stdlib(interpreter.env_mut());
        Self { interpreter }
    }

    pub fn eval(&mut self, code: &str) -> Result<Value> {
        let module = crate::parser::parse_module(code)?;
        self.interpreter.eval_module(&module)
    }

    pub fn interpreter(&self) -> &crate::interpreter::Interpreter {
        &self.interpreter
    }

    pub fn interpreter_mut(&mut self) -> &mut crate::interpreter::Interpreter {
        &mut self.interpreter
    }
}

#[cfg(feature = "bevy-integration")]
impl Default for FusabiInterpreter {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(feature = "bevy-integration")]
/// Plugin to integrate Fusabi interpreter with Bevy
pub struct FusabiPlugin;

#[cfg(feature = "bevy-integration")]
impl Plugin for FusabiPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(FusabiInterpreter::new())
            .add_systems(Update, hot_reload_system);
    }
}

#[cfg(feature = "bevy-integration")]
/// Event for script reloading
#[derive(Event)]
pub struct ScriptReloadEvent {
    pub path: std::path::PathBuf,
    pub content: String,
}

#[cfg(feature = "bevy-integration")]
/// System to handle hot-reloading of scripts
fn hot_reload_system(
    mut events: EventReader<ScriptReloadEvent>,
    mut interpreter: ResMut<FusabiInterpreter>,
) {
    for event in events.read() {
        match interpreter.eval(&event.content) {
            Ok(value) => {
                info!("Reloaded script {:?}: {}", event.path, value);
            }
            Err(e) => {
                error!("Failed to reload script {:?}: {}", event.path, e);
            }
        }
    }
}
