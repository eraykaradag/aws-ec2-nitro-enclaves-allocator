use serde::Deserialize;


//deserializing from allocator.yaml file
#[derive(Debug, PartialEq, Deserialize)]
pub struct ResourcePool{
    pub memory_mib: usize,
    pub cpu_count: usize, //TO:DO make it backward compatible get the range of cpu ids
}
#[derive(Debug, PartialEq, Deserialize)]
struct Pools{
    enclaves: Vec<ResourcePool>,
}

pub fn get_resource_pool()  -> Result<Vec<ResourcePool>, Box<dyn std::error::Error>> {
    //config file deserializing
    let f = std::fs::File::open("/etc/nitro_enclaves/allocator.yaml")?;
    let mut pool: Vec<ResourcePool> =  serde_yaml::from_reader(f)?;
    //pool.enclaves.sort_by_key(|p| p.memory_mib);
    
    Ok(pool)
}