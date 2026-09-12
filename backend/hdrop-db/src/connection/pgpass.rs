use std::path::Path;

pub(super) fn read(path: &Path) -> Vec<u8> {
    super::SecretFile::Passfile.read(path).unwrap_or_default()
}

/// Match one of the four identifying fields and return the rest of the line.
fn match_field<'a>(line: &'a [u8], expected: &[u8]) -> Option<&'a [u8]> {
    if let Some(rest) = line.strip_prefix(b"*:") {
        return Some(rest);
    }
    let mut bytes = line.iter();
    for expected_byte in expected {
        let byte = match bytes.next()? {
            b':' => return None,
            b'\\' => bytes.next()?,
            byte => byte,
        };
        if byte != expected_byte {
            return None;
        }
    }
    bytes.as_slice().strip_prefix(b":")
}

pub(super) fn password(contents: &[u8], fields: [&[u8]; 4]) -> Option<Vec<u8>> {
    contents.split(|&byte| byte == b'\n').find_map(|line| {
        let line = line.strip_suffix(b"\r").unwrap_or(line);
        if line.starts_with(b"#") || line.contains(&0) {
            return None;
        }
        let raw_password = fields.into_iter().try_fold(line, match_field)?;
        let mut bytes = raw_password.iter().copied();
        let mut password = Vec::new();
        while let Some(byte) = bytes.next() {
            match byte {
                b':' => break,
                b'\\' => password.push(bytes.next().unwrap_or(b'\\')),
                byte => password.push(byte),
            }
        }
        Some(password)
    })
}

#[cfg(test)]
mod tests {
    #[cfg(unix)]
    use std::os::unix::fs::OpenOptionsExt;
    use std::{fs::OpenOptions, io::Write};

    use super::*;

    #[test]
    fn first_match_respects_wildcards_and_escaped_literal_fields() {
        let contents = b"#*:*:*:*:comment\r\n\
                         \\*:*:*:*:literal-star\r\n\
                         host:*:db:user\\:name:first\\:pass\\\\word\r\n\
                         *:*:*:*:fallback";
        assert_eq!(
            password(contents, [b"host", b"5432", b"db", b"user:name"]).as_deref(),
            Some(&b"first:pass\\word"[..])
        );
        assert_eq!(
            password(contents, [b"*", b"", b"", b""]).as_deref(),
            Some(&b"literal-star"[..])
        );
        assert_eq!(
            password(contents, [b"host", b"5432", b"other", b"user:name"]).as_deref(),
            Some(&b"fallback"[..])
        );
    }

    #[test]
    fn parses_raw_bytes_and_preserves_password_field_boundaries() {
        let fields: [&[u8]; 4] = [b"\xff", b"5432", b"db", b"user"];
        assert_eq!(
            password(b"\xff:5432:db:user:\xfe\\:x:ignored", fields).as_deref(),
            Some(&b"\xfe:x"[..])
        );
        assert_eq!(password(b"*:*:*:*:\\", fields).as_deref(), Some(&b"\\"[..]));
        assert_eq!(password(b"*:*:*:*:", fields), Some(Vec::new()));
        assert_eq!(
            password(b"badline\n*:*:other:*:unused\n*:*:*:user", fields),
            None
        );
    }

    #[test]
    fn reads_only_regular_private_files() {
        let dir = std::env::temp_dir().join(format!("hdrop-pgpass-{}", uuid::Uuid::new_v4()));
        std::fs::create_dir(&dir).unwrap();
        let path = dir.join("pgpass");
        assert!(read(&dir).is_empty());
        assert!(read(&path).is_empty());

        let mut options = OpenOptions::new();
        options.create_new(true).write(true);
        #[cfg(unix)]
        options.mode(0o600);
        options.open(&path).unwrap().write_all(b"fixture").unwrap();
        assert_eq!(read(&path), b"fixture");
        #[cfg(unix)]
        {
            use std::{
                ffi::CString,
                os::unix::{ffi::OsStrExt, fs::PermissionsExt},
            };
            std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o640)).unwrap();
            assert!(read(&path).is_empty());
            std::fs::remove_file(&path).unwrap();
            let fifo = CString::new(path.as_os_str().as_bytes()).unwrap();
            // SAFETY: fifo owns the NUL-terminated pathname passed to mkfifo.
            assert_eq!(unsafe { libc::mkfifo(fifo.as_ptr(), 0o600) }, 0);
            assert!(read(&path).is_empty());
        }
        std::fs::remove_file(path).unwrap();
        std::fs::remove_dir(dir).unwrap();
    }
}
