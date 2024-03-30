use serde_derive::Deserialize;

#[derive(Debug, Default)]
pub struct ListBucketRequest<'t> {
    pub prefix: Option<&'t str>,
    pub marker: Option<&'t str>,
    pub max_keys: Option<u32>,
    pub delimiter: Option<&'t str>,
    pub key_marker: Option<&'t str>,
    pub version_id_marker: Option<&'t str>,
}

/*
<?xml version=\"1.0\" encoding=\"UTF-8\" standalone=\"yes\"?>
<ListBucketResult xmlns=\"http://obs.hc.sbercloud.ru/doc/2015-06-30/\">
    <Name>rust-api-test</Name>
    <Prefix></Prefix>
    <KeyCount>2</KeyCount>
    <MaxKeys>1000</MaxKeys>
    <IsTruncated>false</IsTruncated>
    <Contents>
        <Key>test.txt</Key>
        <LastModified>2023-08-20T11:04:49.119Z</LastModified>
        <ETag>\"bf0b7283b196369ba8723a79750d937b\"</ETag>
        <Size>35</Size>
        <StorageClass>STANDARD</StorageClass>
    </Contents>
    <Contents>
        <Key>test2.txt</Key>
        <LastModified>2023-08-20T11:04:49.250Z</LastModified>
        <ETag>\"bf0b7283b196369ba8723a79750d937b\"</ETag>
        <Size>35</Size>
        <StorageClass>STANDARD</StorageClass>
    </Contents>
</ListBucketResult>
*/
#[derive(Deserialize, Debug)]
pub struct ListBucketResult {
    #[serde(rename="Name")]
    pub name: String,
    
    #[serde(rename="Prefix")]
    pub prefix: String,
    
    #[serde(rename="KeyCount")]
    pub key_count: Option<u64>,
    
    #[serde(rename="MaxKeys")]
    pub max_keys: Option<u64>,

    #[serde(rename="IsTruncated")]
    pub is_truncated: Option<bool>,

    #[serde(rename="Delimiter")]
    pub delimeter: Option<String>,

    #[serde(rename="Marker")]
    pub marker: Option<String>,

    #[serde(rename="NextMarker")]
    pub next_marker: Option<String>,

    //#[serde(rename="CommonPrefixes")]
    //pub common_prefixes: Option<String>,

    #[serde(rename="Contents")]
    pub contents: Option<Vec<ListBucketContents>>,
}

#[derive(Deserialize, Debug)]
pub struct ListBucketContents {
    #[serde(rename="Key")]
    pub key: String,

    #[serde(rename="LastModified")]
    pub last_modified: String,
    
    #[serde(rename="ETag")]
    pub etag: String,
    
    #[serde(rename="Type")]
    pub type_: Option<String>,
    
    #[serde(rename="Size")]
    pub size: u64,
    
    #[serde(rename="StorageClass")]
    pub storage_class: String,

    #[serde(rename="Owner")]
    pub owner: Option<Owner>,
}

#[derive(Deserialize, Debug)]
pub struct Owner {
    #[serde(rename="ID")]
    pub id: String,

    #[serde(rename="DisplayName")]
    pub display_name: Option<String>,
}


/// Object metadata returned by [obs::Bucket::get_object_meta]
pub struct ObjectMeta {
    /// Length of the object in bytes as seen by OBS 
    pub content_length: Option<u64>,
    /// Type of the object's content
    pub content_type: Option<String>,
    /// Date the object was last modified, like "WED, 01 Jul 2015 01:19:21 GMT"
    pub last_modified: Option<String>,
}
