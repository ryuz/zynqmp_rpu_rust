

#[macro_use]
extern crate jelly_kernel;

use jelly_kernel as kernel;
use kernel::context::*;
use kernel::task::*;


static mut STACK0: [isize; 256] = [0; 256];
static mut TASK0: Task = task_default!();

static mut STACK1: [isize; 256] = [0; 256];
static mut TASK1: Task = task_default!();


fn main() {
    println!("Start");
    unsafe {
        
        kernel::initialize();
        TASK0.create(0, task0, 0, &mut STACK0);
        TASK1.create(0, task1, 0, &mut STACK1);

        TASK0.activate();
        TASK1.activate();
    }
    println!("End");
}


fn task0(_exinf: isize) {
    println!("Task0");
    let t = unsafe { &mut TASK1};
    t.activate();
    println!("Task0_end");
}

fn task1(_exinf: isize) {
    println!("Task1");
}

