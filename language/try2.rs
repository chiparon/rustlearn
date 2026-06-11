fn main() {
    // s1 获得字符串的所有权（数据存储在堆上）
    let s1 = String::from("Hello, RUNOOB!");

    // 所有权从 s1 转移到 s2，此后 s1 不再有效
    let s2 = s1;

    // println!("{}", s1); // 编译错误！s1 已被 moved
    println!("{}", s2);    // 正确：s2 现在是所有者

    // 基本类型（存储在栈上）实现了 Copy trait，不会发生 move
    let x = 42;
    let y = x;  // x 仍然有效，因为 i32 实现了 Copy
    println!("x = {}, y = {}", x, y); // 都能使用

    // 不可变引用：可以同时存在多个
    let s3 = &s2; // 不可变借用
    let s4 = &s2; // 另一个不可变借用，允许
    println!("s3 = {}, s4 = {}", s3, s4);

    // 可变引用：同一时刻只能有一个
    let mut data = String::from("runoob");
    let r = &mut data; // 可变借用
    r.push_str("!");   // 通过可变引用修改原值
    println!("修改后: {}", r);
    // s3 和 s4 的作用域已结束，所以可以创建可变引用
    
}
