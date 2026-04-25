use bcrypt;

fn main() {
    let password = "bendy2024";
    let hash = bcrypt::hash(password, bcrypt::DEFAULT_COST).unwrap();
    println!("{}", hash);
}
