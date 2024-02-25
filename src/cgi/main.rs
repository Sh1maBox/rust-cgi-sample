use std::io::Read;

fn main() {
    println!("sh1ma");
    let mut buffer = [0; 1024];
    let n = std::io::stdin().read(buffer.as_mut()).unwrap();
    println!("moratta: {}", String::from_utf8_lossy(&buffer[..n]));
}
