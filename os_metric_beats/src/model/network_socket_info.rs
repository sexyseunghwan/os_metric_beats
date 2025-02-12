use crate::common::*;

#[derive(Clone, Serialize, Deserialize, Debug, new)]
pub struct NetworkSocketInfo {
    pub tcp_connections: i32,
    pub udp_sockets: i32,
    pub tcp_established: i32,
    pub tcp_timewait: i32,
    pub tcp_listen: i32,
    pub tcp_close_wait: i32,
}
