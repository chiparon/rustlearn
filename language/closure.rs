fn main() {
    let name = String::from("Rust");
    //reference catch
    let print_name = || {
        println!("{}", name);
    };

    print_name();

    println!("{}", name);
    println!("{}", name);
    //mutable reference catch
    let mut count = 0;

    let mut add_one = || {
        count += 1;
    };

    add_one();
    add_one();

    println!("{}", count);
    
}