#![no_std]
#![no_main]
#![feature(asm)]

mod bootstrap;

#[macro_use]
mod uart;
use uart::*;
mod timer;

use core::panic::PanicInfo;

fn wait(n: i32) {
    let mut v: i32 = 0;
    for i in 1..n {
        unsafe { core::ptr::write_volatile(&mut v, i) };
    }
}

#[panic_handler]
fn panic(_panic: &PanicInfo<'_>) -> ! {
    println!("\r\n!!!panic!!!");
    loop {}
}

fn debug_print(str: &str) {
    println!("{}", str);
}

mod memdump;

// use kernel::irc::pl390;

use pudding_kernel as kernel;
use kernel::*;

static mut STACK_INT: [u8; 4096] = [0; 4096];

static mut STACK0: [u8; 4096] = [0; 4096];
static mut STACK1: [u8; 4096] = [0; 4096];
static mut TASK0: Task = Task::new();
static mut TASK1: Task = Task::new();

static mut SEM0: Semaphore = Semaphore::new(0, Order::Fifo);


// main
#[no_mangle]
pub unsafe extern "C" fn main() -> ! {
    //  uart_write(0x23);
    wait(10000);
    println!("Hello world");

    /*
    println!("---- ICC ----");
    memdump::memdump(0xf9001000, 32);
    println!("---- IDC ----");
    memdump::memdump(0xf9000000, 32);
    println!("-------------");
    */

    println!("kernel start");

    kernel::set_debug_print(Some(debug_print));

    kernel::initialize();
    kernel::interrupt::initialize(&mut STACK_INT);

    kernel::irc::pl390::initialize(0xf9001000, 0xf9000000);
    let pl390 = pudding_kernel::irc::pl390::take();

    let targetcpu: u8 = 0x01;
    pl390.icd_disable();

    // set TTC0-1
    pl390.icd_set_target(74, targetcpu);

    // PL
    for i in 0..8 {
        pl390.icd_set_target(121 + i, targetcpu);
        pl390.icd_set_config(121 + i, 0x01); // 0x01: level, 0x03: edge
    }
    for i in 0..8 {
        pl390.icd_set_target(136 + i, targetcpu);
        pl390.icd_set_config(136 + i, 0x01); // 0x01: level, 0x03: edge
    }

    pl390.icd_enable();

    timer::timer_initialize(timer_int_handler);
    
    wait(100);
    //      println!("timer:{}", timer::timer_get_counter_value());

    TASK0.create(0, task0, 0, &mut STACK0);
    TASK1.create(1, task1, 1, &mut STACK1);
    TASK0.activate();
    TASK1.activate();
    
    println!("Idle loop");
    kernel::idle_loop();

    /*
    loop {
        //        kernel::cpu::cpu_unlock();
        println!(
            "timer:{} [s]",
            timer::timer_get_counter_value() as f32 / 100000000.0
        );
        //        println!("state:{}", system::is_interrupt_state());
        wait(1000000);
        loop {}
    }
    */
}

fn task0(_exinf: isize) {
    println!("Task0:start");
    unsafe {
        for _ in 0..3 {
            println!("Task0:sleep_strat");
            kernel::sleep(1000);
            println!("Task0:sleep_end");
        }

        println!("Task0:signal to semaphore");
        SEM0.signal();

    }
    println!("Task0:end");
    //    println!("state:{}", system::is_interrupt_state());
}

fn task1(_exinf: isize) {
    println!("Task1:start");
    unsafe {
        println!("Task1: wait semaphore");
        SEM0.wait();
    }
    //    println!("state:{}", system::is_interrupt_state());
    println!("Task1:end");
}

// static mut TIMER_COUNTER: u32 = 0;


// タイマ割込みハンドラ
fn timer_int_handler() {
    //  割込み要因クリア
    timer::timer_clear_interrupt();
    
    // カーネルにタイムティック供給
    kernel::supply_time_tick(1);
        
    /*
    unsafe{
        TIMER_COUNTER = TIMER_COUNTER.wrapping_add(1);
        if TIMER_COUNTER % 1000 == 0 {
            //            println!("timer irq:{}", system::is_interrupt_state());
            println!("timer irq");
            TASK0.activate();
        }
    }
    */
}

