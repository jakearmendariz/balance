
extern crate crypto;
use rocket::http::RawStr;
use rand::Rng;
use sha1::{Sha1, Digest};
use std::convert::TryInto;

#[derive(Copy, Clone)]
pub struct AppState {
    pub repl_factor:u8,
    pub view:[IPAddress; 8],
    pub length:usize,
    pub ring:[VirtualNode; 512]
}

#[derive(Copy, Clone, Default)]
pub struct IPAddress {
    pub ip:[u8;4],
    pub port:u32,
}

#[derive(Copy, Clone, Default)]
pub struct VirtualNode {
    pub hash:[u8; 20],
    pub id:u8
}

impl IPAddress {
    fn to_string(self) -> String{
        return format!("{}.{}.{}.{}:{}", self.ip[0], self.ip[1], self.ip[2], self.ip[3], self.port);
    }
}

impl Default for AppState {
    fn default() -> Self {
        let mut app_state = AppState {
            repl_factor: std::env::var("REPL_FACTOR").unwrap()
                            .parse::<u8>().unwrap(),
            view: [IPAddress::default(); 8],
            length:0,
            ring:[VirtualNode::default(); 512]
        };
        let view_env = std::env::var("VIEW").unwrap();
        let view_iter = view_env.split(",");
        for (i,address) in view_iter.enumerate() {
            app_state.build_ip(address.to_string(), i);
            let ip_address = app_state.view[i];
            app_state.build_ring(ip_address.to_string(), i);
        }
        app_state.ring.sort_by(|a, b| a.hash.cmp(&b.hash));
        return app_state;
    }
}


impl AppState {
    pub fn build_ip(&mut self, address:String, i:usize) {
        let split = address.split(":").collect::<Vec<&str>>();
        let mut ip_address = IPAddress::default();
        for (j, v) in split[0].split(".").enumerate() {
            ip_address.ip[j] = v.parse::<u8>().unwrap();
        }
        self.length += 1;
        ip_address.port = split[1].parse::<u32>().unwrap();
        self.view[i] = ip_address;   
    }
    
    pub fn build_ring(&mut self, address:String, i:usize) {
        let mut index = i*64;
        let mut result;
        {
            let mut hasher = Sha1::new();
            hasher.update(address);
            result = hasher.finalize();
            self.ring[index] = VirtualNode {
                hash:result.as_slice().try_into().expect("Wrong length"), 
                id:i as u8
            };
        }
        for _ in 1..64 {
            index += 1;
            let mut hasher = Sha1::new();
            hasher.update(result);
            result = hasher.finalize();
            let hash_arr:[u8; 20] = result.as_slice().try_into().expect("Wrong length");
            self.ring[index] = VirtualNode{hash:hash_arr, id:i as u8};
        }
    }

    pub fn choose_address(self, key:&RawStr) -> String {
        let i = self.search_ring(key);
        return format!("http://localhost:{}", self.view[i as usize].port)
    }

    pub fn print_view(self) {
        for i in 0..self.length {
            println!("{}", format!("http://localhost:{}", self.view[i].port))
        }
    }

    pub fn random_address(self) -> String {
        let mut rng = rand::thread_rng();
        let i:usize = rng.gen_range(0..self.length);
        return format!("http://localhost:{}", self.view[i].port);
    }

    pub fn search_ring(self, key:&RawStr) -> u8 {
        let mut l:usize = 512 - self.length*64;
        let mut r:usize = 512;
        let mut hasher = Sha1::new();
        hasher.update(key);
        let key_hash: [u8; 20] = hasher.finalize().as_slice().try_into().expect("Wrong length");
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
        println!("Error: could not find address for key l:{}, r:{}", l, r);
        return 1 as u8;
    }
}