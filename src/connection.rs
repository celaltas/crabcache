use mio::net::TcpStream;

const MAX_MESSAGE_SIZE: usize = 4096;

#[derive(Debug)]
pub enum ConnectionState {
    ReadyToRead,
    ReadyToWrite,
    Closing,
}

pub struct Connection {
    pub stream: TcpStream,
    pub state: ConnectionState,
    read_buffer_size: usize,
    pub read_buffer: [u8; 4 + MAX_MESSAGE_SIZE],
    write_buffer_size: usize,
    pub write_buffer: [u8; 4 + MAX_MESSAGE_SIZE],
    write_buffer_sent: usize,
}

impl Connection {
    pub fn new(stream: TcpStream) -> Connection {
        Connection {
            stream,
            state: ConnectionState::ReadyToRead,
            read_buffer_size: 0,
            read_buffer: [0; 4 + MAX_MESSAGE_SIZE],
            write_buffer_size: 0,
            write_buffer: [0; 4 + MAX_MESSAGE_SIZE],
            write_buffer_sent: 0,
        }
    }

    pub fn state(&self) -> &ConnectionState {
        &self.state
    }

    pub fn stream(&self) -> &TcpStream {
        &self.stream
    }

    pub fn stream_mut(&mut self) -> &mut TcpStream {
        &mut self.stream
    }

    pub fn set_state(&mut self, state: ConnectionState) {
        self.state = state;
    }

    pub fn get_write_buffer(&self) -> &[u8] {
        &self.write_buffer
    }

    pub fn reset_write_buffer(&mut self) {
        self.write_buffer_size = 0;
        self.write_buffer_sent = 0;
        self.write_buffer = [0; 4 + MAX_MESSAGE_SIZE];
    }

    pub fn reset_read_buffer(&mut self) {
        self.read_buffer_size = 0;
        self.read_buffer = [0; 4 + MAX_MESSAGE_SIZE];
    }
}
