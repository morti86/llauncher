use tracing::debug;

#[macro_export]
macro_rules! log_err {
    ($e:expr) => {
        if let Err(e) = $e {
            error!("Error: {}", $e);
        }
    };
    ($e:expr, $code:expr) => {
        if let Err(e) = $e {
            error!("Error[{}]: {}", $code, $e);
        }
    };
}

#[macro_export]
macro_rules! button_nf {
    ($text:expr) => {
        button( text($text).align_x(iced::Alignment::Center).font(Font::with_name("Symbols Nerd Font")))
            .width(34.0)
            
    };
}

#[macro_export]
macro_rules! button_nft {
    ($text:expr, $tp:expr, $msg:ident) => {

        tooltip(
            button( text($text).align_x(iced::Alignment::Center).font(Font::with_name("Symbols Nerd Font")))
                .width(34.0)
                .on_press(Message::$msg), 
            container(text($tp)).padding(5.0).style(container::rounded_box) , tooltip::Position::FollowCursor
            )
    }
}

#[macro_export]
macro_rules! make_enum {
    ($name:ident, [$op1:ident]) => {
        #[derive(Clone, Debug, Copy, PartialEq, serde::Serialize, serde::Deserialize)]
        pub enum $name {
            $op1,
        }

        impl Default for $name {
            fn default() -> Self {
                $name::$op1
            }
        }

        impl $name {
            // Fixed array with commas
            pub const ALL: &'static [Self] = &[$name::$op1];

            pub fn to_string(&self) -> String {
                match self {
                    $name::$op1 => stringify!($op1).to_string(),
                }
            }

            pub fn as_str(&self) -> &str {
                match self {
                    $name::$op1 => stringify!($op1),
                }
            }
        }

        impl From<String> for $name {
            fn from(s: String) -> Self {
                let s = s.as_str();
                match s {
                    _ => $name::$op1,
                }

            }
        }

        impl std::fmt::Display for $name {
            fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
                f.write_str(self.to_string().as_str())
            }
        }


    };

    ($name:ident, [$op1:ident, $($opt:ident),*]) => {
        #[derive(Clone, Debug, Copy, PartialEq, serde::Serialize, serde::Deserialize)]
        pub enum $name {
            $op1,
            $(
                $opt,
            )*
        }

        impl Default for $name {
            fn default() -> Self {
                $name::$op1
            }
        }

        impl $name {
            // Fixed array with commas
            pub const ALL: &'static [Self] = &[$name::$op1, $($name::$opt),+];

            pub fn to_string(&self) -> String {
                match self {
                    $name::$op1 => stringify!($op1).to_string(),
                    $(
                        $name::$opt => stringify!($opt).to_string(),
                    )*
                }
            }

            pub fn as_str(&self) -> &str {
                match self {
                    $name::$op1 => stringify!($op1),
                    $(
                        $name::$opt => stringify!($opt),
                    )*
                }
            }
        }

        impl From<String> for $name {
            fn from(s: String) -> Self {
                let s = s.as_str();
                match s {
                    stringify!($op1) => $name::$op1,
                    $(
                        stringify!($opt) => $name::$opt,
                    )*
                        _ => $name::$op1,
                }

            }
        }

        impl std::fmt::Display for $name {
            fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
                f.write_str(self.to_string().as_str())
            }
        }
    };
}

#[macro_export]
macro_rules! modal {
    ($err:expr) => {
        iced::Task::done(Message::ShowModal($err.to_string()))
    };
}

pub fn str_to_op(s: String) -> Option<String> {
    if s.is_empty() {
        None
    } else {
        Some(s)
    }
}

pub const APP_NAME: &str = "llauncher";
pub const CONFIG_FILE: &str = "llauncher.toml";

pub fn find_config_path() -> Option<std::path::PathBuf> {
    // 1. Check User Config Directory
    if let Some(mut config_dir) = dirs::config_dir() {
        config_dir.push(APP_NAME);
        config_dir.push(CONFIG_FILE);
        
        if config_dir.exists() {
            debug!("Found user config at {:?}", config_dir);
            return Some(config_dir);
        }
    }

    // 2. Check Executable Directory
    if let Ok(exe_path) = env::current_exe()
        && let Some(exe_dir) = exe_path.parent() {
            let mut local_config = exe_dir.to_path_buf();
            local_config.push(CONFIG_FILE);
            debug!("Checking local: {:?}", local_config);
            if local_config.exists() {
                debug!("Local config");
                return Some(local_config);
            }
    }

    None
}
