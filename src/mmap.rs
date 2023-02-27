use lazy_static::lazy_static;

// Note: this requires the `cargo` feature
use std::ffi::{CString,c_void};

fn open_mem()->i32 {
        let cs = CString::new("/dev/mem").unwrap();
        unsafe {
          let fd = libc::open(cs.as_ptr(),libc::O_SYNC|libc::O_RDWR);
          if fd < 0 {
            println!("open /dev/mem error");
          }
          return fd;
        }
}

lazy_static! {
    pub static ref MEM_FD:i32 = open_mem();
}

pub fn ioremap32_read_swap(addr:u32)->Result<u32,String> {
    if let Ok(res) = ioremap32_read(addr) {
        if cfg!(target_endian = "big") {
            Ok(res.to_le())
        } else {
            Ok(res.to_be())
        }
    } else {
        Err("error to perform mmap read".to_owned())
    }
}

pub fn ioremap32_write_swap(addr:u32, write_val:u32)->Result<(),String>{
    let sw;
    if cfg!(target_endian = "big") {
            sw = write_val.to_le();
    } else {
            sw = write_val.to_be();
    }
    if  let Ok(()) = ioremap32_write(addr, sw) {
        Ok(())
    } else {
        Err("error to perform mmap write".to_owned())
    }
}

pub fn ioremap32_write(addr:u32, write_val:u32)->Result<(),String>{
    if *MEM_FD < 0 {
        return Err("handle of /dev/mem is wrong".to_owned());
    }
    let mask = (4096-1) as u32;
    let ofst= addr as u32 & mask;
    let page = addr as u32 & !mask;
    let len=4;

    unsafe {
        #[cfg(any(target_arch = "aarch64", target_arch="x86_64"))]
        let vaddr = libc::mmap64(0 as *mut c_void, (ofst+len) as usize,libc::PROT_READ|libc::PROT_WRITE,libc::MAP_SHARED, *MEM_FD, page as i64);
        #[cfg(target_arch = "powerpc")]
        let vaddr = libc::mmap(0 as *mut c_void, (ofst+len) as usize,libc::PROT_READ|libc::PROT_WRITE,libc::MAP_SHARED, *MEM_FD, page as i32);
        let mapped = std::slice::from_raw_parts_mut(vaddr as *mut u8, len as usize+ofst as usize);
        
        let v = write_val.to_be_bytes().to_vec();
        mapped[ofst as usize]=v[0];
        mapped[ofst as usize+1]=v[1];
        mapped[ofst as usize+2]=v[2];
        mapped[ofst as usize+3]=v[3];
        libc::munmap(vaddr,(ofst+len) as usize);
        Ok(())
    }
}

pub fn ioremap32_read(addr:u32)->Result<u32,String> {
    if *MEM_FD < 0 {
        return Err("handle of /dev/mem is wrong".to_owned());
    }
    let mask = (4096-1) as u32;
    let ofst= addr as u32 & mask;
    let page = addr as u32 & !mask;
    let len=4;

    unsafe {
        #[cfg(any(target_arch = "aarch64", target_arch="x86_64"))]
        let vaddr = libc::mmap64(0 as *mut c_void, (ofst+len) as usize,libc::PROT_READ|libc::PROT_WRITE,libc::MAP_SHARED, *MEM_FD, page as i64);
        #[cfg(target_arch = "powerpc")]
        let vaddr = libc::mmap(0 as *mut c_void, (ofst+len) as usize,libc::PROT_READ|libc::PROT_WRITE,libc::MAP_SHARED, *MEM_FD, page as i32);
        let mapped = std::slice::from_raw_parts_mut(vaddr as *mut u8, len as usize+ofst as usize);
        let mut t_vec=vec![0u8;len as usize+ofst as usize];

        for i in 0..t_vec.len() {
            t_vec[i]=mapped[i];
        }
        let _4_array=[t_vec[ofst as usize], t_vec[ofst as usize+1], t_vec[ofst as usize+2], t_vec[ofst as usize+3]];
        let ret = u32::from_be_bytes(_4_array);
        libc::munmap(vaddr,(ofst+len) as usize);
    //    libc::close(*MEM_FD);
        return Ok(ret);
    }
}
#[test] fn basic_test() {
    ioremap(0x1004);
    println!("{:x}",ioremap32_read(0x10).unwrap());
    println!("{:x}",ioremap32_read_swap(0x10).unwrap());
    ioremap32_write_swap(0x10, 0xf000ff52);
    println!("{:x}",ioremap32_read_swap(0x10).unwrap());
    ioremap32_write(0x10, 0x53ff00f0);
    println!("{:x}",ioremap32_read_swap(0x10).unwrap());
}
pub fn iounmap(addr:* mut c_void, ofst:u32, len:u32) {
    unsafe {
      libc::munmap(addr,(ofst+len) as usize);
    }
}
pub fn ioremap(addr:u32)->Result<(*mut c_void,u32),String> {
    if *MEM_FD < 0 {
        return Err("handle of /dev/mem is wrong".to_owned());
    }
    let mask = (4096-1) as u32;
    let ofst= addr as u32 & mask;
    let page = addr as u32 & !mask;
    let len=4;

    unsafe {
        #[cfg(any(target_arch = "aarch64", target_arch="x86_64"))]
        let mut vaddr = libc::mmap64(0 as *mut c_void, (ofst+len) as usize,libc::PROT_READ|libc::PROT_WRITE,libc::MAP_SHARED, *MEM_FD, page as i64);
        #[cfg(target_arch = "powerpc")]
        let mut vaddr = libc::mmap(0 as *mut c_void, (ofst+len) as usize,libc::PROT_READ|libc::PROT_WRITE,libc::MAP_SHARED, *MEM_FD, page as i32);
        vaddr = vaddr.add(ofst as usize);
        return Ok((vaddr,ofst));
    }
}
