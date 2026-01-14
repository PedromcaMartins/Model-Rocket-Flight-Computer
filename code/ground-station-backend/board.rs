use flight_computer::tasks::postcard::Context;

pub struct GroundStationConfig {
    pub postcard_context: Context,
    pub postcard_server_depth: usize,
    pub postcard_server_receive_buffer_size: usize,
}

impl Default for GroundStationConfig {
    fn default() -> Self {
        Self {
            postcard_context: Context {},
            postcard_server_depth: 1024,
            postcard_server_receive_buffer_size: 1024,
        }
    }
}

pub async fn init() -> Self {
    // postcard rpc setup
    let (postcard_server, postcard_client) = postcard_local_setup(
        board_config.postcard_context,
        board_config.postcard_server_depth,
        board_config.postcard_server_receive_buffer_size,
    );
}
