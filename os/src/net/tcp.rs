use super::{Mutex, Socket};
use crate::{
    fs::{file_trait::File, FileDescriptor, OpenFlags}, net::{
        address,
        config::NET_INTERFACE,
        MAX_BUFFER_SIZE, SHUT_WR,
    }, task::current_task, utils::{
        error::{GeneralRet, SyscallErr, SyscallRet},
        random::RNG,
    }
};
use alloc::{ sync::Arc, vec};
use core::time::Duration;
use log::info;
use smoltcp::{
    iface::SocketHandle,
    socket::{self, tcp},
    wire::{IpEndpoint, IpListenEndpoint},
};

use crate::mm::UserBuffer;
use crate::fs::{Stat,Statx};
use crate::fs::DiskInodeType;
use alloc::sync::Weak;
use crate::fs::directory_tree::DirectoryTreeNode;
use alloc::vec::Vec;
use alloc::string::String;
use crate::fs::Dirent;
use crate::fs::SeekWhence;
use crate::fs::PageCache;



pub const TCP_MSS_DEFAULT: u32 = 1 << 15;
pub const TCP_MSS: u32 = if TCP_MSS_DEFAULT > MAX_BUFFER_SIZE as u32 {
    MAX_BUFFER_SIZE as u32
} else {
    TCP_MSS_DEFAULT
};

pub struct TcpSocket {
    inner: Mutex<TcpSocketInner>,
    socket_handler: SocketHandle,
}

#[allow(unused)]
struct TcpSocketInner {
    local_endpoint: IpListenEndpoint,
    remote_endpoint: Option<IpEndpoint>,
    last_state: tcp::State,
    recvbuf_size: usize,
    sendbuf_size: usize,
    // TODO: add more
}

impl Socket for TcpSocket {
    fn bind(&self, addr: IpListenEndpoint) -> SyscallRet {
        info!("[tcp::bind] bind to: {:?}", addr);
        self.inner.lock().local_endpoint = addr;
        Ok(0)
    }

    fn listen(&self) -> SyscallRet {
        let local = self.inner.lock().local_endpoint;
        info!(
            "[Tcp::listen] {} listening: {:?}",
            self.socket_handler, local
        );
        NET_INTERFACE.tcp_socket(self.socket_handler, |socket| {
            let ret = socket.listen(local).ok().ok_or(SyscallErr::EADDRINUSE);
            self.inner.lock().last_state = socket.state();
            ret
        })?;
        Ok(0)
    }

    fn accept(&self, sockfd: u32, addr: usize, addrlen: usize) -> crate::utils::error::SyscallRet {
        // get old socket
        let task = current_task().unwrap();
        let mut fd_table = task.files.lock();
        let mut socket_table = task.socket_table.lock();
        let old_file = fd_table.get_ref(sockfd as usize).unwrap();
        let old_nonblock = old_file.get_nonblock();
        let old_cloexec = old_file.get_cloexec();

        let peer_addr = self._accept(old_nonblock)?;
        log::info!("[Socket::accept] get peer_addr: {:?}", peer_addr);
        let local = self.loacl_endpoint();
        log::info!("[Socket::accept] new socket try bind to : {:?}", local);
        let new_socket = TcpSocket::new();
        use core::convert::TryInto;
        new_socket.bind(local.try_into().expect("cannot convert to ListenEndpoint"))?;
        log::info!("[Socket::accept] new socket listen");
        new_socket.listen()?;
        address::fill_with_endpoint(peer_addr, addr, addrlen)?;
        let new_socket = Arc::new(new_socket);
        log::debug!("[Socket::accept] take old sock");
        // 取出旧的
        let old_file = fd_table.take(sockfd as usize).unwrap();
        let old_socket: Option<Arc<dyn Socket>> =
            socket_table.get_ref(sockfd as usize).cloned();
        // 新的替换旧的
        log::debug!("[Socket::accept] replace old sock to new");
        let _ = fd_table.insert_at(
            FileDescriptor::new(old_cloexec, old_nonblock, new_socket.clone()),
            sockfd as usize,
        );
        socket_table
            .insert(sockfd as usize, new_socket.clone());
        // 旧的插在新的fd上
        let fd = fd_table.insert(old_file).unwrap();
        socket_table.insert(fd, old_socket.unwrap());
        log::info!("[Socket::accept] insert old sock to newfd: {}", fd);
        Ok(fd)
    }

    fn socket_type(&self) -> super::SocketType {
        super::SocketType::SOCK_STREAM
    }

