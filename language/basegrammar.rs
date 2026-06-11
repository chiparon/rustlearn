fn main(){
    let name = "NAME";// name is immutable var
    //name ="other" is invalid
    println!("name:{}",name);

    let mut score=0;//score is mutable var using mut
    println!("initial score:{}",score);
    score=100;
    println!("updated score:{}",score);
    
    //shadowing: let reclaim var of same name
    let x=5;
    let x=x+1;//let make x be a new var, value=6;
    let x=x*2;//let make x a new var, value=12;
    println!("final x value:{}",x);

    // constant shall claim its type.
    const MAX_POINTS:u32=100_000;

    //claim a tuple:
    let t:(i32,f64)=(10,3.14);//set length, varied type
    let info = ("name",19,true);//likely of py.

    //claim a array:
    let a:[i32;3]=[1,2,3];//use [T;N] nott; on stack

    //claim a slice?:
    let a = [1, 2, 3, 4, 5];
    let s: &[i32] = &a[1..4];  // s = [2, 3, 4]
    //it is a slice, borrowed not copied.
    

    //claim a dynamic array
    let v:Vec<i32> = vec![1,2,3];//varied length, on heap

    //claim a string
    let s = String::from("sttt");//String : mutable; &str: borrowed slice

    //control flow

    let score=70;
    //if , can be return value
    let grade= if score>=90{
        "A"
    }else if score>=60{
        "B"
    }else {
        "C"
    };
    println!("Grade:{}",grade);

    //loop: can be return value
    let mut count=0;
    let result =loop{//infinite loop, getout with break.
        count +=1;
        if count==5{
            break count *2;//as return value of
        }
    };
    println!("loop returns:{}",result);

    //while : condition loop
    let mut n=3;
    while n>0{
        println!("倒计时:{}",n);
        n-=1;
    }

    //for loop
    for num in 1..=3{// for $var in $StartIndex .. (=)$EndIndex
        println!("number is {}",num);
    }

    
}