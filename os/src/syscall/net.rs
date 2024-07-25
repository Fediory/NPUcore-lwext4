use crate::mm::{translated_ref, translated_refmut};
use crate::{
    config::PAGE_SIZE, fs::FileDescriptor, net::{
        address::{self, SocketAddrv4},
        make_unix_socket_pair, Socket, SocketType, TCP_MSS,
    }, 
    task::current_task,
};
use super::errno::*;

use log::info;
use smoltcp::wire::IpListenEndpoint;
/// level
const SOL_SOCKET: u32 = 1;
const SOL_TCP: u32 = 6;
/// option name
const TCP_NODELAY: u32 = 1;
const TCP_MAXSEG: u32 = 2;
#[allow(unused)]
const TCP_INFO: u32 = 11;
const TCP_CONGESTION: u32 = 13;
const SO_SNDBUF: u32 = 7;
const SO_RCVBUF: u32 = 8;
const SO_KEEPALIVE: u32 = 9;

pub fn sys_socket(domain: u32, socket_type: u32, protocol: u32) -> isize {
    info!(
        "[sys_socket] domain: {}, type: {}, protocol: {}",
        domain, socket_type, protocol
    );
    let result = match <dyn Socket>::alloc(domain, socket_type){
        Ok(sockfd) => {
            info!("[sys_socket] new sockfd: {}", sockfd);
            sockfd as isize
        },
        Err(e) => {
            info!("[sys_socket] new sockfd failed", );
            -(e as isize)
        }
    };
    result
}

pub fn sys_bind(sockfd: u32, addr: usize, addrlen: u32) -> isize {
    let addr_buf = trans_ref!(addr, addrlen);
    let socket = get_socket!(sockfd);
    let endpoint = address::listen_endpoint(addr_buf).unwrap();
    match socket.socket_type() {
        SocketType::SOCK_STREAM => socket.bind(endpoint).unwrap() as isize,
        SocketType::SOCK_DGRAM => {
            let res = current_task().unwrap().socket_table.lock().can_bind(endpoint);
            if res.is_none(){
                info!("[sys_bind] not find port exist");
                socket.bind(endpoint).unwrap() as isize
            }else {
                let (_,sock) = res.unwrap();
                current_task().unwrap().socket_table.lock().insert(sockfd as usize, sock.clone());
                let _ = current_task().unwrap().files.lock().insert(FileDescriptor::new(false,false,sock));
                0
            }
        }
        _ => todo!(),
    }
}

pub fn sys_listen(sockfd: u32, _backlog: u32) -> isize {
    let socket = get_socket!(sockfd);
    socket.listen().unwrap() as isize
}

pub  fn sys_accept(sockfd: u32, addr: usize, addrlen: usize) -> isize {
    let socket = get_socket!(sockfd);
    socket.accept(sockfd, addr, addrlen).unwrap() as isize
}

pub  fn sys_connect(sockfd: u32, addr: usize, addrlen: u32) -> isize {
    let addr_buf = trans_ref!(addr, addrlen);
    let socket = get_socket!(sockfd);
    socket.connect(addr_buf).unwrap() as isize
}

pub fn sys_getsockname(sockfd: u32, addr: usize, addrlen: usize) -> isize {
    let socket = get_socket!(sockfd);
    socket.addr(addr, addrlen).unwrap() as isize
}

pub fn sys_getpeername(sockfd: u32, addr: usize, addrlen: usize) -> isize {
    let socket = get_socket!(sockfd);
    socket.peer_addr(addr, addrlen).unwrap() as isize
}

pub fn sys_sendto(
    sockfd: u32,
    buf: usize,
    len: usize,
    _flags: u32,
    dest_addr: usize,
    addrlen: u32,
) -> isize {
    let task = current_task().unwrap();
    let socket_file = match task.files.lock().get_ref(sockfd as usize) {
        Ok(file) => file.clone(),
        Err(e) => return e,
    };
    let buf = trans_ref!(buf, len);
    let socket = get_socket!(sockfd);
    log::info!("[sys_sendto] get socket sockfd: {}", sockfd);
    let mut offset = 0 as usize; 
    let len = match socket.socket_type() {
        SocketType::SOCK_STREAM => socket_file.file.write(Some(&mut offset),buf),
        SocketType::SOCK_DGRAM => {
            info!("[sys_sendto] socket is udp");
            if socket.loacl_endpoint().port == 0 {
                let addr = SocketAddrv4::new([0; 16].as_slice());
                let endpoint = IpListenEndpoint::from(addr);
                let _ = socket.bind(endpoint);
            }
            let dest_addr = trans_ref!(dest_addr, addrlen);
            let _ = socket.connect(dest_addr);
            socket_file.file.write(Some(&mut offset),buf)
        }
        _ => todo!(),
    };
    len as isize
}

pub  fn sys_recvfrom(
    sockfd: u32,
    buf: usize,
    len: u32,
    _flags: u32,
    src_addr: usize,
    addrlen: usize,
) -> isize {
    let socket_file = current_task().unwrap().files.lock().get_ref(sockfd as usize).unwrap().clone();
    let task = current_task().unwrap();
    let token = task.get_user_token();
    let buf = translated_refmut(token, buf as *mut u8).unwrap();
    let buf = unsafe { core::slice::from_raw_parts_mut(buf as *mut u8, len as usize) };
    //info!("[sys_recvfrom] file filags: {:?}", socket_file.flags);
    let socket = get_socket!(sockfd);

    info!("[sys_recvfrom] get socket sockfd: {}", sockfd);

    let mut offset = 0 as usize;
    match socket.socket_type() {
        SocketType::SOCK_STREAM => {
            let len = socket_file.file.read(Some(&mut offset),buf);
            if src_addr != 0 {
                let _ = socket.peer_addr(src_addr, addrlen);
            }
            len as isize
        }
        SocketType::SOCK_DGRAM => {
            let len = socket_file.file.read(Some(&mut offset),buf);
            if src_addr != 0 {
                let _ = socket.peer_addr(src_addr, addrlen);
            }
            len as isize
        }
        _ => todo!(),
    }
}