    fn connect<'a>(&'a self, addr_buf: &'a [u8]) -> crate::utils::error::SyscallRet {
        let remote_endpoint = address::endpoint(addr_buf)?;
        self._connect(remote_endpoint)?;
        loop {
            let state = NET_INTERFACE.tcp_socket(self.socket_handler, |socket| socket.state());
            match state {
                tcp::State::Closed => {
                    // close but not already connect, retry
                    info!("[Tcp::connect] {} already closed, try again", self.socket_handler);
                    self._connect(remote_endpoint)?;
                }
                tcp::State::Established => {
                    info!("[Tcp::connect] {} connected, state {:?}", self.socket_handler, state);
                    return Ok(0);
                }
                _ => {
                    info!("[Tcp::connect] {} not connect yet, state {:?}", self.socket_handler, state);
                }
            }
            suspend_current_and_run_next();
            // thread::sleep(Duration::from_secs(1));
        }
    }
    fn recv_buf_size(&self) -> usize {
        self.inner.lock().recvbuf_size
    }

    fn send_buf_size(&self) -> usize {
        self.inner.lock().sendbuf_size
    }

    fn set_recv_buf_size(&self, size: usize) {
        self.inner.lock().recvbuf_size = size;
    }

    fn set_send_buf_size(&self, size: usize) {
        self.inner.lock().sendbuf_size = size;
    }

    fn loacl_endpoint(&self) -> IpListenEndpoint {
        self.inner.lock().local_endpoint
    }

    fn remote_endpoint(&self) -> Option<IpEndpoint> {
        NET_INTERFACE.poll();
        let ret = NET_INTERFACE.tcp_socket(self.socket_handler, |socket| socket.remote_endpoint());
        NET_INTERFACE.poll();
        ret
    }

    fn shutdown(&self, how: u32) -> GeneralRet<()> {
        info!("[TcpSocket::shutdown] how {}", how);
        NET_INTERFACE.tcp_socket(self.socket_handler, |socket| match how {
            SHUT_WR => socket.close(),
            _ => socket.abort(),
        });
        NET_INTERFACE.poll();
        Ok(())
    }

    fn set_nagle_enabled(&self, enabled: bool) -> SyscallRet {
        NET_INTERFACE.tcp_socket(self.socket_handler, |socket| {
            socket.set_nagle_enabled(enabled)
        });
        Ok(0)
    }

    fn set_keep_alive(&self, enabled: bool) -> SyscallRet {
        if enabled {
            NET_INTERFACE.tcp_socket(self.socket_handler, |socket| {
                socket.set_keep_alive(Some(Duration::from_secs(1).into()))
            });
        }
        Ok(0)
    }
}

impl TcpSocket {
    pub fn new() -> Self {
        let tx_buf = socket::tcp::SocketBuffer::new(vec![0 as u8; MAX_BUFFER_SIZE]);
        let rx_buf = socket::tcp::SocketBuffer::new(vec![0 as u8; MAX_BUFFER_SIZE]);
        let socket = socket::tcp::Socket::new(rx_buf, tx_buf);
        let socket_handler = NET_INTERFACE.add_socket(socket);
        info!("[TcpSocket::new] new {}", socket_handler);
        NET_INTERFACE.poll();
        Self {
            socket_handler,
            inner: Mutex::new(TcpSocketInner {
                local_endpoint: IpListenEndpoint {
                    addr: None,
                    port: unsafe { RNG.positive_u32() as u16 },
                },
                remote_endpoint: None,
                last_state: tcp::State::Closed,
                recvbuf_size: MAX_BUFFER_SIZE,
                sendbuf_size: MAX_BUFFER_SIZE,
            }),
        }
    }

