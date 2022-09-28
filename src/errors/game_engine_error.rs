use std::io;

/// All errors that can occur with the game engine
#[derive(Debug)]
pub enum GameEngineError {
  FailedToReadLuaFile(io::Error),
  FailedToRunLuaFile(rlua::Error),
  MissingRequiredLuaMethod(&'static str, rlua::Error),
  FailedToRunMethod(&'static str, rlua::Error),
  JSONToLua(rlua::Error),
  LuaToJSON(rlua::Error),
}

impl GameEngineError {
  pub fn get_developer_notes(&self) -> String {
    match self {
      GameEngineError::FailedToReadLuaFile(error) => {
        format!("Failed to read Lua file: {}", error)
      },

      GameEngineError::FailedToRunLuaFile(error) => {
        format!("Failed to run Lua file: {}", error)
      },

      GameEngineError::MissingRequiredLuaMethod(method, error) => {
        format!("Missing required method: {} ({})", method, error)
      },

      GameEngineError::FailedToRunMethod(method, error) => {
        format!("Failed to run method {}: {}", method, error)
      },

      GameEngineError::JSONToLua(error) => {
        format!("Failed to serialize JSON to Lua value: {}", error)
      },

      GameEngineError::LuaToJSON(error) => {
        format!("Failed to serialize Lua to JSON value: {}", error)
      },
    }
  }
}
