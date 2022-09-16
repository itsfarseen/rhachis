use rhachis::rand::Noise;

fn main() {
    for i in 0..5 {
        let i = i * 100 + 400000;
        println!("Seed: {i}");
        let noise = Noise::from_seed(i);
        for j in 0..7 {
            println!("{:032b} {:08x}", noise.get(j), noise.get(j));
        }
    }

    let noise = Noise::new();
    println!("New");
    for j in 0..7 {
        println!("{:032b} {:08x}", noise.get(j), noise.get(j));
    }
}
