use anyhow::Result;
use byteorder::{ByteOrder, LittleEndian};

pub mod del;
pub mod get;
pub mod set;

#[derive(Debug)]
pub enum Command {
    Get(Vec<u8>),
    Set(Vec<u8>, Vec<u8>),
    Del(Vec<u8>),
}

impl PartialEq for Command {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Command::Get(a), Command::Get(b)) => a == b,
            (Command::Set(a1, a2), Command::Set(b1, b2)) => a1 == b1 && a2 == b2,
            (Command::Del(a), Command::Del(b)) => a == b,
            _ => false,
        }
    }
}

impl Command {
    pub fn parse_request(request: &[u8]) -> Result<Command> {
        let request = resolve_command_payload(request)?;
        let mut tokens = request.split(|b| *b == b' ');
        let command = tokens
            .next()
            .ok_or_else(|| anyhow::anyhow!("Invalid request"))?;

        match command {
            b"GET" => {
                let key = tokens
                    .next()
                    .ok_or_else(|| anyhow::anyhow!("Expected key for GET"))?
                    .to_vec();
                Ok(Command::Get(key))
            }
            b"SET" => {
                let key = tokens
                    .next()
                    .ok_or_else(|| anyhow::anyhow!("Expected key for SET"))?
                    .to_vec();
                let value = tokens
                    .next()
                    .ok_or_else(|| anyhow::anyhow!("Expected value for SET"))?
                    .to_vec();
                Ok(Command::Set(key, value))
            }
            b"DEL" => {
                let key = tokens
                    .next()
                    .ok_or_else(|| anyhow::anyhow!("Expected key for DEL"))?
                    .to_vec();
                Ok(Command::Del(key))
            }
            _ => Err(anyhow::anyhow!("Invalid command")),
        }
    }
}

fn resolve_command_payload(request: &[u8]) -> Result<Vec<u8>> {
    let mut items = Vec::new();
    if request.len() < 4 {
        return Err(anyhow::anyhow!("Invalid request"));
    }
    let length = LittleEndian::read_u32(&request[..4]);
    println!("lenght: {length}");
    if length < 2 || length > 3 {
        return Err(anyhow::anyhow!("Invalid length"));
    }
    let mut current_pos = 4;
    for _ in 0..length {
        let item_length = LittleEndian::read_u32(&request[current_pos..current_pos + 4]);
        current_pos += 4;
        let item = &request[current_pos..current_pos + item_length as usize];
        items.extend_from_slice(item);
        items.push(b' ');
        current_pos += item_length as usize;
    }
    items.pop();
    Ok(items)
}

#[cfg(test)]
mod tests {

    use super::*;
    use std::str::from_utf8;

    fn generate_command_payload(args: Vec<String>) -> Vec<u8> {
        let mut request = Vec::new();
        request.resize(4, 0);
        LittleEndian::write_u32(&mut request[..4], args.len() as u32);
        let mut current_pos = 4;
        for arg in args.iter() {
            request.resize(current_pos + 4, 0);
            LittleEndian::write_u32(&mut request[current_pos..current_pos + 4], arg.len() as u32);
            current_pos += 4;
            request.extend_from_slice(arg.as_bytes());
            current_pos += arg.len();
        }
        request
    }

    #[test]
    fn test_generate_command_payload() {
        let args = vec!["GET".to_string(), "key".to_string()];
        let request = generate_command_payload(args);
        assert_eq!(2, LittleEndian::read_u32(&request[..4]));
        assert_eq!("GET", from_utf8(&request[8..11]).unwrap());
    }

    #[test]
    fn test_resolve_command_request() {
        // Valid GET request
        let args = vec!["GET".to_string(), "key".to_string()];
        let request = generate_command_payload(args);
        let response = resolve_command_payload(&request);
        assert!(response.is_ok());
        let response = response.unwrap();
        let mut tokens = response.split(|b| *b == b' ');
        assert_eq!(tokens.next().unwrap(), b"GET");
        assert_eq!(tokens.next().unwrap(), b"key");

        // Valid SET request
        let args = vec!["SET".to_string(), "key".to_string(), "value".to_string()];
        let request = generate_command_payload(args);
        let response = resolve_command_payload(&request);
        assert!(response.is_ok());
        let response = response.unwrap();
        let mut tokens = response.split(|b| *b == b' ');
        assert_eq!(tokens.next().unwrap(), b"SET");
        assert_eq!(tokens.next().unwrap(), b"key");
        assert_eq!(tokens.next().unwrap(), b"value");

        // Invalid request (length < 4)
        let invalid_request = vec![0, 0, 0];
        let response = resolve_command_payload(&invalid_request);
        assert!(response.is_err());

        // Invalid request (length prefix < 2)
        let mut invalid_request = vec![0, 0, 0, 1];
        invalid_request.extend_from_slice(b"GET");
        let response = resolve_command_payload(&invalid_request);
        assert!(response.is_err());

        // Invalid request (length prefix > 3)
        let mut invalid_request = vec![0, 0, 0, 4];
        invalid_request.extend_from_slice(b"GET key value extra");
        let response = resolve_command_payload(&invalid_request);
        assert!(response.is_err());

        // Invalid request (incorrect item length)
        let mut invalid_request = vec![0, 0, 0, 2];
        invalid_request.extend_from_slice(&[0, 0, 0, 3]); // Length of "GET"
        invalid_request.extend_from_slice(b"GET");
        invalid_request.extend_from_slice(&[0, 0, 0, 5]); // Incorrect length
        invalid_request.extend_from_slice(b"key");
        let response = resolve_command_payload(&invalid_request);
        assert!(response.is_err());
    }

    #[test]
    fn test_parse_request() {
        let args = vec!["GET".to_string(), "name".to_string()];
        let request = generate_command_payload(args);
        let command = Command::parse_request(&request);
        assert!(command.is_ok());
        let command = command.unwrap();
        assert_eq!(command, Command::Get(b"name".to_vec()));

        let args = vec!["SET".to_string(), "age".to_string(), "32".to_string()];
        let request = generate_command_payload(args);
        let command = Command::parse_request(&request);
        assert!(command.is_ok());
        let command = command.unwrap();
        assert_eq!(command, Command::Set(b"age".to_vec(), b"32".to_vec()));

        let args = vec!["DEL".to_string(), "name".to_string()];
        let request = generate_command_payload(args);
        let command = Command::parse_request(&request);
        assert!(command.is_ok());
        let command = command.unwrap();
        assert_eq!(command, Command::Del(b"name".to_vec()));
    }
}
