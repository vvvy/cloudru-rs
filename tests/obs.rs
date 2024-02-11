use std::io::Read;

use cloudru::{*, obs::*, config::*};
use tracing::Level;
use tracing_subscriber::FmtSubscriber;


fn basic_test(bucket: Bucket) -> Result<()> {
    let data = b"Quick brown fox jumps over lazy dog".to_vec();
    let () = bucket.put_object("test.txt", data.clone())?;

    let mut data_out = vec![];
    let () = bucket.get_object("test.txt", &mut data_out)?;

    assert_eq!(data, data_out);

    let () = bucket.put_object("test2.txt", data.clone())?;
    let list = bucket.list(None)?;
    println!("{list:?}");

    Ok(())
}

fn object_reader_test(bucket: Bucket) -> Result<()> {
    let data: Vec<u8> = b"Quick brown fox jumps over lazy dog".to_vec();
    let () = bucket.put_object("test.txt", data.clone())?;

    let mut reader = bucket.object_reader("test.txt")?;

    let mut buf = [0; 10];

    assert_eq!(reader.read(&mut buf).unwrap(), 10);
    assert_eq!(buf, &data[0..10]);

    assert_eq!(reader.read(&mut buf).unwrap(), 10);
    assert_eq!(buf, &data[10..20]);

    Ok(())
}


#[test]
fn obs() -> Result<()> {
    let cfg_ini = ini::Ini::load_from_file(".obs.test.config")?;
    let bucket_name = cfg_ini.general_section().get("bucket").unwrap();
    let credentials_id = cfg_ini.general_section().get("credentials_id").unwrap_or("default");

    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::DEBUG)
        .with_ansi(false)
        .finish();

    tracing::subscriber::set_global_default(subscriber)
        .expect("setting default subscriber failed");

    let config = read_config(DEFAULT_CONFIG_FILE.to_owned())?;
    let aksk = read_credentials(DEFAULT_CREDENTIALS_FILE.to_owned(), credentials_id.to_owned())?;
    let endpoint = config.endpoint.resolve(config::svc_id::obs, None)?.to_string();

    let bucket = Bucket::new(bucket_name.to_owned(), endpoint, aksk)?;

    basic_test(bucket.clone())?;
    object_reader_test(bucket)?;

    Ok(())
}