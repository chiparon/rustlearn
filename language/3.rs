fn main() {
    // match 与数字匹配
    let score = 85;
    match score {
        90..=100 => println!("RUNOOB 评级: A"), // 范围匹配
        60..=89  => println!("RUNOOB 评级: B"),
        0..=59   => println!("RUNOOB 评级: C"),
        _ => println!("无效分数"),                // 通配符，匹配所有剩余情况
    }

    // match 解构元组
    let pair = (3, 7);
    match pair {
        (0, y) => println!("第一个是 0，第二个是 {}", y),
        (x, 0) => println!("第一个是 {}，第二个是 0", x),
        (x, y) => println!("两个值: {} 和 {}", x, y),
    }
}