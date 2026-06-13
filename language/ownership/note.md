in rust data move is divided.
copy dtype: 
- integers
- bool
- float
- char
- tuples contains above ones
these data will be directly copied when you should 
move.

String::from data is actually a ptr pointing to heap.
So it is not a copy dtype, a new move will change 
its ownership, and the former one lose its effect.
How to solve?
&str : reference
.clone() : clone the data

### Ownership
```rust
fn main() {
    let s = String::from("hello");
    // s 被声明有效

    takes_ownership(s);
    // s 的值被当作参数传入函数
    // 所以可以当作 s 已经被移动，从这里开始已经无效

    let x = 5;
    // x 被声明有效

    makes_copy(x);
    // x 的值被当作参数传入函数
    // 但 x 是基本类型，依然有效
    // 在这里依然可以使用 x 却不能使用 s

} // 函数结束, x 无效, 然后是 s. 但 s 已被移动, 所以不用被释放


fn takes_ownership(some_string: String) { 
    // 一个 String 参数 some_string 传入，有效
    println!("{}", some_string);
} // 函数结束, 参数 some_string 在这里释放

fn makes_copy(some_integer: i32) { 
    // 一个 i32 参数 some_integer 传入，有效
    println!("{}", some_integer);
} // 函数结束, 参数 some_integer 是基本类型, 无需释放
```
as we can see, while moved into fn, the variable 
will be freed after fn.

```rust

fn main() {
    let s1 = gives_ownership();
    // gives_ownership 移动它的返回值到 s1

    let s2 = String::from("hello");
    // s2 被声明有效

    let s3 = takes_and_gives_back(s2);
    // s2 被当作参数移动, s3 获得返回值所有权
} // s3 无效被释放, s2 被移动, s1 无效被释放.

fn gives_ownership() -> String {
    let some_string = String::from("hello");
    // some_string 被声明有效

    return some_string;
    // some_string 被当作返回值移动出函数
}

fn takes_and_gives_back(a_string: String) -> String { 
    // a_string 被声明有效

    a_string  // a_string 被当作返回值移出函数
}
```
被当作函数返回值的变量所有权将会被移动出函数并返回到调用函数的地方，而不会直接被无效释放。

```rust
fn main() {
    let s1 = String::from("hello");
    let s2 = &s1;
    let s3 = s1;//ownership goto s3, s2 no longer borrowed.
    s2.push_str("oob");//s2 is borrowed ownership,no right to change the value.
    println!("{}", s2);//error
    let mut s1 = String::from("run");
    // s1 是可变的

    let s2 = &mut s1;
    // s2 是可变的引用

    s2.push_str("oob");
    println!("{}", s2);
    // mutable ref only one. like read write lock.
    let mut s = String::from("hello");

    let r1 = &mut s;
    let r2 = &mut s;

    println!("{}, {}", r1, r2);
}

```


### dangling reference

```rust
fn main() {
    let reference_to_nothing = dangle();
}

fn dangle() -> &String {
    let s = String::from("hello");

    &s
}

```
