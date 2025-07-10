use std::{
    env,
    io::{Error, Write},
    path,
};












pub fn dir_creation(path_str: &str) -> Result<(), Error> {
    let path = path::Path::new(path_str);
    
    
    std::fs::create_dir_all(path)?;

    Ok(())
}













pub fn file_creation(path_str: &str, content: &str) -> Result<(), Error> {
    let path = path::Path::new(path_str);

    
    if let Some(parent_dir) = path.parent() {
        
        
        std::fs::create_dir_all(parent_dir)?;
    }

    
    
    let mut file = std::fs::File::create(path)?;

    
    file.write_all(content.as_bytes())?;

    Ok(())
}










pub fn is_exists(path_str: &str) -> bool {
    
    
    env::current_dir().unwrap().join(path_str).exists()
}










pub fn is_file_exists(path_str: &str) -> bool {
    
    env::current_dir().unwrap().join(path_str).is_file()
}










pub fn is_dir_exists(path_str: &str) -> bool {
    
    env::current_dir().unwrap().join(path_str).is_dir()
}
