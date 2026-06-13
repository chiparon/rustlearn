use std::io::{self, Write};

// // 这个函数负责：
// // 1. 打印提示语
// // 2. 从终端读取用户输入的一行文本
// // 3. 去掉输入末尾的换行符
// // 4. 返回处理后的字符串
// fn prompt_user_input(prompt: &str) -> String {
//     // print! 不会自动换行，所以用户可以在提示语后面直接输入。
//     print!("{}", prompt);

//     // 强制把提示语立刻输出到终端。
//     // 如果不 flush，有些环境下程序可能会先等输入，提示语却还没显示出来。
//     // flush() 这个方法来自上面导入的 Write trait。
//     io::stdout().flush().expect("failed to flush stdout");

//     // 创建一个空的 String。
//     // mut 表示这个变量是可变的，因为 read_line 会把用户输入写进 input。
//     let mut input = String::new();

//     // stdin() 表示标准输入，也就是终端输入。
//     // read_line 会读取用户输入的一整行，并追加到 input 里面。
//     // &mut input 表示把 input 的“可变引用”传进去，让 read_line 可以修改它。
//     io::stdin()
//         .read_line(&mut input)
//         .expect("failed to read user input");

//     // read_line 会保留末尾的换行符，比如用户输入 apples，实际可能是 "apples\n"。
//     // trim() 去掉首尾空白和换行。
//     // to_string() 把 trim() 得到的 &str 转回拥有所有权的 String。
//     input.trim().to_string()
// }

// // 读取购物清单。
// // 一直让用户输入商品名，直到用户输入 done 为止。
// // 返回值 Vec<String> 表示“装着很多 String 的动态数组”。
// fn read_shopping_list() -> Vec<String> {
//     // 创建一个空 vector，用来保存购物清单。
//     // 这里没有显式写 Vec<String>，Rust 会根据后面 push 进去的是 String 自动推断类型。
//     let mut shopping_list = Vec::new();

//     // loop 是 Rust 的无限循环。
//     // 之后遇到 break 才会跳出循环。
//     loop {
//         let input = prompt_user_input("Enter an item to add to the list: ");

//         // 把输入转成小写再比较。
//         // 这样 done、Done、DONE 都可以结束输入。
//         if input.to_lowercase() == "done" {
//             break;
//         }

//         // 把这次输入的商品名放进购物清单。
//         shopping_list.push(input);
//     }

//     // Rust 里函数最后一行如果没有分号，就会作为返回值。
//     // 所以这里相当于 return shopping_list;
//     shopping_list
// }

// // 打印购物清单。
// // 参数类型是 &Vec<String>，表示“借用”这个 vector，而不是拿走它的所有权。
// fn print_shopping_list(shopping_list: &Vec<String>) {
//     println!("Remember to buy:");

//     // 这里遍历的是借来的购物清单。
//     // item 是每个商品的引用，商品本身不会被移出 vector。
//     for item in shopping_list {
//         println!("* {}", item);
//     }
// }

// fn main() {
//     // 调用函数读取完整购物清单。
//     let shopping_list = read_shopping_list();

//     // &shopping_list 表示把购物清单的引用传进去。
//     // main 仍然拥有 shopping_list 的所有权。
//     print_shopping_list(&shopping_list);
// }
fn prompt_user_input (prompt:&str)->String{
    print!("{}",prompt);
    io::stdout().flush().expect("failed to flush stdout");
    let mut s = String::new();
    io::stdin()
        .read_line(&mut s)
        .expect("Failed to read line");
    s.trim().to_string()

}
fn read_shopping_list()->Vec<String>{
    let mut shopping_list =Vec::new();
    loop {
        let input = prompt_user_input("Enter an item to add to the list: ");
        if input.to_lowercase()=="done"{
            break;
        }else{
            shopping_list.push(input);
        }
    }
    shopping_list
}
fn display_shopping_list(list:&Vec<String>){
    println!("Remember to buy:");
    for item in list{
        println!("* {}",item);
    }
}
fn main(){
    let list:Vec<String> = read_shopping_list();
    display_shopping_list(&list);
}