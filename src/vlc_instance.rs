/*
* Copyright (c) 2025 xiSage
*
* This library is free software; you can redistribute it and/or
* modify it under the terms of the GNU Lesser General Public
* License as published by the Free Software Foundation; either
* version 2.1 of the License, or (at your option) any later version.
*
* This library is distributed in the hope that it will be useful,
* but WITHOUT ANY WARRANTY; without even the implied warranty of
* MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the GNU
* Lesser General Public License for more details.
*
* You should have received a copy of the GNU Lesser General Public
* License along with this library; if not, write to the Free Software
* Foundation, Inc., 51 Franklin Street, Fifth Floor, Boston, MA  02110-1301
* USA
*/

use crate::{util::cstring_from_gstring, vlc};
use godot::{
    classes::{class_macros::sys::GDEXTENSION_VARIANT_TYPE_STRING, notify::ObjectNotification, Engine, ProjectSettings},
    global::PropertyHint,
    prelude::*,
};
use printf::printf;
use std::{
    ffi::{c_char, c_int, c_void, CString},
    mem, ptr,
};

pub fn get() -> *mut vlc::libvlc_instance_t {
    Engine::singleton()
        .get_singleton("VLCInstance")
        .expect("VLCInstance not found")
        .cast::<VLCInstance>()
        .bind()
        .get_vlc_instance()
}

#[derive(GodotClass)]
#[class(base=Object, tool)]
pub struct VLCInstance {
    instance: Option<*mut vlc::libvlc_instance_t>,
    base: Base<Object>,
}

#[godot_api]
impl IObject for VLCInstance {
    fn init(base: Base<Object>) -> Self {
        // Setup log level setting
        if !ProjectSettings::singleton().has_setting("vlc/log_level") {
            ProjectSettings::singleton().set_setting("vlc/log_level", &Variant::from(4));
        }
        ProjectSettings::singleton().set_initial_value("vlc/log_level", &Variant::from(4));
        let mut info = Dictionary::new();
        let _ = info.insert("name", "vlc/log_level");
        let _ = info.insert("type", VariantType::INT);
        let _ = info.insert("hint", PropertyHint::ENUM);
        let _ = info.insert("hint_string", "Debug, Info, Warning, Error, Disabled");
        ProjectSettings::singleton().add_property_info(&info);
        ProjectSettings::singleton().set_restart_if_changed("vlc/log_level", true);
        let debug_level: i32 = ProjectSettings::singleton()
            .get_setting("vlc/log_level")
            .try_to()
            .unwrap();

        // Setup arguments setting
        if !ProjectSettings::singleton().has_setting("vlc/arguments") {
            ProjectSettings::singleton().set_setting("vlc/arguments", &Variant::from(Array::<GString>::new()));
        }
        ProjectSettings::singleton().set_initial_value("vlc/arguments", &Variant::from(Array::<GString>::new()));
        let mut info = Dictionary::new();
        let _ = info.insert("name", "vlc/arguments");
        let _ = info.insert("type", VariantType::ARRAY);
        let _ = info.insert("hint", PropertyHint::TYPE_STRING);
        let _ = info.insert("hint_string", format!("{}:", GDEXTENSION_VARIANT_TYPE_STRING));
        ProjectSettings::singleton().add_property_info(&info);
        ProjectSettings::singleton().set_restart_if_changed("vlc/arguments", true);

        // Setup Steam Deck compatibility setting
        if !ProjectSettings::singleton().has_setting("vlc/steamdeck_mode") {
            // Auto-detect Steam Deck by checking for SteamOS
            let is_steamdeck = std::env::var("SteamDeck").is_ok() ||
                              std::env::var("STEAM_RUNTIME").is_ok() ||
                              std::path::Path::new("/etc/steamos-release").exists();
            ProjectSettings::singleton().set_setting("vlc/steamdeck_mode", &Variant::from(is_steamdeck));
        }
        ProjectSettings::singleton().set_initial_value("vlc/steamdeck_mode", &Variant::from(false));
        let mut info = Dictionary::new();
        let _ = info.insert("name", "vlc/steamdeck_mode");
        let _ = info.insert("type", VariantType::BOOL);
        let _ = info.insert("hint", PropertyHint::NONE);
        let _ = info.insert("hint_string", "Enable compatibility mode for Steam Deck (uses system VLC)");
        ProjectSettings::singleton().add_property_info(&info);
        ProjectSettings::singleton().set_restart_if_changed("vlc/steamdeck_mode", true);

        let steamdeck_mode: bool = ProjectSettings::singleton()
            .get_setting("vlc/steamdeck_mode")
            .try_to()
            .unwrap_or(false);

        // Add hardware acceleration for Steam Deck if enabled
        let mut arguments: Array<GString> = ProjectSettings::singleton()
            .get_setting("vlc/arguments")
            .try_to()
            .unwrap_or_default();

        if steamdeck_mode {
            godot_print!("VLC: Steam Deck mode enabled - using system VLC with hardware acceleration");
            // Add hardware acceleration arguments for Steam Deck
            if !arguments.as_slice().iter().any(|s| s.to_string().contains("avcodec-hw")) {
                arguments.push("--avcodec-hw=any".into());
            }
        }

        let args: Vec<CString> = arguments.iter_shared().map(cstring_from_gstring).collect();
        let argc = args.len() as c_int;
        let args: Vec<_> = args.iter().map(|s| s.as_ptr()).collect();
        let argv = args.as_ptr();

        let instance = unsafe { vlc::libvlc_new(argc, argv) };

        // Log VLC version for debugging
        if !instance.is_null() {
            let version = unsafe {
                let ver_ptr = vlc::libvlc_get_version();
                if !ver_ptr.is_null() {
                    std::ffi::CStr::from_ptr(ver_ptr).to_string_lossy().into_owned()
                } else {
                    "unknown".to_string()
                }
            };
            godot_print!("VLC initialized successfully. Version: {}", version);
        } else {
            godot_error!("VLC failed to initialize! Make sure VLC is installed on your system.");
        }
        #[allow(clippy::missing_transmute_annotations)]
        let debug_cb =
            unsafe { Some(mem::transmute(VLCInstance::log_callback_debug as *const ())) };
        #[allow(clippy::missing_transmute_annotations)]
        let info_cb = unsafe { Some(mem::transmute(VLCInstance::log_callback_info as *const ())) };
        #[allow(clippy::missing_transmute_annotations)]
        let warning_cb = unsafe {
            Some(mem::transmute(
                VLCInstance::log_callback_warning as *const (),
            ))
        };
        #[allow(clippy::missing_transmute_annotations)]
        let error_cb =
            unsafe { Some(mem::transmute(VLCInstance::log_callback_error as *const ())) };
        unsafe {
            match debug_level {
                0 => vlc::libvlc_log_set(instance, debug_cb, ptr::null_mut()),
                1 => vlc::libvlc_log_set(instance, info_cb, ptr::null_mut()),
                2 => vlc::libvlc_log_set(instance, warning_cb, ptr::null_mut()),
                3 => vlc::libvlc_log_set(instance, error_cb, ptr::null_mut()),
                _ => {}
            }
        }
        let instance = Some(instance);
        Self { instance, base }
    }

