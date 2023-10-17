// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

//! The tauri plugin to create and manipulate windows from JS.

use crate::{
  plugin::{Builder, TauriPlugin},
  Runtime,
};

#[cfg(desktop)]
mod desktop_commands {
  use serde::Deserialize;
  use tauri_utils::ProgressBarState;

  use super::*;
  use crate::{
    command,
    utils::config::{WindowConfig, WindowEffectsConfig},
    AppHandle, CursorIcon, Icon, Manager, Monitor, PhysicalPosition, PhysicalSize, Position, Size,
    Theme, UserAttentionType, Window, WindowBuilder,
  };

  #[derive(Deserialize)]
  #[serde(untagged)]
  pub enum IconDto {
    #[cfg(any(feature = "icon-png", feature = "icon-ico"))]
    File(std::path::PathBuf),
    #[cfg(any(feature = "icon-png", feature = "icon-ico"))]
    Raw(Vec<u8>),
    Rgba {
      rgba: Vec<u8>,
      width: u32,
      height: u32,
    },
  }

  impl From<IconDto> for Icon {
    fn from(icon: IconDto) -> Self {
      match icon {
        #[cfg(any(feature = "icon-png", feature = "icon-ico"))]
        IconDto::File(path) => Self::File(path),
        #[cfg(any(feature = "icon-png", feature = "icon-ico"))]
        IconDto::Raw(raw) => Self::Raw(raw),
        IconDto::Rgba {
          rgba,
          width,
          height,
        } => Self::Rgba {
          rgba,
          width,
          height,
        },
      }
    }
  }

  #[command(root = "crate")]
  pub async fn create<R: Runtime>(app: AppHandle<R>, options: WindowConfig) -> crate::Result<()> {
    WindowBuilder::from_config(&app, options).build()?;
    Ok(())
  }

  fn get_window<R: Runtime>(window: Window<R>, label: Option<String>) -> crate::Result<Window<R>> {
    match label {
      Some(l) if !l.is_empty() => window.get_window(&l).ok_or(crate::Error::WindowNotFound),
      _ => Ok(window),
    }
  }

  macro_rules! getter {
    ($cmd: ident, $ret: ty) => {
      #[command(root = "crate")]
      pub async fn $cmd<R: Runtime>(
        window: Window<R>,
        label: Option<String>,
      ) -> crate::Result<$ret> {
        get_window(window, label)?.$cmd().map_err(Into::into)
      }
    };
  }

  macro_rules! setter {
    ($cmd: ident) => {
      #[command(root = "crate")]
      pub async fn $cmd<R: Runtime>(window: Window<R>, label: Option<String>) -> crate::Result<()> {
        get_window(window, label)?.$cmd().map_err(Into::into)
      }
    };

    ($cmd: ident, $input: ty) => {
      #[command(root = "crate")]
      pub async fn $cmd<R: Runtime>(
        window: Window<R>,
        label: Option<String>,
        value: $input,
      ) -> crate::Result<()> {
        get_window(window, label)?.$cmd(value).map_err(Into::into)
      }
    };
  }

  getter!(scale_factor, f64);
  getter!(inner_position, PhysicalPosition<i32>);
  getter!(outer_position, PhysicalPosition<i32>);
  getter!(inner_size, PhysicalSize<u32>);
  getter!(outer_size, PhysicalSize<u32>);
  getter!(is_fullscreen, bool);
  getter!(is_minimized, bool);
  getter!(is_maximized, bool);
  getter!(is_focused, bool);
  getter!(is_decorated, bool);
  getter!(is_resizable, bool);
  getter!(is_maximizable, bool);
  getter!(is_minimizable, bool);
  getter!(is_closable, bool);
  getter!(is_visible, bool);
  getter!(title, String);
  getter!(current_monitor, Option<Monitor>);
  getter!(primary_monitor, Option<Monitor>);
  getter!(available_monitors, Vec<Monitor>);
  getter!(theme, Theme);

  setter!(center);
  setter!(request_user_attention, Option<UserAttentionType>);
  setter!(set_resizable, bool);
  setter!(set_maximizable, bool);
  setter!(set_minimizable, bool);
  setter!(set_closable, bool);
  setter!(set_title, &str);
  setter!(maximize);
  setter!(unmaximize);
  setter!(minimize);
  setter!(unminimize);
  setter!(show);
  setter!(hide);
  setter!(close);
  setter!(set_decorations, bool);
  setter!(set_shadow, bool);
  setter!(set_effects, Option<WindowEffectsConfig>);
  setter!(set_always_on_top, bool);
  setter!(set_content_protected, bool);
  setter!(set_size, Size);
  setter!(set_min_size, Option<Size>);
  setter!(set_max_size, Option<Size>);
  setter!(set_position, Position);
  setter!(set_fullscreen, bool);
  setter!(set_focus);
  setter!(set_skip_taskbar, bool);
  setter!(set_cursor_grab, bool);
  setter!(set_cursor_visible, bool);
  setter!(set_cursor_icon, CursorIcon);
  setter!(set_cursor_position, Position);
  setter!(set_ignore_cursor_events, bool);
  setter!(start_dragging);
  setter!(set_progress_bar, ProgressBarState);
  setter!(print);

