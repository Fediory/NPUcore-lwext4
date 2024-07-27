use super::Mutex;
use super::Socket;
use crate::{
    fs::{
        dev::pipe::{make_pipe,Pipe},
        file_trait::File,  OpenFlags,
    },
    utils::error::{SyscallErr,SyscallRet},
};
use alloc::sync::Arc;
use smoltcp::wire::IpEndpoint;

use crate::mm::UserBuffer;
use crate::fs::Stat;
use crate::fs::Statx;
use crate::fs::DiskInodeType;
use alloc::sync::Weak;
use crate::fs::directory_tree::DirectoryTreeNode;
use alloc::vec::Vec;
use alloc::string::String;
use crate::fs::Dirent;
use crate::fs::SeekWhence;
use crate::fs::PageCache;
#[allow(unused)]
pub struct UnixSocket<const N: usize> {
    //file_meta: FileMeta,
    // read_end: Arc<Pipe<N>>,
    // write_end: Arc<Pipe<N>>,
    read_end: Arc<Pipe>,
    write_end: Arc<Pipe>,
}

impl<const N: usize> Socket for UnixSocket<N> {
    fn bind(&self, _addr: smoltcp::wire::IpListenEndpoint) -> crate::utils::error::SyscallRet {
        todo!();
    }

    fn listen(&self) -> crate::utils::error::SyscallRet {
        todo!();
   }

    fn connect(&self, _addr_buf: &[u8]) -> SyscallRet {
        todo!();
    }

    fn accept(&self, _sockfd: u32, _addr: usize, _addrlen: usize) -> SyscallRet {
        todo!();
    }

    fn socket_type(&self) -> super::SocketType {
        todo!()
    }

    fn recv_buf_size(&self) -> usize {
        todo!()
    }

    fn send_buf_size(&self) -> usize {
        todo!()
    }

    fn set_recv_buf_size(&self, _size: usize) {
        todo!()
    }

    fn set_send_buf_size(&self, _size: usize) {
        todo!()
    }

    fn loacl_endpoint(&self) -> smoltcp::wire::IpListenEndpoint {
        todo!()
    }

    fn remote_endpoint(&self) -> Option<IpEndpoint> {
        None
    }

    fn shutdown(&self, how: u32) -> crate::utils::error::GeneralRet<()> {
        log::info!("[UnixSocket::shutdown] how {}", how);
        Ok(())
    }

    fn set_nagle_enabled(&self, _enabled: bool) -> crate::utils::error::SyscallRet {
        Err(SyscallErr::EOPNOTSUPP)
    }

    fn set_keep_alive(&self, _enabled: bool) -> crate::utils::error::SyscallRet {
        Err(SyscallErr::EOPNOTSUPP)
    }
}

impl<const N: usize> UnixSocket<N> {
    pub fn new(read_end: Arc<Pipe>, write_end: Arc<Pipe>) -> Self {
        Self {
            //file_meta: FileMeta::new(crate::fs::InodeMode::FileSOCK),
            // buf: Mutex::new(VecDeque::new()),
            read_end,
            write_end,
        }
    }
}
impl<const N: usize> File for UnixSocket<N> {
    fn deep_clone(&self) -> Arc<dyn File>{
        todo!();
    }
    fn readable(&self) -> bool{
        todo!();
    }
    fn writable(&self) -> bool{
        todo!();
    }
    fn read(&self, _offset: Option<&mut usize>, _buf: &mut [u8]) -> usize{todo!();}
    fn write(&self, _offset: Option<&mut usize>, _buf: &[u8]) -> usize{todo!();}
    fn r_ready(&self) -> bool{todo!();}
    fn w_ready(&self) -> bool{todo!();}
    fn read_user(&self, _offset: Option<usize>, _buf: UserBuffer) -> usize{todo!();}
    fn write_user(&self, _offset: Option<usize>, _buf: UserBuffer) -> usize{todo!();}
    fn get_size(&self) -> usize{todo!();}
    fn get_stat(&self) -> Stat{todo!();}
    fn get_statx(&self) -> Statx{todo!();}
    fn get_file_type(&self) -> DiskInodeType{todo!();}
    fn is_dir(&self) -> bool {todo!();}
    fn is_file(&self) -> bool {todo!();}
    fn info_dirtree_node(&self, _dirnode_ptr: Weak<DirectoryTreeNode>){todo!();}
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

pub fn make_unix_socket_pair<const N: usize>() -> (Arc<UnixSocket<N>>, Arc<UnixSocket<N>>){
    let (read1, write1) = make_pipe();
    let (read2, write2) = make_pipe();
    let socket1 = Arc::new(UnixSocket::new(read1, write2));
    let socket2 = Arc::new(UnixSocket::new(read2, write1));
    (socket1, socket2)
}
