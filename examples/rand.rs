use rhachis::rand::Noise;

fn main() {
    for i in 0..5 {
        let i = i * 100 + 400000;
        println!("Seed: {i}");
        let noise = Noise::from_seed(i);
        for j in 0..7 {
            let val = noise.get(j);
            println!("{val:032b} {val:08x}");
        }
    }

    let mut noise = Noise::new();
    println!("New: {}", noise.seed);
    for _ in 0..7 {
        println!("{:032b}", noise.next_range(4..8));
    }
}