  #[command(root = "crate")]
  pub async fn set_icon<R: Runtime>(
    window: Window<R>,
    label: Option<String>,
    value: IconDto,
  ) -> crate::Result<()> {
    get_window(window, label)?
      .set_icon(value.into())
      .map_err(Into::into)
  }

  #[command(root = "crate")]
  pub async fn toggle_maximize<R: Runtime>(
    window: Window<R>,
    label: Option<String>,
  ) -> crate::Result<()> {
    let window = get_window(window, label)?;
    match window.is_maximized()? {
      true => window.unmaximize()?,
      false => window.maximize()?,
    };
    Ok(())
  }

  #[command(root = "crate")]
  pub async fn internal_toggle_maximize<R: Runtime>(
    window: Window<R>,
    label: Option<String>,
  ) -> crate::Result<()> {
    let window = get_window(window, label)?;
    if window.is_resizable()? {
      match window.is_maximized()? {
        true => window.unmaximize()?,
        false => window.maximize()?,
      };
    }
    Ok(())
  }

  #[cfg(any(debug_assertions, feature = "devtools"))]
  #[command(root = "crate")]
  pub async fn internal_toggle_devtools<R: Runtime>(
    window: Window<R>,
    label: Option<String>,
  ) -> crate::Result<()> {
    let window = get_window(window, label)?;
    if window.is_devtools_open() {
      window.close_devtools();
    } else {
      window.open_devtools();
    }
    Ok(())
  }
}

/// Initializes the plugin.
pub fn init<R: Runtime>() -> TauriPlugin<R> {
  let mut init_script = String::new();
  // window.print works on Linux/Windows; need to use the API on macOS
  #[cfg(any(target_os = "macos", target_os = "ios"))]
  {
    init_script.push_str(include_str!("./scripts/print.js"));
  }
  init_script.push_str(include_str!("./scripts/drag.js"));

  #[cfg(any(debug_assertions, feature = "devtools"))]
  {
    use serialize_to_javascript::{default_template, DefaultTemplate, Template};

    #[derive(Template)]
    #[default_template("./scripts/toggle-devtools.js")]
    struct Devtools<'a> {
      os_name: &'a str,
    }

    init_script.push_str(
      &Devtools {
        os_name: std::env::consts::OS,
      }
      .render_default(&Default::default())
      .unwrap()
      .into_string(),
    );
  }

  Builder::new("window")
    .js_init_script(init_script)
    .invoke_handler(|invoke| {
      #[cfg(desktop)]
      {
        let handler: Box<dyn Fn(crate::ipc::Invoke<R>) -> bool> =
          Box::new(crate::generate_handler![
            desktop_commands::create,
            // getters
            desktop_commands::scale_factor,
            desktop_commands::inner_position,
            desktop_commands::outer_position,
            desktop_commands::inner_size,
            desktop_commands::outer_size,
            desktop_commands::is_fullscreen,
            desktop_commands::is_minimized,
            desktop_commands::is_maximized,
            desktop_commands::is_focused,
            desktop_commands::is_decorated,
            desktop_commands::is_resizable,
            desktop_commands::is_maximizable,
            desktop_commands::is_minimizable,
            desktop_commands::is_closable,
            desktop_commands::is_visible,
            desktop_commands::title,
            desktop_commands::current_monitor,
            desktop_commands::primary_monitor,
            desktop_commands::available_monitors,
            desktop_commands::theme,
            // setters
            desktop_commands::center,
            desktop_commands::request_user_attention,
            desktop_commands::set_resizable,
            desktop_commands::set_maximizable,
            desktop_commands::set_minimizable,
            desktop_commands::set_closable,
            desktop_commands::set_title,
            desktop_commands::maximize,
            desktop_commands::unmaximize,
            desktop_commands::minimize,
            desktop_commands::unminimize,
            desktop_commands::show,
            desktop_commands::hide,
            desktop_commands::close,
            desktop_commands::set_decorations,
            desktop_commands::set_shadow,
            desktop_commands::set_effects,
            desktop_commands::set_always_on_top,
            desktop_commands::set_content_protected,
            desktop_commands::set_size,
            desktop_commands::set_min_size,
            desktop_commands::set_max_size,
            desktop_commands::set_position,
            desktop_commands::set_fullscreen,
            desktop_commands::set_focus,
            desktop_commands::set_skip_taskbar,
            desktop_commands::set_cursor_grab,
            desktop_commands::set_cursor_visible,
            desktop_commands::set_cursor_icon,
            desktop_commands::set_cursor_position,
            desktop_commands::set_ignore_cursor_events,
            desktop_commands::start_dragging,
            desktop_commands::set_progress_bar,
            desktop_commands::print,
            desktop_commands::set_icon,
            desktop_commands::toggle_maximize,
            desktop_commands::internal_toggle_maximize,
            #[cfg(any(debug_assertions, feature = "devtools"))]
            desktop_commands::internal_toggle_devtools,
          ]);
        #[allow(clippy::needless_return)]
        return handler(invoke);
      }
      #[cfg(mobile)]
      {
        invoke.resolver.reject("Window API not available on mobile");
        return true;
      }
    })
    .build()
}