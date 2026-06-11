/*fn $name(a:dtype,b:dtype)->dtype{
    a+b
}
*/
//no return, last expression is what fn return, no ; aswell.
fn add(a:i32,b:i32)->i32{
    a+b
}
fn divide(a: f64, b: f64) -> (f64, String) {
    if b == 0.0 {
        return (0.0, String::from("错误：除数不能为零"));
    }
    (a / b, String::from("OK")) // 返回元组
}
fn largest<T: PartialOrd>(list: &[T]) -> &T {//T:PartialOrd : trait bound, allow compare </>.
    let mut largest = &list[0];
    for item in list {
        if item > largest {
            largest = item;
        }
    }
    largest
}

//struct
struct User{
    name:String,
    email:String,
    active:bool,
    login_count:u64,
}
//impl : User realize method
impl User{
    fn new(name: String , email: String)->User{
        User{
            name,
            email,
            active:true,
            login_count:0,//DEFAULT VALUE
        }
    }
    fn summary(&self)->String {//&self for immutable quotation of case.
        format!("{} ({}) - LOGIN COUNT={}",self.name, self.email, self.login_count)
    }
    fn login(&mut self){
        self.login_count+=1;
    }
}
fn main(){
    let mut user=User::new(//:: still stands for method use
        String::from("chiparon"),
        String::from("chiparon@mail.sdu.edu.cn"),

    );
    println!("{}",user.summary());
    user.login();
    user.login();
    println!("{}",user.summary());
    println!("login count is {}",user.login_count);
}