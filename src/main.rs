extern crate rand;

mod chip8;

#[macro_use] extern crate native_windows_gui as nwg;

use chip8::*;

use CanvasId::*;
use nwg::{Event, EventArgs, Ui,fatal_message,dispatch_events,Timer};
use nwg::constants as nwgc;

use std::sync::mpsc::channel;
use std::sync::mpsc::Sender;
use std::sync::mpsc::Receiver;

use std::time::Duration;
use std::thread;
use std::env;

#[derive(Debug,Clone,Hash)]
pub enum CanvasId {
    MainWindow,
    Canvas,
    Paint,
    KeyDown,
    KeyUp,
    TimeGfx,
    KeyEventTx,
    GfxRx,
    SolidBrush(u8),
}



nwg_template!(
    head: setup_ui<CanvasId>,
    controls: [
        (MainWindow, nwg_window!( title="Chip8"; size=(640,320); resizable=false)),
        (Canvas, nwg_canvas!(parent=MainWindow; size=(640,320))),
        (TimeGfx, nwg_timer!(interval=1))
    ];

    events:[

        (TimeGfx, TimeGfx, Event::Tick, |app,_,_,_| {
            app.trigger(&Canvas,Event::Paint,EventArgs::None);
        }),


        (Canvas, Paint, Event::Paint, | app,_,_,_| {         
            let mut canvas =nwg_get_mut!(app; (Canvas,nwg::Canvas<CanvasId>));
            let mut timer = nwg_get_mut!(app; (TimeGfx,Timer));

            let gfx_rx = nwg_get_mut!(app;(GfxRx, Receiver<[u8;64*32]>));
            
            if !timer.running(){
                timer.start();
            }

            let mut renderer =canvas.renderer().unwrap();

            renderer.clear(0.3,0.3,0.6,1.0);


            let gfx = match gfx_rx.recv(){
                Ok(gfx) => gfx,
                Err(err) => panic!(err),
            };

            for row in 0..32{
                for col in 0..64{
                    if gfx[row*64+col] != 0 { 
           
                        let left = (col*10) as f32;
                        let top = (row*10) as f32;
                        let right = left + 10f32;
                        let bottom = top + 10f32;

                        let rect = nwgc::Rectangle{ left:left, right:right, top:top, bottom:bottom };
                        renderer.draw_rectangle(&SolidBrush(0),None,&rect,1.0).unwrap();
                    }
                }
            }


        }),

        (MainWindow, KeyDown,Event::KeyDown, |app,_,_,args| {
            match args{
                &EventArgs::Key(k) => {
                    let pressed = ((k as u8) as char).to_lowercase().next().unwrap();
                    let key_tx = nwg_get_mut!(app;(KeyEventTx, Sender<char>));
                    key_tx.send(pressed);

                },
                _ => println!("not a key"),
            }
        
        }),

        (MainWindow, KeyUp,Event::KeyUp, |app,_,_,_| {
                    let key_tx = nwg_get_mut!(app;(KeyEventTx, Sender<char>));
                    key_tx.send('_');
        })

    ];
    resources:[];

    values: []

);

fn setup_canvas_resources(app: &Ui<CanvasId>){
    let mut canvas = nwg_get_mut!(app; (Canvas, nwg::Canvas<CanvasId>));

    let b1 = nwgc::SolidBrush{color:(1.0, 1.0, 1.0, 1.0)};
    let b2 = nwgc::SolidBrush{color:(0.0, 0.0, 0.0, 1.0)};
    
    canvas.create_solid_brush(&SolidBrush(0), &b1).expect("Failed to create brush 1");
    canvas.create_solid_brush(&SolidBrush(1), &b2).expect("Failed to create brush 22");
}



fn main() {



    let app: Ui<CanvasId>;

    match Ui::new(){
        Ok(_app) => {app = _app},
        Err(e) => fatal_message("Fatal Error", &format!("{:?}",e)),
    };

    let (key_tx,key_rx): (Sender<char>, Receiver<char>) = channel();
    let (gfx_tx,gfx_rx): (Sender<[u8;64*32]>, Receiver<[u8;64*32]>) = channel();

    app.pack_value(&KeyEventTx,key_tx);
    app.pack_value(&GfxRx, gfx_rx);

    let args: Vec<String> = env::args().collect();

    if args.len() != 2 {
        println!("specify the game file path");
        return;
    }else{
        println!("loading {}", &args[1]);
    }

    let path = &args[1];

    let mut chip8 = Chip8::new(path);

    thread::spawn(move || {
       
        //faster frame_rate leads to memory leak:
        //  rx in UI cant handle all the gfx updates
        //faster frame_rate leads to gfx lag: 
        //  rx in UI draws all the old frames before the changed ones
        let frame_rate = Duration::from_millis(1000/60);
        
        //slow down the loop to aprox. 1 Megahertz, otherwise keypresses are too fast.
        let clock_rate = Duration::new(0,1000);

        let mut now = std::time::Instant::now();
        loop{

            match key_rx.try_recv(){
                Ok(key) => {
                    chip8.update_keys(key);
                }
                Err(_) => {},
            }

            chip8.emulate_cycle();
            
            if now.elapsed() >= frame_rate {
                chip8.decrease_dt();
                gfx_tx.send(*chip8.get_gfx());
                now = std::time::Instant::now();
            }
        
            thread::sleep(clock_rate);
        }
    });

    if let Err(e) = setup_ui(&app) {
        fatal_message("Fatal Error", &format!("{:?}",e));
    }

    setup_canvas_resources(&app);
    dispatch_events();
}

