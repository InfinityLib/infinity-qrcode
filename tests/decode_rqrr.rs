use infinitylib_qrcode_rs::{CorrectionLevel, Mode, QrCode};

/// Renders the matrix as a high-contrast greyscale image and decodes it with
/// `rqrr`. This catches real-world scanning problems that matrix comparison
/// alone would not reveal.
fn decode(qr: &QrCode) -> String {
    const SCALE: usize = 8;
    const QUIET_ZONE: usize = 4;

    let size = qr.size();
    let image_size = (size + 2 * QUIET_ZONE) * SCALE;

    let mut image = rqrr::PreparedImage::prepare_from_greyscale(image_size, image_size, |x, y| {
        let module_x = x / SCALE;
        let module_y = y / SCALE;

        if module_x < QUIET_ZONE || module_y < QUIET_ZONE {
            return 255;
        }

        let module_x = module_x - QUIET_ZONE;
        let module_y = module_y - QUIET_ZONE;

        if module_x >= size || module_y >= size {
            return 255;
        }

        if qr.modules()[module_y][module_x] == 1 {
            0
        } else {
            255
        }
    });

    let grids = image.detect_grids();
    assert_eq!(grids.len(), 1, "expected exactly one detected QR grid");

    let (_, content) = grids
        .into_iter()
        .next()
        .expect("one grid was detected")
        .decode()
        .expect("QR grid should decode");

    content
}

#[test]
fn decodes_version_1_for_all_correction_levels() {
    let cases = [
        (CorrectionLevel::Low, "Hello, world!"),
        (CorrectionLevel::Medium, "Hello, world!"),
        (CorrectionLevel::Quartile, "Hello!"),
        (CorrectionLevel::High, "Hi!"),
    ];

    for (level, data) in cases {
        let qr = QrCode::encode_version(data, 1, level).expect("QR generation failed");
        assert_eq!(decode(&qr), data, "failed to decode {level:?}");
    }
}

#[test]
fn decodes_version_1_for_all_supported_modes() {
    let cases = [
        (Mode::Numeric, "01234567"),
        (Mode::Alphanumeric, "HELLO WORLD"),
        (Mode::Byte, "Hi!"),
    ];

    for (mode, data) in cases {
        let qr = QrCode::encode_with_mode(data, mode, 1, CorrectionLevel::Medium)
            .expect("QR generation failed");
        assert_eq!(decode(&qr), data, "failed to decode {mode:?}");
    }
}

#[test]
fn decodes_version_1_with_automatic_mode_detection() {
    for data in ["01234567", "HELLO WORLD", "Hello, world!"] {
        let qr = QrCode::encode(data, CorrectionLevel::Medium).expect("QR generation failed");
        assert_eq!(decode(&qr), data, "failed to decode {data:?}");
    }
}

#[test]
fn decodes_versions_2_to_6() {
    let lengths = [0, 7, 14, 24, 34, 44, 58];

    for version in 2..=6u8 {
        let data = "a".repeat(lengths[version as usize]);
        let qr = QrCode::encode_version(&data, version, CorrectionLevel::High)
            .expect("QR generation failed");

        assert_eq!(qr.version(), version);
        assert_eq!(decode(&qr), data, "failed to decode version {version}");
    }
}

#[test]
fn decodes_large_versions() {
    let cases = [(7u8, 50usize), (10, 100), (20, 300), (40, 1000)];

    for (version, length) in cases {
        let data = "a".repeat(length);
        let qr = QrCode::encode_version(&data, version, CorrectionLevel::High)
            .expect("QR generation failed");

        assert_eq!(qr.version(), version);
        assert_eq!(decode(&qr), data, "failed to decode version {version}");
    }
}

#[test]
fn decodes_automatic_version_selection() {
    let data = "a".repeat(45);
    let qr = QrCode::encode(&data, CorrectionLevel::High).expect("QR generation failed");

    assert_eq!(qr.version(), 6);
    assert_eq!(decode(&qr), data);
}
