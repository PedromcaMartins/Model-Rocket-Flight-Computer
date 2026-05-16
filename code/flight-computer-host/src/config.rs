pub struct Config;
impl Config {
    pub const SIM_SOCKET_PATH: &'static str = "fc-sim.sock";
    pub const GS_SOCKET_PATH: &'static str = "fc-gs.sock";
    pub const SERVER_BUFFER_SIZE: usize = 8192;
}
