use crate::driver::config::OptLevel;

/// Pass options controlling MIR optimization passes.
#[derive(Debug, Clone)]
pub struct PassOptions {
    pub opt_level: OptLevel,
    pub enable_const_fold: bool,
    pub enable_peephole: bool,
    pub enable_opcode_select: bool,
}

impl PassOptions {
    /// Create pass options from environment variable or default configuration.
    pub fn from_env() -> Self {
        Self::from_env_or_config(OptLevel::O2)
    }

    /// Create pass options from a specific OptLevel configuration with environment variable overrides.
    pub fn from_config(config_opt: OptLevel) -> Self {
        Self::from_env_or_config(config_opt)
    }

    /// Create pass options based on OptLevel and environment variable overrides.
    pub fn from_env_or_config(config_opt: OptLevel) -> Self {
        let opt_level = if let Ok(val) = std::env::var("MAMBA_OPT_LEVEL") {
            match val.trim() {
                "0" => OptLevel::O0,
                "1" => OptLevel::O1,
                "2" => OptLevel::O2,
                "3" => OptLevel::O3,
                _ => config_opt,
            }
        } else {
            config_opt
        };

        let is_enabled = !matches!(opt_level, OptLevel::O0);
        let mut enable_const_fold = is_enabled;
        let mut enable_peephole = is_enabled;
        let mut enable_opcode_select = is_enabled;

        if let Ok(passes) = std::env::var("MAMBA_OPT_PASSES") {
            for pass in passes.split(',') {
                let pass = pass.trim();
                match pass {
                    "+const_fold" | "const_fold" => enable_const_fold = true,
                    "+peephole" | "peephole" => enable_peephole = true,
                    "+opcode_select" | "opcode_select" => enable_opcode_select = true,
                    _ => {}
                }
            }
        }

        if let Ok(passes) = std::env::var("MAMBA_DISABLE_PASSES") {
            for pass in passes.split(',') {
                let pass = pass.trim();
                match pass {
                    "-const_fold" | "const_fold" => enable_const_fold = false,
                    "-peephole" | "peephole" => enable_peephole = false,
                    "-opcode_select" | "opcode_select" => enable_opcode_select = false,
                    _ => {}
                }
            }
        }

        Self {
            opt_level,
            enable_const_fold,
            enable_peephole,
            enable_opcode_select,
        }
    }

    /// Check whether a specific pass is enabled.
    pub fn is_pass_enabled(&self, pass_name: &str) -> bool {
        match pass_name {
            "const_fold" => self.enable_const_fold,
            "peephole" => self.enable_peephole,
            "opcode_select" => self.enable_opcode_select,
            _ => self.opt_level > OptLevel::O0,
        }
    }
}

impl Default for PassOptions {
    fn default() -> Self {
        Self::from_env_or_config(OptLevel::O2)
    }
}
