use std::io::{Read, Write};

use cloudru::{Client, Result, obs::Bucket};
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

fn object_writer_test(bucket: Bucket) -> Result<()> {
    let obj = "test_writer.txt";
    let data: Vec<u8> = b"Quick brown fox jumps over lazy dog".to_vec();

    bucket.delete_object(obj)?;

    let mut writer = bucket.object_writer(obj)?;

    assert_eq!(writer.write(&data[0..10]).unwrap(), 10);
    assert_eq!(writer.write(&data[10..]).unwrap(), data.len() - 10);

    let mut data_read = vec![];
    bucket.get_object(obj, &mut data_read)?;
    assert_eq!(data, data_read);

    Ok(())
}



#[test]
fn obs() -> Result<()> {
    let bypass: bool = std::env::var("CLOUDRU_BYPASS_INTEGRATION_TESTS")
        .ok().unwrap_or_else(|| "true".to_owned()).parse().unwrap();
    if bypass { 
        eprintln!("Test bypassed as CLOUDRU_BYPASS_INTEGRATION_TESTS is set to true");
        return Ok(());
    }

    let cfg_ini = ini::Ini::load_from_file(".obs.test.config")?;
    let bucket_name = cfg_ini.general_section().get("bucket").unwrap();
    let credentials_id = cfg_ini.general_section().get("credentials_id").unwrap_or("default");

    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::DEBUG)
        .with_ansi(false)
        .finish();

    tracing::subscriber::set_global_default(subscriber)
        .expect("setting default subscriber failed");

    let client = Client::builder().credentials_id(credentials_id).build()?;
    let obs = client.obs()?;
    let bucket = obs.bucket(bucket_name.to_owned())?;

    basic_test(bucket.clone())?;
    object_reader_test(bucket.clone())?;
    object_writer_test(bucket.clone())?;

    Ok(())
}