    fn on_notification(&mut self, what: ObjectNotification) {
        if what == ObjectNotification::PREDELETE && self.instance.is_some() {
            unsafe {
                vlc::libvlc_release(self.instance.unwrap());
                self.instance = None;
            }
        }
    }
}

#[godot_api]
impl VLCInstance {
    pub fn get_vlc_instance(&self) -> *mut vlc::libvlc_instance_t {
        self.instance.unwrap()
    }

    unsafe extern "C" fn log_callback_debug(
        _data: *mut c_void,
        level: c_int,
        _ctx: *const vlc::libvlc_log_t,
        fmt: *const c_char,
        args: *mut c_void,
    ) {
        let s: String = printf(fmt, args);
        match level as vlc::libvlc_log_level {
            vlc::libvlc_log_level_LIBVLC_NOTICE => {
                godot_print!("LibVLC: {}", s);
            }
            vlc::libvlc_log_level_LIBVLC_WARNING => {
                godot_warn!("LibVLC: {}", s);
            }
            vlc::libvlc_log_level_LIBVLC_ERROR => {
                godot_error!("LibVLC: {}", s);
            }
            vlc::libvlc_log_level_LIBVLC_DEBUG => {
                godot_print!("LibVLC: [DEBUG] {}", s);
            }
            _ => {}
        }
    }

    unsafe extern "C" fn log_callback_info(
        _data: *mut c_void,
        level: c_int,
        _ctx: *const vlc::libvlc_log_t,
        fmt: *const c_char,
        args: *mut c_void,
    ) {
        let s: String = printf(fmt, args);
        match level as vlc::libvlc_log_level {
            vlc::libvlc_log_level_LIBVLC_NOTICE => {
                godot_print!("LibVLC: {}", s);
            }
            vlc::libvlc_log_level_LIBVLC_WARNING => {
                godot_warn!("LibVLC: {}", s);
            }
            vlc::libvlc_log_level_LIBVLC_ERROR => {
                godot_error!("LibVLC: {}", s);
            }
            _ => {}
        }
    }

    unsafe extern "C" fn log_callback_warning(
        _data: *mut c_void,
        level: c_int,
        _ctx: *const vlc::libvlc_log_t,
        fmt: *const c_char,
        args: *mut c_void,
    ) {
        let s: String = printf(fmt, args);
        match level as vlc::libvlc_log_level {
            vlc::libvlc_log_level_LIBVLC_WARNING => {
                godot_warn!("LibVLC: {}", s);
            }
            vlc::libvlc_log_level_LIBVLC_ERROR => {
                godot_error!("LibVLC: {}", s);
            }
            _ => {}
        }
    }

    unsafe extern "C" fn log_callback_error(
        _data: *mut c_void,
        level: c_int,
        _ctx: *const vlc::libvlc_log_t,
        fmt: *const c_char,
        args: *mut c_void,
    ) {
        let s: String = printf(fmt, args);
        if level as vlc::libvlc_log_level == vlc::libvlc_log_level_LIBVLC_ERROR {
            godot_error!("LibVLC: {}", s);
        }
    }
}
