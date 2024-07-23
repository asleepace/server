const BASE64_CHARS: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";

/**
 * Encode a byte vector into a base64 string.
 */
pub fn base64_encode(data: &[u8]) -> String {
    let mut output = String::new();
    let mut i = 0;
    while i < data.len() {
        let mut n = u32::from(data[i]) << 16;
        if i + 1 < data.len() {
            n += u32::from(data[i + 1]) << 8;
        }
        if i + 2 < data.len() {
            n += u32::from(data[i + 2]);
        }
        output.push(BASE64_CHARS[(n >> 18 & 63) as usize] as char);
        output.push(BASE64_CHARS[(n >> 12 & 63) as usize] as char);
        output.push(if i + 1 < data.len() {
            BASE64_CHARS[(n >> 6 & 63) as usize] as char
        } else {
            '='
        });
        output.push(if i + 2 < data.len() {
            BASE64_CHARS[(n & 63) as usize] as char
        } else {
            '='
        });

        i += 3;
    }

    output
}

/**
 * Decode a base64 string into a byte vector.
 */
pub fn base64_decode(input: &str) -> Result<Vec<u8>, &'static str> {
    let input = input.trim_end_matches('=');
    let mut output = Vec::with_capacity(input.len() * 3 / 4);

    // Create a lookup table for base64 characters
    let mut reverse_lookup = [0u8; 256];
    for (i, &c) in BASE64_CHARS.iter().enumerate() {
        reverse_lookup[c as usize] = i as u8;
    }

    let mut buf = 0u32;
    let mut buf_len = 0;

    for &c in input.as_bytes() {
        if c == b'=' {
            break;
        }
        let v = reverse_lookup[c as usize];
        if v == 0 && c != BASE64_CHARS[0] {
            return Err("Invalid base64 character");
        }
        buf = (buf << 6) | u32::from(v);
        buf_len += 6;
        if buf_len >= 8 {
            buf_len -= 8;
            output.push((buf >> buf_len) as u8);
        }
    }

    Ok(output)
}