pub fn sys_getsockopt(
    sockfd: u32,
    level: u32,
    optname: u32,
    optval_ptr_: usize,
    optlen: usize,
) -> isize {
    let task = current_task().unwrap();
    let token = task.get_user_token();
    let optval_ptr = translated_refmut(token, optval_ptr_ as *mut u32).unwrap();
    let optlen = translated_refmut(token, optlen as *mut u32).unwrap();
    match (level, optname) {
        (SOL_TCP, TCP_MAXSEG) => {
            // return max tcp fregment size (MSS)
            let len = core::mem::size_of::<u32>();
            unsafe {
                *(optval_ptr as *mut u32) = TCP_MSS;
                *(optlen as *mut u32) = len as u32;
            }
        }
        (SOL_TCP, TCP_CONGESTION) => {
            let optval_ptr = translated_refmut(token, optval_ptr_ as *mut u8).unwrap();
            let congestion = "reno";
            let buf =
                unsafe { core::slice::from_raw_parts_mut(optval_ptr as *mut u8, congestion.len()) };
            buf.copy_from_slice(congestion.as_bytes());
            unsafe {
                *(optlen as *mut u32) = congestion.len() as u32;
            }
        }
        (SOL_SOCKET, SO_SNDBUF | SO_RCVBUF) => {
            // let len = core::mem::size_of::<u32>();
            let socket = get_socket!(sockfd);

            match optname {
                SO_SNDBUF => {
                    let size = socket.send_buf_size();
                    unsafe {
                        *(optval_ptr as *mut u32) = size as u32;
                        *(optlen as *mut u32) = 4;
                    }
                }
                SO_RCVBUF => {
                    let size = socket.recv_buf_size();
                    unsafe {
                        *(optval_ptr as *mut u32) = size as u32;
                        *(optlen as *mut u32) = 4;
                    }
                }
                _ => {}
            }
        }
        _ => {
            log::warn!("[sys_getsockopt] level: {}, optname: {}", level, optname);
        }
    }
    0 as isize
}

pub fn sys_setsockopt(
    sockfd: u32,
    level: u32,
    optname: u32,
    optval_ptr: usize,
    _optlen: u32,
) -> isize {
    let socket = get_socket!(sockfd);
    let task = current_task().unwrap();
    let token = task.get_user_token();
    let optval_ptr = translated_refmut(token, optval_ptr as *mut u32).unwrap();
    match (level, optname) {
        (SOL_SOCKET, SO_SNDBUF | SO_RCVBUF) => {
            let size = unsafe { *(optval_ptr as *mut u32) };
            match optname {
                SO_SNDBUF => {
                    socket.set_send_buf_size(size as usize);
                }
                SO_RCVBUF => {
                    socket.set_recv_buf_size(size as usize);
                }
                _ => {}
            }
        }
        (SOL_TCP, TCP_NODELAY) => {
            // close Nagle’s Algorithm
            let enabled = unsafe { *(optval_ptr as *const u32) };
            log::debug!("[sys_setsockopt] set TCPNODELY: {}", enabled);
            let _ = match enabled {
                0 => socket.set_nagle_enabled(true),
                _ => socket.set_nagle_enabled(false),
            };
        }
        (SOL_SOCKET, SO_KEEPALIVE) => {
            let enabled = unsafe { *(optval_ptr as *const u32) };
            log::debug!("[sys_setsockopt] set socket KEEPALIVE: {}", enabled);
            let _ = match enabled {
                1 => socket.set_keep_alive(true),
                _ => socket.set_keep_alive(false),
            };
        }
        _ => {
            log::warn!("[sys_setsockopt] level: {}, optname: {}", level, optname);
        }
    }
    0 as isize
}

pub fn sys_sock_shutdown(sockfd: u32, how: u32) -> isize {
    log::info!("[sys_shutdown] sockfd {}, how {}", sockfd, how);
    let socket = get_socket!(sockfd);
    let _ = socket.shutdown(how);
    0 as isize
}

pub fn sys_socketpair(domain: u32, socket_type: u32, protocol: u32, sv: usize) -> isize {
    info!(
        "[sys_socketpair] domain {}, type {}, protocol {}, sv {}",
        domain, socket_type, protocol, sv
    );
    let len = 2 * core::mem::size_of::<u32>();
    let sv = unsafe { core::slice::from_raw_parts_mut(sv as *mut u32, len) };
    let (socket1, socket2) = make_unix_socket_pair::<PAGE_SIZE>();
    let fd1 = current_task().unwrap().files.lock().insert(FileDescriptor::new(false, false, socket1));
    let fd2 = current_task().unwrap().files.lock().insert(FileDescriptor::new(false, false, socket2));
    sv[0] = fd1.unwrap() as u32;
    sv[1] = fd2.unwrap() as u32;
    info!("[sys_socketpair] new sv: {:?}", sv);
    0 as isize
}
