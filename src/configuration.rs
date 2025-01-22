use serde::Deserialize;
use crate::resources;
use crate::error::Error;

//deserializing from allocator.yaml file
#[derive(Debug, PartialEq, Deserialize,Clone)]
pub struct ResourcePool{
    pub memory_mib: usize,
    pub cpu_count: Option<usize>,
    pub cpu_pool: Option<String>,
}
pub fn get_resource_pool()  -> Result<Vec<ResourcePool>, Box<dyn std::error::Error>> {
    //config file deserializing
    let f = std::fs::File::open("/etc/nitro_enclaves/allocator.yaml")?;
    let mut pool: Vec<ResourcePool> =  match serde_yaml::from_reader(f) {
        Ok(pool) => pool,
        Err(ExpectedArray) => {return Err(Box::new(Error::OldConfigFile));},
    };
    for enclave in &pool {
        if enclave.cpu_pool.is_some() && enclave.cpu_count.is_some() {
            return Err(Box::new(Error::BothOptionsForCpu));
        }
    }
    //pool.enclaves.sort_by_key(|p| p.memory_mib);
    Ok(pool)
}
pub fn get_cpu_pool() -> Result<Option<std::collections::BTreeSet::<usize>>, Box<dyn std::error::Error>> {
    let f = std::fs::read_to_string("/sys/module/nitro_enclaves/parameters/ne_cpus")?;
    if f.trim().is_empty() {
        return Ok(None);
    }
    let cpu_list = resources::cpu::parse_cpu_list(&f[..])?;
    Ok(Some(cpu_list))
}
//clears everything in a numa node.
pub fn crystal_clear() -> Result<(), Box<dyn std::error::Error>> {
    match get_cpu_pool()?{
		Some(cpu_list) => {
		//find numa by one of cpuids
		let numa = resources::cpu::get_numa_node_for_cpu(cpu_list.clone().into_iter().next().unwrap())?;
		//release everything
		let _ = resources::huge_pages::release_all_huge_pages(numa)?;
		let _ = resources::cpu::deallocate_cpu_set(&cpu_list);
		}
		None => {}  
  	};
    Ok(())
}