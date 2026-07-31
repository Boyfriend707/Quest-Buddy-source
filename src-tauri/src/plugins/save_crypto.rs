use aes::cipher::{block_padding::Pkcs7, BlockDecryptMut, BlockEncryptMut, KeyInit};
use aes::Aes256;
use aes::cipher::generic_array::GenericArray;

const KEY: &[u8; 32] = b"UKu52ePUBwetZ9wNX88o54dnfKRu0T1l";

pub fn decrypt_save(data: &[u8]) -> Option<String> {
    let data = data.get(22..)?;

    let (len, offset) = read_vlq(data);
    let encoded = data.get(offset..offset + len)?;
    let encoded = if encoded.last() == Some(&0x0B) {
        &encoded[..encoded.len() - 1]
    } else {
        encoded
    };

    let encrypted = base64::Engine::decode(&base64::engine::general_purpose::STANDARD, encoded).ok()?;
    let key = GenericArray::from_slice(KEY);
    let cipher = Aes256::new(key);
    let mut buf = encrypted.clone();
    let pt = cipher.decrypt_padded_mut::<Pkcs7>(&mut buf).ok()?;
    String::from_utf8(pt.to_vec()).ok()
}

pub fn encrypt_save(json: &str, original: &[u8]) -> Option<Vec<u8>> {
    let header = original.get(..22)?;

    let key = GenericArray::from_slice(KEY);
    let cipher = Aes256::new(key);

    let json_bytes = json.as_bytes();
    let block_size = 16;
    let buf_len = ((json_bytes.len() + block_size - 1) / block_size + 1) * block_size;
    let mut buf = vec![0u8; buf_len];
    buf[..json_bytes.len()].copy_from_slice(json_bytes);

    let encrypted = cipher.encrypt_padded_mut::<Pkcs7>(&mut buf, json_bytes.len()).ok()?;
    let encoded = base64::Engine::encode(&base64::engine::general_purpose::STANDARD, encrypted);

    let vlq = write_vlq(encoded.len());

    let mut result = header.to_vec();
    result.extend_from_slice(&vlq);
    result.extend_from_slice(encoded.as_bytes());
    result.push(0x0B);
    Some(result)
}

fn read_vlq(data: &[u8]) -> (usize, usize) {
    let mut len = 0usize;
    let mut shift = 0;
    let mut offset = 0;
    for &byte in data {
        offset += 1;
        len |= ((byte & 0x7F) as usize) << shift;
        if byte & 0x80 == 0 {
            break;
        }
        shift += 7;
    }
    (len, offset)
}

fn write_vlq(mut value: usize) -> Vec<u8> {
    let mut result = Vec::new();
    loop {
        let mut byte = (value & 0x7F) as u8;
        value >>= 7;
        if value > 0 {
            byte |= 0x80;
        }
        result.push(byte);
        if value == 0 {
            break;
        }
    }
    result
}
