//! Sysfs-based enclave resource allocation
pub type CpuSet = std::collections::BTreeSet::<usize>;
pub type Pages = std::collections::HashMap<usize, usize>;

mod cpu;
mod huge_pages;
use crate::configuration::ResourcePool;

#[derive(thiserror::Error, Debug)]
pub enum Error
{
	#[error(transparent)]
	Cpu(#[from] cpu::Error),
	#[error(transparent)]
	HugePage(#[from] huge_pages::Error),
	#[error("failed to find suitable combination of CPUs and memory")]
	Allocation,
}

pub struct Allocation
{
	// Both allocations implement Drop
	cpu_set_allocation: cpu::Allocation,
	_huge_pages_allocation: huge_pages::Allocation,
}

impl Allocation
{
	pub fn new(cpu_count: usize, memory_mib: usize) -> Result<Self, Error>
	{
		// Find NUMA nodes with a suitable CPU set
		for (numa_node, cpu_set) in cpu::find_suitable_cpu_sets(cpu_count)?.into_iter()
		{
			// Try to allocate the memory on the NUMA node ...
			let huge_pages_allocation =
				match huge_pages::Allocation::new(numa_node, memory_mib)
				{
					Ok(allocation) => allocation,
					Err(huge_pages::Error::InsufficientMemory) => continue,
					Err(error) => return Err(error.into()),
				};

			// ... if successful, also allocate the CPU set
			let cpu_set_allocation = cpu::Allocation::new(cpu_set)?;

			return Ok(Self
				{
					cpu_set_allocation,
					_huge_pages_allocation: huge_pages_allocation,
				});
		}

		Err(Error::Allocation)
	}
	pub fn find_n_allocate(pool: Vec<ResourcePool>) -> Result<(), Error>
	{
		let total_cpu_count = pool.iter().map(|p| p.cpu_count).sum();
		// Find NUMA nodes with a suitable CPU set	
		'outer: for (numa_node, cpu_set) in cpu::find_suitable_cpu_sets(total_cpu_count)?.into_iter()
		{
			// Try to allocate the memory on the NUMA node ...
			let mut allocated_pages: Vec<huge_pages::Allocation> = Vec::new();
			for enclave in &pool {
				let huge_pages_allocation = 
					match huge_pages::Allocation::new(numa_node, enclave.memory_mib)
					{
						Ok(allocation) => allocation,
						Err(huge_pages::Error::InsufficientMemory) => {
							//release everything
							for delete in &allocated_pages {
								delete.release_resources();
							}
							continue 'outer;
						}
						Err(error) => return Err(error.into()),
					};
				allocated_pages.push(huge_pages_allocation);
			}
			// ... if successful, also allocate the CPU set
			let cpu_set_allocation = cpu::Allocation::new(cpu_set)?;
		}
		Ok(())
	}
	pub fn cpu_count(&self) -> usize
	{
		self.cpu_set_allocation.cpu_count()
	}
}