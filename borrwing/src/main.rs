fn main() {
    let y: &i32;
    {
        let x = 5;
        y = &x;
        println!("{}", y);
    }
    
    // This println belove is not possible as `y` is only valid for the scope where `x` exists. So, this causes error:
    // error: `x` does not live long enough
    // y = &x;
    //println!("{}", y);
}
