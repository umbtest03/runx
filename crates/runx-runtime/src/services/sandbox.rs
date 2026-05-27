use std::collections::BTreeMap;
use std::path::Path;

#[cfg(feature = "mcp")]
use runx_parser::SkillMcpServer;
use runx_parser::SkillSource;

use crate::RuntimeError;
use crate::sandbox::SandboxPlan;
#[cfg(feature = "mcp")]
use crate::sandbox::prepare_mcp_process_sandbox;
#[cfg(feature = "cli-tool")]
use crate::sandbox::prepare_process_sandbox;

#[derive(Clone, Copy, Debug, Default)]
pub(crate) struct SandboxServices;

impl SandboxServices {
    #[cfg(feature = "cli-tool")]
    pub(crate) fn process_plan(
        self,
        source: &SkillSource,
        skill_directory: &Path,
        inputs: &runx_contracts::JsonObject,
        base_env: &BTreeMap<String, String>,
    ) -> Result<SandboxPlan, RuntimeError> {
        prepare_process_sandbox(source, skill_directory, inputs, base_env)
    }

    #[cfg(feature = "mcp")]
    pub(crate) fn mcp_process_plan(
        self,
        source: &SkillSource,
        server: &SkillMcpServer,
        skill_directory: &Path,
        base_env: &BTreeMap<String, String>,
    ) -> Result<SandboxPlan, RuntimeError> {
        prepare_mcp_process_sandbox(source, server, skill_directory, base_env)
    }
}