    fn _connect(&self, remote_endpoint: IpEndpoint) -> GeneralRet<()> {
        self.inner.lock().remote_endpoint = Some(remote_endpoint);
        let local = self.inner.lock().local_endpoint;
        info!(
            "[Tcp::connect] local: {:?}, remote: {:?}",
            local, remote_endpoint
        );
        NET_INTERFACE.inner_handler(|inner| {
            let socket = inner.sockets.get_mut::<tcp::Socket>(self.socket_handler);
            let ret = socket.connect(inner.iface.context(), remote_endpoint, local);
            if ret.is_err() {
                log::info!("[Tcp::connect] {} connect error occur", self.socket_handler);
                match ret.err().unwrap() {
                    tcp::ConnectError::Unaddressable => return Err(SyscallErr::EINVAL),
                    tcp::ConnectError::InvalidState => return Err(SyscallErr::EISCONN),
                }
            }
            info!("berfore poll socket state: {}", socket.state());
            Ok(())
        })?;
        Ok(())
    }
    fn _accept(&self, nonblock: bool) -> GeneralRet<IpEndpoint> {
        loop {
            NET_INTERFACE.poll();
            let ret = NET_INTERFACE.tcp_socket(self.socket_handler, |socket| {
                if !socket.is_open() {
                    log::info!("[TcpAcceptFuture::poll] this socket is not open");
                    return Err(SyscallErr::EINVAL);
                }
                if socket.state() == tcp::State::SynReceived
                    || socket.state() == tcp::State::Established
                {
                    self.inner.lock().last_state = socket.state();
                    log::info!("[TcpAcceptFuture::poll] state become {:?}", socket.state());
                    return Ok(socket.remote_endpoint().unwrap());
                }
                // log::info!(
                //     "[TcpAcceptFuture::poll] not syn yet, state {:?}",
                //     socket.state()
                // );
                if nonblock {
                    log::info!("[TcpAcceptFuture::poll] flags set nonblock");
                    return Err(SyscallErr::EAGAIN);
                }
                // 使用 continue 跳过当前循环并开始下一次迭代
                return Err(SyscallErr::EAGAIN);
            });
            NET_INTERFACE.poll();
            match ret {
                Ok(endpoint) => return GeneralRet::Ok(endpoint),
                Err(SyscallErr::EAGAIN) => {
                    suspend_current_and_run_next();
                    // 如果返回 EAGAIN 错误，继续循环
                    continue;
                }
                Err(err) => return GeneralRet::Err(err),
            }
        }
    }
}
use crate::task::suspend_current_and_run_next;
impl Drop for TcpSocket {
    fn drop(&mut self) {
        info!(
            "[TcpSocket::drop] drop socket {}, localep {:?}",
            self.socket_handler,
            self.inner.lock().local_endpoint
        );
        NET_INTERFACE.tcp_socket(self.socket_handler, |socket| {
            info!("[TcpSocket::drop] before state is {:?}", socket.state());
            if socket.is_open() {
                socket.close();
            }
            info!("[TcpSocket::drop] after state is {:?}", socket.state());
        });
        NET_INTERFACE.poll();
        NET_INTERFACE.remove(self.socket_handler);
        NET_INTERFACE.poll();
    }
}

