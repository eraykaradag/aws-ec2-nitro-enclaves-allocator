mod resources;
mod error;
mod configuration;


pub use error::Error as Error;


fn main()  -> Result<(), Box<dyn std::error::Error>> {
    
    
    let pool = configuration::get_resource_pool()?;
    
    let _ = resources::Allocation::find_n_allocate(pool);
    
    Ok(())
}
