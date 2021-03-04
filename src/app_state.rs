
extern crate crypto;
use rocket::http::RawStr;
use rand::Rng;
use std::convert::TryInto;
use std::sync::Mutex;
use openssl::sha::sha1;
#[derive(Default)]
pub struct SharedState {
    pub state: Mutex<AppState>
}

#[derive(Debug, Copy, Clone)]
pub struct AppState {
    pub repl_factor:u8,
    pub view:[IPAddress; 8],
    pub length:usize,
    pub ring:[VirtualNode; 512]
}

#[derive(Debug, Copy, Clone, Default)]
pub struct IPAddress {
    pub ip:[u8;4],
    pub port:u32,
}

#[derive(Debug, Copy, Clone, Default)]
pub struct VirtualNode {
    pub hash:[u8; 20],
    pub id:u8
}

impl IPAddress {
    pub fn to_string(self) -> String{
        return format!("{}.{}.{}.{}:{}", self.ip[0], self.ip[1], self.ip[2], self.ip[3], self.port);
    }
}

impl Default for AppState {
    fn default() -> Self {
        let repl_factor = std::env::var("REPL_FACTOR")
            .unwrap().parse::<u8>().unwrap();
        let view_env = std::env::var("VIEW").unwrap();
        let mut app_state = AppState {
            repl_factor: repl_factor,
            view: [IPAddress::default(); 8],
            length:0,
            ring:[VirtualNode::default(); 512]
        };
        let view_iter = view_env.split(',');
        for (i,address) in view_iter.enumerate() {
            app_state.build_ip(address.to_string(), i);
            let ip_address = app_state.view[i];
            app_state.build_ring(ip_address.to_string(), i);
        }
        app_state.ring.sort_by(|a, b| a.hash.cmp(&b.hash));
        app_state
    }
}


impl AppState {
    pub fn build_ip(&mut self, address:String, i:usize) {
        if address.is_empty() {
            return;
        }
        let split = address.split(':').collect::<Vec<&str>>();
        let mut ip_address = IPAddress::default();
        for (j, v) in split[0].split('.').enumerate() {
            ip_address.ip[j] = match v.parse::<u8>() {
                Ok(ip_num) => ip_num,
                Err(_) => panic!("Cannot parse ip address: {}", address)
            };
        }
        self.length += 1;
        ip_address.port = match split[1].parse::<u32>() {
            Ok(port) => port,
            Err(_) => panic!("Cannot parse port from port from ip address: {}", address)
        };
        self.view[i] = ip_address;   
    }

    pub fn build_ring(&mut self, address:String, i:usize) {
        let mut index = i*64;
        let mut hashed_address:[u8; 20] = address.as_bytes().try_into().expect("Wrong length");
        for _ in 0..64 {
            hashed_address = sha1(&hashed_address).try_into().expect("Wrong length");
            self.ring[index] = VirtualNode{
                hash:hashed_address, 
                id:i as u8
            };
            index += 1;
        }
    }

    pub fn choose_address(self, key:&RawStr) -> String {
        let i:usize = self.search_ring(key) as usize;
        return format!("http://{}", self.view[i].to_string());
    }

    pub fn print_view(self) {
        for i in 0..self.length {
            println!("http://{}", self.view[i].to_string());
        }
    }

    pub fn random_address(self) -> String {
        let mut rng = rand::thread_rng();
        let i:usize = rng.gen_range(0..self.length);
        self.view[i].to_string()
    }

    pub fn search_ring(self, key:&RawStr) -> u8 {
        let mut l:usize = 512 - self.length*64;
        let mut r:usize = 512;
        let key_hash: [u8; 20] = sha1(&key.as_bytes()).try_into().expect("Wrong length");
        if self.ring[0].hash >=  key_hash && self.ring[r-1].hash >= key_hash {
            return self.ring[0].id;
        }
        while l < r {
            let mid = (l + r)/2;
            if self.ring[mid].hash <=  key_hash && self.ring[mid+1].hash >= key_hash {
                return self.ring[mid].id;
            }else if self.ring[mid].hash >= key_hash{
                r = mid;
            } else {
                l = mid + 1;
            }
        }
        eprintln!("Error: could not find address for key l:{}, r:{}", l, r);
        1_u8
    }
}