impl File for TcpSocket {
    fn deep_clone(&self) -> Arc<dyn File>{
        todo!();
    }
    fn readable(&self) -> bool{
        true
    }
    fn writable(&self) -> bool{
        true
    }
    fn read(&self, _offset: Option<&mut usize>, buf: &mut [u8]) -> usize{
        let ret = self._read(buf).unwrap();
        ret
    }
    fn write(&self, _offset: Option<&mut usize>, buf: &[u8]) -> usize{
        NET_INTERFACE.poll();
        let ret = NET_INTERFACE.tcp_socket(self.socket_handler, |socket| {
            if !socket.may_send() {
                log::info!("[TcpSendFuture::poll] err when send");
                return SyscallErr::ENOTCONN as usize;
            }
            if !socket.can_send() {
                log::info!("[TcpSendFuture::poll] cannot send yet");
                suspend_current_and_run_next();
                return SyscallErr::EAGAIN as usize;
            }
            log::info!("[TcpSendFuture::poll] start to send...");
            info!(
                "[TcpSendFuture::poll] {:?} -> {:?}",
                socket.local_endpoint(),
                socket.remote_endpoint()
            );
            match socket.send_slice(buf) {
                Ok(nbytes) => {
                    info!("[TcpSendFuture::poll] send {} bytes", nbytes);
                    return nbytes;
                }
                Err(_) => SyscallErr::ENOTCONN as usize,
            }
        });
        NET_INTERFACE.poll();
        ret
    }
    fn r_ready(&self) -> bool{true}
    fn w_ready(&self) -> bool{true}
    fn read_user(&self, _offset: Option<usize>, buf: UserBuffer) -> usize{
        let mut buffers = buf.buffers;
        let buf = unsafe { core::slice::from_raw_parts_mut(buffers[0].as_mut_ptr() as *mut u8, buf.len as usize) };
        let ret = self._read(buf).unwrap();
        ret
    }
    fn write_user(&self, _offset: Option<usize>, buf: UserBuffer) -> usize{
        let mut buffers = buf.buffers;
        let buf = unsafe { core::slice::from_raw_parts_mut(buffers[0].as_mut_ptr() as *mut u8, buf.len as usize) };
        self.write(None, buf)
    }
    fn get_size(&self) -> usize{todo!();}
    fn get_stat(&self) -> Stat{todo!();}
    fn get_statx(&self) -> Statx{todo!();}
    fn get_file_type(&self) -> DiskInodeType{todo!();}
    fn is_dir(&self) -> bool {todo!();}
    fn is_file(&self) -> bool {todo!();}
    fn info_dirtree_node(&mut self, _dirnode_ptr: Weak<DirectoryTreeNode>){todo!();}
    fn get_dirtree_node(&self) -> Option<Arc<DirectoryTreeNode>>{todo!();}
    /// open
    fn open(&self, _flags: OpenFlags, _special_use: bool) -> Arc<dyn File>{todo!();}
    fn open_subfile(&self) -> Result<Vec<(String, Arc<dyn File>)>, isize>{todo!();}
    /// create
    fn create(&self, _name: &str, _file_type: DiskInodeType) -> Result<Arc<dyn File>, isize>{todo!();}
    fn link_child(&self, _name: &str, _child: &Self) -> Result<(), isize>{todo!();}
    /// delete(unlink)
    fn unlink(&self, _delete: bool) -> Result<(), isize>{todo!();}
    /// dirent
    fn get_dirent(&self, _count: usize) -> Vec<Dirent>{todo!();}
    /// offset
    fn get_offset(&self) -> usize {todo!();}
    fn lseek(&self, _offset: isize, _whence: SeekWhence) -> Result<usize, isize>{todo!();}
    /// size
    fn modify_size(&self, _diff: isize) -> Result<(), isize>{todo!();}
    fn truncate_size(&self, _new_size: usize) -> Result<(), isize>{todo!();}
    // time
    fn set_timestamp(&self, _ctime: Option<usize>, _atime: Option<usize>, _mtime: Option<usize>){todo!();}
    /// cache
    fn get_single_cache(&self, _offset: usize) -> Result<Arc<Mutex<PageCache>>, ()>{todo!();}
    fn get_all_caches(&self) -> Result<Vec<Arc<Mutex<PageCache>>>, ()>{todo!();}
    /// memory related
    fn oom(&self) -> usize{todo!();}
    /// poll, select related
    fn hang_up(&self) -> bool{todo!();}
    /// iotcl
    fn ioctl(&self, _cmd: u32, _argp: usize) -> isize {todo!();}
    /// fcntl
    fn fcntl(&self, _cmd: u32, _arg: u32) -> isize{todo!();}

    
}

impl TcpSocket {
    fn _read<'a>(&'a self, buf: &'a mut [u8]) -> GeneralRet<usize> {
        loop {
            NET_INTERFACE.poll();
            let ret = NET_INTERFACE.tcp_socket(self.socket_handler, |socket| {
                if socket.state() == tcp::State::CloseWait || 
                socket.state() == tcp::State::TimeWait || socket.state() == tcp::State::FinWait2{
                    log::info!("[TcpRecvFuture::poll] state become {:?}", socket.state());
                    return Ok(0);
                }
                if !socket.may_recv() {
                    log::info!(
                        "[TcpRecvFuture::poll] err when recv, state {:?}",
                        socket.state()
                    );
                    return Err(SyscallErr::ENOTCONN);
                }
                log::info!("[TcpRecvFuture::poll] state {:?}", socket.state());
                if !socket.can_recv() {
                    // panic!();
                    log::info!("[TcpRecvFuture::poll] cannot recv yet");
                    return Err(SyscallErr::EAGAIN);
                }
                log::info!("[TcpRecvFuture::poll] start to recv...");
                info!(
                    "[TcpRecvFuture::poll] {:?} <- {:?}",
                    socket.local_endpoint(),
                    socket.remote_endpoint()
                );
                match socket.recv_slice(buf) {
                    Ok(nbytes) => {
                        info!("[TcpRecvFuture::poll] recv {} bytes", nbytes);
                        Ok(nbytes)
                    }
                    Err(_) => return Err(SyscallErr::ENOTCONN),
                }
            });
            NET_INTERFACE.poll();
            match ret {
                Ok(result) => return GeneralRet::Ok(result),
                Err(SyscallErr::EAGAIN) => {
                    suspend_current_and_run_next();
                    // 如果返回 EAGAIN 错误，继续循环
                    continue;
                }
                Err(err) => return GeneralRet::Err(err),
            }
        }
    }
}