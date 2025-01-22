//! Sysfs-based enclave resource allocation
pub mod cpu;
pub mod huge_pages;
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
	#[error("Config file cannot include cpus from different numa nodes")]
	NumaDifference,
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
	pub fn find_n_allocate(mut pool: Vec<ResourcePool>,target_numa: usize) -> Result<(), Error>
	{
		pool.retain(|p| p.cpu_count.is_some());
		let total_cpu_count = pool.iter().filter_map(|p| p.cpu_count).sum();
		if total_cpu_count == 0 {
			return Ok(());
		}
		// Find NUMA nodes with a suitable CPU set
		'outer: for (numa_node, cpu_set) in cpu::find_suitable_cpu_sets(total_cpu_count)?.into_iter()
		.filter(|(numa_node, _)| target_numa == usize::MAX || *numa_node == target_numa) //TO:DO create different function for this
		{
			let mut allocated_pages:Vec<huge_pages::Allocation> = Vec::with_capacity(pool.len());
			// Try to allocate the memory on the NUMA node ...
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
			match cpu::Allocation::new(cpu_set) {
				Ok(_) => {return Ok(())},
				Err(_) => {
					for delete in &allocated_pages {
						delete.release_resources();
					}
					return Err(Error::Cpu(cpu::Error::InsufficientCpuPool));
				},
			}
		}
		return Err(Error::Allocation);
	}
	pub fn allocate_by_cpu_pools(mut pools: Vec<ResourcePool>) -> Result<usize, Error>
	{
		pools.retain(|p| p.cpu_pool.is_some());
		if pools.len() == 0 {
			return Ok(usize::MAX);
		}
		let mut allocated_pages:Vec<huge_pages::Allocation> = Vec::with_capacity(pools.len());
		let mut final_cpu_list = cpu::CpuSet::new();
		let mut numa_node = usize::MAX;
		for pool in &pools {
			let cpu_list = cpu::parse_cpu_list(&pool.cpu_pool.clone().unwrap()[..])?;
			let numa = cpu::get_numa_node_for_cpu(cpu_list.clone().into_iter().next().unwrap())?;
			if numa_node != usize::MAX && numa_node != numa {//check all cpu ids
				return Err(Error::NumaDifference);
			}
			else {
				numa_node = numa;
				final_cpu_list.extend(cpu_list);
			}
		}
		for enclave in &pools {
			let huge_pages_allocation = 
				match huge_pages::Allocation::new(numa_node, enclave.memory_mib)
				{
					Ok(allocation) => allocation,
					Err(huge_pages::Error::InsufficientMemory) => {
						//release everything
						for delete in &allocated_pages {
							delete.release_resources();
						}
						return Err(Error::Allocation);
					}
					Err(error) => return Err(error.into()),
				};
			allocated_pages.push(huge_pages_allocation);
		}
		// ... if successful, also allocate the CPU set
		match cpu::Allocation::new(final_cpu_list){
			Ok(_) => { return Ok(numa_node);},
			Err(_) => {
				for delete in &allocated_pages {
					delete.release_resources();
				}
				return Err(Error::Cpu(cpu::Error::InsufficientCpuPool));
			},
		}
	}
	pub fn cpu_count(&self) -> usize
	{
		self.cpu_set_allocation.cpu_count()
	}
}