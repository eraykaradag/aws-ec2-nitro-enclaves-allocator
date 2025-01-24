#[derive(thiserror::Error, Debug)]
pub enum Error
{
	#[error(transparent)]
	ParseInt(#[from] std::num::ParseIntError),
	#[error(transparent)]
	TryFromInt(#[from] std::num::TryFromIntError),
	#[error(transparent)]
	Allocation(#[from] super::resources::Error),
	#[error("Nitro CLI error: {0}")]
	NitroCli(String),
	#[error("Config file cannot include cpu_count and cpu_pool tag at the same time")]
	BothOptionsForCpu,
	#[error("Invalid config file: This might happened due to old config file or config file corruption. See release notes :")]
	ConfigFileCorruption,
}
