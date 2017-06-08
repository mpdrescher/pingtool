extern crate oping;
extern crate term;
extern crate tabwriter;

use term::color;
use tabwriter::TabWriter;
use oping::{Ping, PingResult};

use std::thread;
use std::env;
use std::collections::VecDeque;
use std::io::Write;

const BUFFER_SIZE: usize = 120; //keeping track of the last two minutes
const TIMEOUT: f64 = 5.0;

const COLOR_1: u16 = color::CYAN;
const COLOR_2: u16 = color::YELLOW;

const VERSION: &'static str = "v.0.1";
const REPOSITORY: &'static str = "https://github.com/mpdrescher/pingtool";

struct Host {
    name: String,
    address: Option<String>,
    history: VecDeque<f64>,
    current: f64,
    min: f64,
    max: f64,
    avg: f64,
}

impl Host {
    pub fn new(name: String) -> Host {
        Host {
            name: name,
            address: None,
            history: VecDeque::with_capacity(BUFFER_SIZE),
            current: 0.0,
            min: 0.0,
            max: 0.0,
            avg: 0.0,
        }
    }

    pub fn set_address(&mut self, address: String) {
        self.address = Some(address);
    }
    
    pub fn name(&self) -> &String {
        &self.name
    }

    pub fn push_value(&mut self, val: f64) {
        if self.history.len() == BUFFER_SIZE {
            let _ = self.history.pop_back();
        }
        self.history.push_front(val);
        self.current = val;
    }

    pub fn address(&self) -> &Option<String> {
        &self.address
    }
    
    //calculate display values -> min, max, avg
    pub fn calculate(&mut self) {
        let mut sum = 0.0;
        let mut min = TIMEOUT*1000.0;
        let mut max = 0.0;
        for elem in &self.history {
            if elem < &min {
                min = elem.clone();
            }
            if elem > &max {
                max = elem.clone();
            }
            sum += elem.clone();
        }
        let avg = sum / self.history.len() as f64;
        self.min = min;
        self.max = max;
        self.avg = avg;
    }

    pub fn current(&self) -> f64 {
        self.current
    }
    
    pub fn min(&self) -> f64 {
        self.min
    }

    pub fn max(&self) -> f64 {
        self.max
    }

    pub fn avg(&self) -> f64 {
        self.avg
    }
}

fn main() {
    println!("");
    
    let mut args = env::args().collect::<Vec<String>>();
    if args.len() <= 1 {
        println!("USAGE: pingtool <Host>...");
        println!();
        println!("pingtool {}", VERSION);
        println!("{}", REPOSITORY);
        println!();
        println!("dependencies:");
        println!("    oping       <rust crate, library>");
        println!("    term        <rust crate>");
        println!("    tabwriter   <rust crate>");
        println!();
        return;

    }
    args.remove(0);

    let mut hosts = args.into_iter().map(|x| Host::new(x)).collect::<Vec<Host>>();
    let mut out = term::stdout().expect("could not get term stdout");
    
    loop {
        hosts = match ping(hosts) {
            Ok(v) => v,
            Err(e) => {
                println!("error: {}", e);
                return;
            }
        };

        //NOTICE: I'm using ~,^, and * as escape characters to control terminal colors
        //~ -> cyan, ^ -> reset, * -> yellow
        let mut tabw = TabWriter::new(vec!());
        writeln!(tabw, "NAME\tADDRESS\tAVG\tMIN\tMAX\tCURRENT");
        writeln!(tabw, "\t\t\t\t\t");
        for host in &hosts {           
            let address = match host.address {
                Some(ref v) => v,
                None => "<unknown>"
            };
            //i need to pad with spaces, because tabwriter will see the escape chars i use for coloring as actual chars,
            //and consider them when calculating the column width
            //in the end it would probably have been better to write this myself, for better performance and less hack-ness
            writeln!(tabw, "~{}^\t  {}\t  ~{}^\t    {}\t    {}\t    *{}^",
                     host.name(),
                     address,
                     host.avg() as isize,
                     host.min() as isize,
                     host.max() as isize,
                     host.current() as isize).expect("tabwriter error 2");
            
        }
        tabw.flush().expect("tabwriter error 3");
        let tabbed_string = String::from_utf8(tabw.into_inner().expect("tabwriter error 4")).expect("utf8 error");
        for ch in tabbed_string.chars() {
            match ch {
                '~' => {out.fg(COLOR_1);},
                '*' => {out.fg(COLOR_2);},
                '^' => {out.reset();},
                x => {write!(out, "{}", x);}
            }
        }
        println!("");
        
        out.carriage_return().expect("carriage return failed");
        for _ in 0..hosts.len()+3 {
            out.cursor_up().expect("cursor up failed");
        }
        
        thread::sleep_ms(1000);    
    }
}

fn ping(mut hosts: Vec<Host>) -> PingResult<Vec<Host>> {
    let mut ping = Ping::new();
    ping.set_timeout(5.0)?;
    for host in &hosts {
        match ping.add_host(host.name()) {
            Ok(_) => {},
            Err(e) => {
                println!("unknown hostname error: '{}'", host.name());
                return Err(e)
            }
        }
    }
    let responses = ping.send()?;
    for response in responses {
        let name = response.hostname;
        let address = response.address;
        let latency = response.latency_ms;
        let dropped = response.dropped == 1;
        for host in &mut hosts {
            if dropped {
                break;
            }
            if host.name() == &name {
                host.set_address(address);
                host.push_value(latency);
                host.calculate();
                break;
            }
        }
    }
    Ok(hosts)
}
