mod resources;
mod error;
mod configuration;


fn main()  -> Result<(), Box<dyn std::error::Error>> {
	let _ = configuration::crystal_clear();

	match configuration::get_resource_pool() {
    	Ok(pool) => {			
			let numa_node = match resources::Allocation::allocate_by_cpu_pools(pool.clone()){
				Ok(numa) => numa,
				Err(e) =>{
					eprintln!("Allocation failed: {}",e);
					return Err(Box::new(e));
				},//proper error messages				
			};
      		match resources::Allocation::find_n_allocate(pool,numa_node) {
				Ok(_) => {},
				Err(e) => {
					let _ = configuration::crystal_clear();
					eprintln!(" Allocation failed: {}",e);
					return Err(Box::new(e));
				}
			} //check if allocation successful or not, if not crystal clear
    	}
    	Err(e) => {
			eprintln!("Allocation failed: {}",e);
      		return Err(e);
    	}

  	};
  	Ok(())
}
