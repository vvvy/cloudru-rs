


mod blocking {
    use std::io::{Read, Write};

    use cloudru::{*, blocking::{client::*, obs::Bucket}};
    
    
    use tracing::Level;
    use tracing_subscriber::FmtSubscriber;

    fn basic_test(bucket: Bucket) -> Result<()> {
        let data = b"Quick brown fox jumps over lazy dog".to_vec();
        let () = bucket.put_object("test.txt", data.clone())?;
    
        let mut data_out = vec![];
        let () = bucket.get_object("test.txt", &mut data_out)?;
    
        assert_eq!(data, data_out);
    
        let () = bucket.put_object("test2.txt", data.clone())?;
        let _list = bucket.list(None)?;
        //println!("{list:?}");
    
        Ok(())
    }
    
    fn object_io_read_test(bucket: Bucket) -> Result<()> {
        let data: Vec<u8> = b"Quick brown fox jumps over lazy dog".to_vec();
        let () = bucket.put_object("test.txt", data.clone())?;
    
        let mut reader = bucket.object_io("test.txt")?;
    
        let mut buf = [0; 10];
    
        assert_eq!(reader.read(&mut buf).unwrap(), 10);
        assert_eq!(buf, &data[0..10]);
    
        assert_eq!(reader.read(&mut buf).unwrap(), 10);
        assert_eq!(buf, &data[10..20]);
    
        Ok(())
    }
    
    fn object_io_write_test(bucket: Bucket) -> Result<()> {
        let obj = "test_writer.txt";
        let data: Vec<u8> = b"Quick brown fox jumps over lazy dog".to_vec();
    
        bucket.delete_object(obj)?;
    
        let mut writer = bucket.object_io(obj)?;
    
        assert_eq!(writer.write(&data[0..10]).unwrap(), 10);
        assert_eq!(writer.write(&data[10..]).unwrap(), data.len() - 10);
    
        let mut data_read = vec![];
        bucket.get_object(obj, &mut data_read)?;
        assert_eq!(data, data_read);
    
        Ok(())
    }

    fn object_io_empty_write_test(bucket: Bucket) -> Result<()> {
        let obj = "test_writer.txt";
        let data = vec![];
    
        bucket.delete_object(obj)?;
    
        let mut writer = bucket.object_io(obj)?;
    
        assert_eq!(writer.write(&data).unwrap(), 0);
    
        let mut data_read = vec![];
        bucket.get_object(obj, &mut data_read)?;
        assert_eq!(data, data_read);
    
        Ok(())
    }

    fn object_empty_put_test(bucket: Bucket) -> Result<()> {
        let obj = "test_writer.txt";
        let data = vec![];
    
        bucket.delete_object(obj)?;
    
        let () = bucket.put_object(obj, data.clone())?;
    
        let mut data_read = vec![];
        bucket.get_object(obj, &mut data_read)?;
        assert_eq!(data, data_read);
    
        Ok(())
    }
        
    
    #[ignore = "cloudru integration tests are ignored by default"]
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
    
        let client = Client::builder().credentials_id(credentials_id).build()?;
        let obs = client.obs()?;
        let bucket = obs.bucket(bucket_name.to_owned())?;
    
        basic_test(bucket.clone())?;
        object_io_read_test(bucket.clone())?;
        object_io_write_test(bucket.clone())?;
        object_io_empty_write_test(bucket.clone())?;
        object_empty_put_test(bucket.clone())?;
    
        Ok(())
    }
    
}

mod nonblocking {
    use bytes::Bytes;
    use cloudru::{*, nonblocking::{client::*, obs::Bucket}};
    
    
    use tracing::Level;
    use tracing_subscriber::FmtSubscriber;

    async fn basic_test(bucket: Bucket) -> Result<()> {
        let data = Bytes::from_static(b"Quick brown fox jumps over lazy dog");
        let () = bucket.put_object("test.txt", data.clone()).await?;
    
        let data_out = bucket.get_object("test.txt").await?;
    
        assert_eq!(data, data_out);
    
        let () = bucket.put_object("test2.txt", data.clone()).await?;
        let _list = bucket.list(None).await?;
        //println!("{list:?}");
    
        Ok(())
    }
    
    async fn object_io_read_test(bucket: Bucket) -> Result<()> {
        let data = Bytes::from_static(b"Quick brown fox jumps over lazy dog");
        let () = bucket.put_object("test.txt", data.clone()).await?;
    
        let mut reader = bucket.object_io("test.txt").await?;
    
    
        assert_eq!(reader.read(10).await.unwrap(), data.slice(0..10));
        assert_eq!(reader.read(10).await.unwrap(), data.slice(10..20));
    
        Ok(())
    }
    
    async fn object_io_write_test(bucket: Bucket) -> Result<()> {
        let obj = "test_writer.txt";
        let data = Bytes::from_static(b"Quick brown fox jumps over lazy dog");
    
        bucket.delete_object(obj).await?;
    
        let mut writer = bucket.object_io(obj).await?;
    
        assert_eq!(writer.write(data.slice(0..10)).await.unwrap(), 10);
        assert_eq!(writer.write(data.slice(10..)).await.unwrap(), data.len() - 10);
    
        let data_read = bucket.get_object(obj).await?;
        assert_eq!(data, data_read);
    
        Ok(())
    }

    async fn object_io_empty_write_test(bucket: Bucket) -> Result<()> {
        let obj = "test_writer.txt";
        let data = Bytes::from_static(&[]);
    
        bucket.delete_object(obj).await?;
    
        let mut writer = bucket.object_io(obj).await?;
    
        assert_eq!(writer.write(data.clone()).await.unwrap(), 0);
    
        let data_read = bucket.get_object(obj).await?;
        assert_eq!(data, data_read);
    
        Ok(())
    }

    async fn object_empty_put_test(bucket: Bucket) -> Result<()> {
        let obj = "test_writer.txt";
        let data = Bytes::from_static(&[]);
    
        bucket.delete_object(obj).await?;
    
        let () = bucket.put_object(obj, data.clone()).await?;
    
        let data_read = bucket.get_object(obj).await?;
        assert_eq!(data, data_read);
    
        Ok(())
    }
        
    
    #[ignore = "cloudru integration tests are ignored by default"]
    #[tokio::test]
    async fn obs() -> Result<()> {
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
    
        basic_test(bucket.clone()).await?;
        object_io_read_test(bucket.clone()).await?;
        object_io_write_test(bucket.clone()).await?;
        object_io_empty_write_test(bucket.clone()).await?;
        object_empty_put_test(bucket.clone()).await?;
    
        Ok(())
    }
    
}

