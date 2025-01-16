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
}
