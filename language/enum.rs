enum IpAddrKind {
    V4,
    V6,
}

let four = IpAddrKind::V4;
let six = IpAddrKind::V6;
//match
enum Coin {
    Penny,
    Nickel,Dime,Quarter,
}
fn valuecent(coin : Coin)->u8{
    match coin{
        Coin::Penny=>1,Coin::Nickel=>5,
        Coin::Dime=>10,
        Coin::Quarter=>25,//=> links match value with matched output
    }
}
//error deal
// result
enum Result<T,E>{
    Ok(T),
    Err(E),
}

fn divide(a:i32,b:i32)->Result<i32,String>{
    if b==0{
        Err(String::from("division by zero"))
    }
    else{
        Ok(a/b)
    }
}
let r = divide(10, 2);
match r {
    Ok(v) => println!("结果是 {}", v),
    Err(e) => println!("出错: {}", e),
}
