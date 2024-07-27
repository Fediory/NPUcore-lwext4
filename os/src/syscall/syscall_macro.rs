/// 根据给出的 sockfd，返回 socket，找不到则返回 ENOTSOCK
macro_rules! get_socket {
    ($sockfd:expr) => {{
        match current_task().unwrap().socket_table.lock().get_ref($sockfd as usize) {
            Some(socket) => socket.clone(),
            None => return ENOTSOCK,
        }
    }}
}

/// 根据给出的 addr 和 addrlen，将用户空间的虚拟地址转化为物理地址buf，地址不合法返回错误
macro_rules! trans_ref {
    ($addr:expr, $addrlen:expr) => {{
        let token = current_task().unwrap().get_user_token();
        match translated_ref(token, $addr as *const u8) {
            Ok(addr) => {
                unsafe { core::slice::from_raw_parts(addr as *const u8, $addrlen as usize) }
            }
            Err(errno) => {
                return errno;
            }
        }
    }}
}