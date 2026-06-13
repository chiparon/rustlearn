variable is default immutable in rust.
MUST note mutable var.
basic var type in rust:
signed: i8,i16,i32,i64,i128
unsigned:u{},2^i
float: f32,f64
bool:bool
character:char
tuple: (T1,T2,T3..)

被切片引用的字符串禁止更改其值：
```rust
fn main() {
    let mut s = String::from("runoob");
    let slice = &s[0..3];
    s.push_str("yes!"); // 错误
    println!("slice = {}", slice);
}
```
let s = "qasd";-> s: &str

字段名称=现存变量，省略
```rust

let domain = String::from("www.runoob.com");
let name = String::from("RUNOOB");
let runoob = Site {
    domain,  // 等同于 domain : domain,
    name,    // 等同于 name : name,
    nation: String::from("China"),
    traffic: 2013
};
```
新建一个部分值更改的结构体：
```rust
let site = Site {
    domain: String::from("www.runoob.com"),
    name: String::from("RUNOOB"),
    ..runoob//..
};
```
TUPLE STRUCT

```RUST
struct Color(u8, u8, u8);
struct Point(f64, f64);

let black = Color(0, 0, 0);
let origin = Point(0.0, 0.0);
println!("black = ({}, {}, {})", black.0, black.1, black.2);
println!("origin = ({}, {})", origin.0, origin.1);
```
结构体必须掌握字段值所有权
输出结构体
```RU
#[derive(Debug)]
//导入调试库 #[derive(Debug)] ，之后在 println 和 print 宏中就可以用 {:?} 占位符输出一整个结构体
struct Rectangle {
    width: u32,
    height: u32,
}

fn main() {
    let rect1 = Rectangle { width: 30, height: 50 };

    println!("rect1 is {:?}", rect1);
}
```

结构体方法
结构体方法的第一个参数必须是 &self，不需声明类型，因为 self 不是一种风格而是关键字。
```ru
struct Rectangle {
    width: u32,
    height: u32,
}
    
impl Rectangle {
    fn area(&self) -> u32 {
        self.width * self.height
    }
}

fn main() {
    let rect1 = Rectangle { width: 30, height: 50 };
    println!("rect1's area is {}", rect1.area());
}
```
function of struct, no use &self

```ru
#[derive(Debug)]
struct Rectangle {
    width: u32,
    height: u32,
}

impl Rectangle {
    fn create(width: u32, height: u32) -> Rectangle {
        Rectangle { width, height }
    }
}

fn main() {
    let rect = Rectangle::create(30, 50);
    println!("{:?}", rect);
}
```

## enum

并不能像访问结构体字段一样访问枚举类绑定的属性。访问的方法在 match 语法中。
match:
```ru
fn main() {
    enum Book {
        Papery {index: u32},
        Electronic {url: String},
    }
    
    let book = Book::Papery{index: 1001};
    let ebook = Book::Electronic{url: String::from("url...")};
    
    match book {
        Book::Papery { index } => {
            println!("Papery book {}", index);
        },
        Book::Electronic { url } => {
            println!("E-book {}", url);
        }
    }
}
```
match 块也可以当作函数表达式来对待，它也是可以有返回值的：

match 枚举类实例 {
    分类1 => 返回值表达式,
    分类2 => 返回值表达式,
    ...
}
但是所有返回值表达式的类型必须一样


### option: enum to realize null

```ru
enum Option<T> {
    Some(T),
    None,
}
fn main() {
    let opt = Some("Hello");
    let opt: Option<&str> = Option::None;
    match opt {
        Some(something) => {
            println!("{}", something);
        },
        None => {
            println!("opt is nothing");
        }
    }
}
```
